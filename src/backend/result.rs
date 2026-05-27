use arrow::datatypes::DataType;

#[derive(Debug)]
pub enum DuckDBQueryResult {
    Rows {
        columns: Vec<(String, DataType)>,
        data: Vec<Vec<duckdb::types::Value>>,
    },
    Affected(u64),
    Status(String),
    Empty,
}
