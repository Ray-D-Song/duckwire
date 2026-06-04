use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use pgwire::api::auth::StartupHandler;
use pgwire::api::auth::noop::NoopStartupHandler;
use pgwire::api::portal::{Format, Portal};
use pgwire::api::query::{ExtendedQueryHandler, SimpleQueryHandler};
use pgwire::api::results::{
    DataRowEncoder, DescribePortalResponse, DescribeStatementResponse, FieldInfo, QueryResponse,
    Response, Tag,
};
use pgwire::api::stmt::{NoopQueryParser, StoredStatement};
use pgwire::api::{ClientInfo, PgWireServerHandlers, Type};
use pgwire::error::PgWireResult;
use tracing::{debug, error, info};

use crate::backend::connection::DuckDBConnection;
use crate::backend::result::DuckDBQueryResult;
use crate::types::mapping::{
    build_schema_from_columns_with_format, column_type_to_pg, encode_duckdb_owned_value,
    requested_format_for_type,
};

pub struct DuckWireHandler {
    connection: Arc<DuckDBConnection>,
    query_parser: Arc<NoopQueryParser>,
}

impl DuckWireHandler {
    pub fn new(connection: Arc<DuckDBConnection>) -> Self {
        Self {
            connection,
            query_parser: Arc::new(NoopQueryParser::new()),
        }
    }

    fn execute_query(&self, query: &str) -> PgWireResult<Vec<Response>> {
        self.execute_query_with_format(query, None)
    }

    fn execute_query_with_format(
        &self,
        query: &str,
        result_format: Option<&Format>,
    ) -> PgWireResult<Vec<Response>> {
        let mut session = self.connection.create_session();
        let result = session.execute(query).map_err(|e| {
            error!(query = %query.trim(), error = %e, "query failed");
            pgwire::error::PgWireError::ApiError(Box::new(e))
        })?;

        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                let schema = build_schema_from_columns_with_format(&columns, result_format)
                    .map_err(|e| pgwire::error::PgWireError::ApiError(Box::new(e)))?;

                let row_count = data.len();
                let data_rows: Vec<_> = data
                    .into_iter()
                    .map(|row_values| {
                        let mut encoder = DataRowEncoder::new(schema.clone());
                        for value in row_values {
                            encode_duckdb_owned_value(&mut encoder, &value)
                                .map_err(|e| pgwire::error::PgWireError::ApiError(Box::new(e)))?;
                        }
                        Ok(encoder.take_row())
                    })
                    .collect::<PgWireResult<Vec<_>>>()?;

                info!(rows = row_count, "rows returned");
                let s = stream::iter(data_rows.into_iter().map(Ok));
                Ok(vec![Response::Query(QueryResponse::new(schema, s))])
            }
            DuckDBQueryResult::Affected(n) => {
                info!(affected = n, "execution complete");
                Ok(vec![Response::Execution(
                    Tag::new("OK").with_rows(n as usize),
                )])
            }
            DuckDBQueryResult::Status(s) => {
                info!(status = %s, "execution complete");
                Ok(vec![Response::Execution(Tag::new(&s))])
            }
            DuckDBQueryResult::Empty => {
                info!("query suppressed (empty result)");
                Ok(vec![Response::Execution(Tag::new("OK"))])
            }
        }
    }

    fn get_query_schema(&self, sql: &str) -> PgWireResult<Vec<FieldInfo>> {
        self.get_query_schema_with_format(sql, None)
    }

    fn get_query_schema_with_format(
        &self,
        sql: &str,
        result_format: Option<&Format>,
    ) -> PgWireResult<Vec<FieldInfo>> {
        let null_sql = replace_params_with_null(sql);
        let rewritten = self
            .connection
            .transpiler
            .rewrite(&null_sql)
            .unwrap_or_else(|_| null_sql.clone());
        if rewritten.trim().is_empty() {
            return Ok(vec![]);
        }

        let conn = self.connection.conn.lock().map_err(|e| {
            pgwire::error::PgWireError::ApiError(Box::new(duckdb::Error::InvalidColumnName(
                format!("{e}"),
            )))
        })?;

        let upper = rewritten.trim().to_uppercase();
        if upper.starts_with("INSERT") || upper.starts_with("UPDATE") || upper.starts_with("DELETE")
        {
            return Ok(vec![]);
        }

        let mut stmt = match conn.prepare(&rewritten) {
            Ok(s) => s,
            Err(e) => {
                rollback_after_schema_error(&conn, &rewritten, &e);
                return Ok(vec![]);
            }
        };
        if let Err(e) = stmt.execute([]) {
            drop(stmt);
            rollback_after_schema_error(&conn, &rewritten, &e);
            return Ok(vec![]);
        }
        let column_count = stmt.column_count();
        if column_count == 0 {
            return Ok(vec![]);
        }
        let column_names = stmt.column_names();
        let fields: Vec<FieldInfo> = (0..column_count)
            .map(|i| {
                let name = column_names[i].clone();
                let pg_type =
                    column_type_to_pg(&name, &stmt.column_type(i)).unwrap_or(Type::UNKNOWN);
                let format = requested_format_for_type(result_format, i, &pg_type);
                FieldInfo::new(name, None, None, pg_type, format)
            })
            .collect();
        Ok(fields)
    }
}

