use std::sync::{Arc, Mutex};

use duckdb::Connection;
use tracing::{debug, error, info};

use crate::backend::result::DuckDBQueryResult;
use crate::rewrite::Transpiler;

pub struct DuckDBSession {
    conn: Arc<Mutex<Connection>>,
    transpiler: Arc<Transpiler>,
    in_transaction: bool,
}

impl DuckDBSession {
    pub fn new(conn: Arc<Mutex<Connection>>, transpiler: Arc<Transpiler>) -> Self {
        Self {
            conn,
            transpiler,
            in_transaction: false,
        }
    }

    pub fn execute(&mut self, sql: &str) -> Result<DuckDBQueryResult, duckdb::Error> {
        let rewritten = self
            .transpiler
            .rewrite(sql)
            .unwrap_or_else(|_| sql.to_string());

        if rewritten.trim().is_empty() {
            debug!(original = sql, "query suppressed (empty after rewrite)");
            return Ok(DuckDBQueryResult::Empty);
        }

        let upper = rewritten.trim().to_uppercase();
        debug!(original = sql, rewritten = %rewritten, "executing query");

        // Manual transaction tracking: DuckDB's auto-commit mode makes it hard to
        // intercept COMMIT/ROLLBACK after the fact, so we track state with a simple bool.
        // NOTE: nested transactions and savepoints are not supported.
        if self.in_transaction {
            if upper.starts_with("COMMIT") {
                self.in_transaction = false;
                let conn = self.conn.lock().unwrap();
                conn.execute_batch("COMMIT")?;
                info!("COMMIT");
                return Ok(DuckDBQueryResult::Status("COMMIT".to_string()));
            } else if upper.starts_with("ROLLBACK") {
                self.in_transaction = false;
                let conn = self.conn.lock().unwrap();
                conn.execute_batch("ROLLBACK")?;
                info!("ROLLBACK");
                return Ok(DuckDBQueryResult::Status("ROLLBACK".to_string()));
            }
        }

        if upper.starts_with("BEGIN") || upper.starts_with("START TRANSACTION") {
            self.in_transaction = true;
            let conn = self.conn.lock().unwrap();
            conn.execute_batch("BEGIN")?;
            info!("BEGIN");
            return Ok(DuckDBQueryResult::Status("BEGIN".to_string()));
        }

        let conn = self.conn.lock().unwrap();

        if upper.starts_with("SELECT") || upper.starts_with("WITH") || upper.starts_with("SHOW") {
            let mut meta_stmt = match conn.prepare(&rewritten) {
                Ok(s) => s,
                Err(e) => {
                    error!(rewritten = %rewritten, "prepare failed");
                    return Err(e);
                }
            };
            // Workaround: DuckDB's prepare() may not populate column metadata until the
            // statement is actually executed. Calling execute([]) forces column_count(),
            // column_names(), and column_type() to return correct values.
            meta_stmt.execute([])?;
            let column_count = meta_stmt.column_count();
            if column_count == 0 {
                info!(rows = 0, "query completed");
                return Ok(DuckDBQueryResult::Status("SELECT 0".to_string()));
            }
            let column_names = meta_stmt.column_names();
            let columns: Vec<(String, arrow::datatypes::DataType)> = (0..column_count)
                .map(|i| {
                    let name = column_names[i].clone();
                    let dt = meta_stmt.column_type(i);
                    (name, dt)
                })
                .collect();
            drop(meta_stmt);

            let mut stmt = conn.prepare(&rewritten)?;
            let mut rows = stmt.query([])?;
            let mut data = Vec::new();
            while let Some(row) = rows.next()? {
                let mut row_data = Vec::with_capacity(columns.len());
                for i in 0..columns.len() {
                    let value_ref = row.get_ref_unwrap(i);
                    row_data.push(value_ref.to_owned());
                }
                data.push(row_data);
            }

            info!(rows = data.len(), cols = columns.len(), "query completed");
            Ok(DuckDBQueryResult::Rows { columns, data })
        } else {
            let affected = conn.execute(&rewritten, [])?;
            info!(affected, "query completed");
            Ok(DuckDBQueryResult::Affected(affected as u64))
        }
    }
}
