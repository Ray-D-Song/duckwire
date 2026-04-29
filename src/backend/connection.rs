use std::sync::{Arc, Mutex};

use duckdb::Connection;

use crate::backend::catalog::init_pg_compat;
use crate::backend::session::DuckDBSession;
use crate::rewrite::Transpiler;

pub struct DuckDBConnection {
    pub conn: Arc<Mutex<Connection>>,
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
        Ok(Self {
            conn: arc_conn,
            transpiler: Arc::new(Transpiler::new()),
        })
    }

    pub fn create_session(&self) -> DuckDBSession {
        DuckDBSession::new(self.conn.clone(), self.transpiler.clone())
    }
}