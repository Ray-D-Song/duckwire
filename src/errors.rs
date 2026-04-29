use pgwire::error::ErrorInfo;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DuckWireError {
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] duckdb::Error),

    #[error("SQL transpilation error: {0}")]
    Transpile(#[from] polyglot_sql::Error),

    #[error("Unsupported type conversion: {0}")]
    TypeConversion(String),

    #[error("Protocol error: {0}")]
    Protocol(String),
}

impl From<DuckWireError> for pgwire::error::PgWireError {
    fn from(e: DuckWireError) -> Self {
        pgwire::error::PgWireError::UserError(Box::new(ErrorInfo::new(
            "ERROR".into(),
            "HY000".into(),
            e.to_string(),
        )))
    }
}