fn rollback_after_schema_error(conn: &duckdb::Connection, sql: &str, err: &duckdb::Error) {
    match conn.execute_batch("ROLLBACK") {
        Ok(_) => info!(
            query = %sql.trim(),
            error = %err,
            "rolled back after schema probe error"
        ),
        Err(rollback_err) => debug!(
            query = %sql.trim(),
            error = %err,
            rollback_error = %rollback_err,
            "rollback after schema probe error was not needed or failed"
        ),
    }
}

fn replace_params_with_null(sql: &str) -> String {
    let mut result = sql.to_string();
    let mut i = 1;
    loop {
        let placeholder = format!("${}", i);
        if !result.contains(&placeholder) {
            break;
        }
        result = result.replacen(&placeholder, "NULL", 1);
        i += 1;
    }
    result
}

#[async_trait]
impl NoopStartupHandler for DuckWireHandler {}

#[async_trait]
impl SimpleQueryHandler for DuckWireHandler {
    async fn do_query<C>(&self, _client: &mut C, query: &str) -> PgWireResult<Vec<Response>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        info!(query = %query.trim(), "query");
        self.execute_query(query)
    }
}

#[async_trait]
impl ExtendedQueryHandler for DuckWireHandler {
    type Statement = String;
    type QueryParser = NoopQueryParser;

    fn query_parser(&self) -> Arc<Self::QueryParser> {
        self.query_parser.clone()
    }

    async fn do_query<C>(
        &self,
        _client: &mut C,
        portal: &Portal<Self::Statement>,
        _max_rows: usize,
    ) -> PgWireResult<Response>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let original_sql = &portal.statement.statement;
        debug!(
            sql = %original_sql.trim(),
            params = portal.parameter_len(),
            param_types = ?portal.statement.parameter_types,
            "execute portal"
        );
        let query = substitute_params(original_sql, portal);
        debug!(substituted = %query.trim(), "execute after param substitution");
        info!(query = %query.trim(), "extended query");
        let mut responses =
            self.execute_query_with_format(&query, Some(&portal.result_column_format))?;
        Ok(responses.remove(0))
    }

    async fn do_describe_statement<C>(
        &self,
        _client: &mut C,
        target: &StoredStatement<Self::Statement>,
    ) -> PgWireResult<DescribeStatementResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        debug!(sql = %target.statement.trim(), "describe statement");
        let schema = self.get_query_schema(&target.statement).unwrap_or_default();
        let param_types: Vec<Type> = target
            .parameter_types
            .iter()
            .filter_map(|f| f.clone())
            .collect();
        debug!(cols = schema.len(), param_types = ?param_types, "describe statement response");
        Ok(DescribeStatementResponse::new(param_types, schema))
    }

    async fn do_describe_portal<C>(
        &self,
        _client: &mut C,
        target: &Portal<Self::Statement>,
    ) -> PgWireResult<DescribePortalResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let sql = substitute_params(&target.statement.statement, target);
        debug!(sql = %sql.trim(), "describe portal");
        let schema =
            match self.get_query_schema_with_format(&sql, Some(&target.result_column_format)) {
                Ok(fields) => fields,
                Err(_) => vec![],
            };
        debug!(cols = schema.len(), "describe portal response");
        Ok(DescribePortalResponse::new(schema))
    }
}

