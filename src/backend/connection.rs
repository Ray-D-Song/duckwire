use std::sync::{Arc, Mutex};

use duckdb::Connection;

use crate::backend::catalog::init_pg_compat;
use crate::backend::session::DuckDBSession;
use crate::rewrite::Transpiler;

enum DuckDBConnectionMode {
    File(String),
    Shared(Arc<Mutex<Connection>>),
}

pub struct DuckDBConnection {
    mode: DuckDBConnectionMode,
    pub transpiler: Arc<Transpiler>,
}

impl DuckDBConnection {
    pub fn open(path: Option<&str>) -> Result<Self, duckdb::Error> {
        let conn = match path {
            Some(p) => Connection::open(p)?,
            None => Connection::open_in_memory()?,
        };
        let arc_conn = Arc::new(Mutex::new(conn));
        init_pg_compat(&arc_conn);
        let mode = match path {
            Some(p) => DuckDBConnectionMode::File(p.to_string()),
            None => DuckDBConnectionMode::Shared(arc_conn),
        };
        Ok(Self {
            mode,
            transpiler: Arc::new(Transpiler::new()),
        })
    }

    pub fn create_session(&self) -> Result<DuckDBSession, duckdb::Error> {
        let conn = match &self.mode {
            DuckDBConnectionMode::File(path) => Arc::new(Mutex::new(Connection::open(path)?)),
            DuckDBConnectionMode::Shared(conn) => conn.clone(),
        };
        Ok(DuckDBSession::new(conn, self.transpiler.clone()))
    }
}
