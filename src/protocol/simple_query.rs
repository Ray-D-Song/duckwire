use std::sync::{Arc, Mutex};

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
use crate::backend::session::DuckDBSession;
use crate::types::mapping::{build_schema_from_columns_with_format, encode_duckdb_owned_value};

struct ClientDuckDBSession {
    session: Mutex<DuckDBSession>,
}

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

    fn client_session<C>(&self, client: &C) -> PgWireResult<Arc<ClientDuckDBSession>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        if let Some(session) = client.session_extensions().get::<ClientDuckDBSession>() {
            return Ok(session);
        }

        let session = self.connection.create_session().map_err(|e| {
            error!(error = %e, "failed to create DuckDB session");
            pgwire::error::PgWireError::ApiError(Box::new(e))
        })?;
        Ok(client
            .session_extensions()
            .get_or_insert_with(|| ClientDuckDBSession {
                session: Mutex::new(session),
            }))
    }

    fn execute_query<C>(&self, client: &C, query: &str) -> PgWireResult<Vec<Response>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        self.execute_query_with_format(client, query, None)
    }

    fn execute_query_with_format<C>(
        &self,
        client: &C,
        query: &str,
        result_format: Option<&Format>,
    ) -> PgWireResult<Vec<Response>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let session = self.client_session(client)?;
        let result = {
            let mut session = session.session.lock().map_err(|e| {
                pgwire::error::PgWireError::ApiError(Box::new(duckdb::Error::InvalidColumnName(
                    format!("{e}"),
                )))
            })?;
            session.execute(query).map_err(|e| {
                error!(query = %query.trim(), error = %e, "query failed");
                pgwire::error::PgWireError::ApiError(Box::new(e))
            })?
        };

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

    fn get_query_schema<C>(&self, client: &C, sql: &str) -> PgWireResult<Vec<FieldInfo>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        self.get_query_schema_with_format(client, sql, None)
    }

    fn get_query_schema_with_format<C>(
        &self,
        client: &C,
        sql: &str,
        result_format: Option<&Format>,
    ) -> PgWireResult<Vec<FieldInfo>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let null_sql = replace_params_with_null(sql);
        let session = self.client_session(client)?;
        let columns = {
            let mut session = session.session.lock().map_err(|e| {
                pgwire::error::PgWireError::ApiError(Box::new(duckdb::Error::InvalidColumnName(
                    format!("{e}"),
                )))
            })?;
            session.query_columns(&null_sql)
        };
        let schema =
            build_schema_from_columns_with_format(&columns, result_format).map_err(|e| {
                pgwire::error::PgWireError::ApiError(Box::new(duckdb::Error::InvalidColumnName(
                    format!("{e}"),
                )))
            })?;
        Ok((*schema).clone())
    }
}

fn replace_params_with_null(sql: &str) -> String {
    replace_numeric_params(sql, |idx| {
        if idx > 0 {
            Some("NULL".to_string())
        } else {
            None
        }
    })
}

#[async_trait]
impl NoopStartupHandler for DuckWireHandler {}

#[async_trait]
impl SimpleQueryHandler for DuckWireHandler {
    async fn do_query<C>(&self, client: &mut C, query: &str) -> PgWireResult<Vec<Response>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        info!(query = %query.trim(), "query");
        self.execute_query(client, query)
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
        client: &mut C,
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
            self.execute_query_with_format(client, &query, Some(&portal.result_column_format))?;
        Ok(responses.remove(0))
    }

    async fn do_describe_statement<C>(
        &self,
        client: &mut C,
        target: &StoredStatement<Self::Statement>,
    ) -> PgWireResult<DescribeStatementResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        debug!(sql = %target.statement.trim(), "describe statement");
        let schema = self
            .get_query_schema(client, &target.statement)
            .unwrap_or_default();
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
        client: &mut C,
        target: &Portal<Self::Statement>,
    ) -> PgWireResult<DescribePortalResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let sql = substitute_params(&target.statement.statement, target);
        debug!(sql = %sql.trim(), "describe portal");
        let schema = match self.get_query_schema_with_format(
            client,
            &sql,
            Some(&target.result_column_format),
        ) {
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
    replace_numeric_params(query, |idx| {
        if idx == 0 || idx > portal.parameter_len() {
            return None;
        }
        let param_idx = idx - 1;
        let param_type = portal
            .statement
            .parameter_types
            .get(param_idx)
            .cloned()
            .flatten()
            .unwrap_or(Type::UNKNOWN);
        Some(param_to_literal(portal, param_idx, param_type))
    })
}

fn replace_numeric_params(
    sql: &str,
    mut replacement: impl FnMut(usize) -> Option<String>,
) -> String {
    let bytes = sql.as_bytes();
    let mut result = String::with_capacity(sql.len());
    let mut i = 0;
    let mut in_string = false;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                result.push('\'');
                if in_string && bytes.get(i + 1) == Some(&b'\'') {
                    result.push('\'');
                    i += 2;
                    continue;
                }
                in_string = !in_string;
                i += 1;
            }
            b'$' if !in_string => {
                let start = i + 1;
                let mut end = start;
                while bytes.get(end).map(|b| b.is_ascii_digit()).unwrap_or(false) {
                    end += 1;
                }

                if end > start {
                    let idx = sql[start..end].parse::<usize>().unwrap_or(0);
                    if let Some(value) = replacement(idx) {
                        result.push_str(&value);
                    } else {
                        result.push_str(&sql[i..end]);
                    }
                    i = end;
                } else {
                    result.push('$');
                    i += 1;
                }
            }
            _ => {
                let ch = sql[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        }
    }

    result
}

fn quote_sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
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
        // Fallback: treat unknown param types as strings. Extended-query parameters
        // are raw values, so every string literal we inline must be SQL-escaped.
        _ => portal
            .parameter::<String>(idx, &ptype)
            .map(|v| {
                v.map(|s| quote_sql_string(&s))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_numeric_params_uses_original_sql_only() {
        let sql = r#"INSERT INTO logs(a, b, c) VALUES ($1, $2, $3)"#;
        let values = [
            quote_sql_string("ok"),
            quote_sql_string("lambda$handleRequestWithCache'redisson-netty-3-14':272 and $3"),
            quote_sql_string("reactor-http-epoll-9"),
        ];

        let rewritten = replace_numeric_params(sql, |idx| values.get(idx - 1).cloned());

        assert!(
            rewritten.contains("lambda$handleRequestWithCache''redisson-netty-3-14'':272 and $3"),
            "rewritten: {rewritten}"
        );
        assert!(
            rewritten.contains("'reactor-http-epoll-9'"),
            "rewritten: {rewritten}"
        );
    }

    #[test]
    fn replace_numeric_params_ignores_placeholders_inside_sql_strings() {
        let sql = "SELECT '$1' AS literal, $1 AS param, '中文$2' AS text";
        let rewritten = replace_numeric_params(sql, |idx| Some(format!("param_{idx}")));

        assert_eq!(
            rewritten,
            "SELECT '$1' AS literal, param_1 AS param, '中文$2' AS text"
        );
    }

    #[test]
    fn quote_sql_string_escapes_single_quotes() {
        assert_eq!(
            quote_sql_string("GlobalAuthFilter.java:lambda$handleRequestWithCache'redisson'"),
            "'GlobalAuthFilter.java:lambda$handleRequestWithCache''redisson'''"
        );
    }
}