// Replaces $1, $2, ... placeholders with literal values.
// pgwire's extended query protocol sends parameters separately, but we transpile
// the query as plain SQL text, so we must inline them.
fn substitute_params(query: &str, portal: &Portal<String>) -> String {
    if portal.parameter_len() == 0 {
        return query.to_string();
    }
    let mut result = query.to_string();
    for i in 0..portal.parameter_len() {
        let param_type = portal
            .statement
            .parameter_types
            .get(i)
            .cloned()
            .flatten()
            .unwrap_or(Type::UNKNOWN);
        let lit = param_to_literal(portal, i, param_type);
        result = result.replacen(&format!("${}", i + 1), &lit, 1);
    }
    result
}

fn param_to_literal(portal: &Portal<String>, idx: usize, ptype: Type) -> String {
    match ptype {
        Type::INT2 => portal
            .parameter::<i16>(idx, &ptype)
            .map(|v| v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".into()))
            .unwrap_or_else(|_| "NULL".into()),
        Type::INT4 => portal
            .parameter::<i32>(idx, &ptype)
            .map(|v| v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".into()))
            .unwrap_or_else(|_| "NULL".into()),
        Type::INT8 => portal
            .parameter::<i64>(idx, &ptype)
            .map(|v| v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".into()))
            .unwrap_or_else(|_| "NULL".into()),
        Type::FLOAT4 => portal
            .parameter::<f32>(idx, &ptype)
            .map(|v| v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".into()))
            .unwrap_or_else(|_| "NULL".into()),
        Type::FLOAT8 => portal
            .parameter::<f64>(idx, &ptype)
            .map(|v| v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".into()))
            .unwrap_or_else(|_| "NULL".into()),
        Type::BOOL => portal
            .parameter::<bool>(idx, &ptype)
            .map(|v| {
                v.map(|b| {
                    if b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                })
                .unwrap_or_else(|| "NULL".into())
            })
            .unwrap_or_else(|_| "NULL".into()),
        // Fallback: treat unknown param types as strings, single-quoted with escaping.
        // NOTE: types like JSON/UUID may be passed as TEXT by some drivers, so we
        // escape single quotes only for known text types to avoid double-escaping.
        _ => portal
            .parameter::<String>(idx, &ptype)
            .map(|v| {
                v.map(|s| {
                    if matches!(
                        ptype,
                        Type::TEXT | Type::VARCHAR | Type::NAME | Type::BPCHAR
                    ) {
                        format!("'{}'", s.replace('\'', "''"))
                    } else {
                        format!("'{}'", s)
                    }
                })
                .unwrap_or_else(|| "NULL".into())
            })
            .unwrap_or_else(|_| "NULL".into()),
    }
}

pub struct DuckWireHandlerFactory {
    handler: Arc<DuckWireHandler>,
}

impl DuckWireHandlerFactory {
    pub fn new(connection: Arc<DuckDBConnection>) -> Self {
        Self {
            handler: Arc::new(DuckWireHandler::new(connection)),
        }
    }
}

impl PgWireServerHandlers for DuckWireHandlerFactory {
    fn simple_query_handler(&self) -> Arc<impl SimpleQueryHandler> {
        self.handler.clone()
    }

    fn extended_query_handler(&self) -> Arc<impl ExtendedQueryHandler> {
        self.handler.clone()
    }

    fn startup_handler(&self) -> Arc<impl StartupHandler> {
        self.handler.clone()
    }
}
