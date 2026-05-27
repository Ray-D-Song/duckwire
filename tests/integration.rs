mod integration {
    use std::sync::{Arc, Mutex};

    use arrow::datatypes::DataType;
    use duckdb::types::ValueRef;
    use pgwire::api::Type;
    use pgwire::api::results::DataRowEncoder;

    use duckwire::backend::catalog::init_pg_compat;
    use duckwire::backend::result::DuckDBQueryResult;
    use duckwire::backend::session::DuckDBSession;
    use duckwire::rewrite::Transpiler;
    use duckwire::types::mapping::{
        arrow_type_to_pg, build_schema_from_columns, encode_duckdb_owned_value,
        encode_duckdb_value,
    };

    fn make_session() -> DuckDBSession {
        let conn = Arc::new(Mutex::new(duckdb::Connection::open_in_memory().unwrap()));
        init_pg_compat(&conn);
        let transpiler = Arc::new(Transpiler::new());
        DuckDBSession::new(conn, transpiler)
    }

    #[test]
    fn test_type_mapping_arrow_to_pg() {
        assert_eq!(arrow_type_to_pg(&DataType::Boolean).unwrap(), Type::BOOL);
        assert_eq!(arrow_type_to_pg(&DataType::Int8).unwrap(), Type::INT2);
        assert_eq!(arrow_type_to_pg(&DataType::Int16).unwrap(), Type::INT2);
        assert_eq!(arrow_type_to_pg(&DataType::Int32).unwrap(), Type::INT4);
        assert_eq!(arrow_type_to_pg(&DataType::Int64).unwrap(), Type::INT8);
        assert_eq!(arrow_type_to_pg(&DataType::UInt8).unwrap(), Type::INT2);
        assert_eq!(arrow_type_to_pg(&DataType::UInt16).unwrap(), Type::INT4);
        assert_eq!(arrow_type_to_pg(&DataType::UInt32).unwrap(), Type::INT8);
        assert_eq!(arrow_type_to_pg(&DataType::UInt64).unwrap(), Type::INT8);
        assert_eq!(arrow_type_to_pg(&DataType::Float32).unwrap(), Type::FLOAT4);
        assert_eq!(arrow_type_to_pg(&DataType::Float64).unwrap(), Type::FLOAT8);
        assert_eq!(arrow_type_to_pg(&DataType::Utf8).unwrap(), Type::TEXT);
        assert_eq!(arrow_type_to_pg(&DataType::LargeUtf8).unwrap(), Type::TEXT);
        assert_eq!(arrow_type_to_pg(&DataType::Binary).unwrap(), Type::BYTEA);
        assert_eq!(arrow_type_to_pg(&DataType::Date32).unwrap(), Type::DATE);
        assert_eq!(
            arrow_type_to_pg(&DataType::Timestamp(
                arrow::datatypes::TimeUnit::Microsecond,
                None
            ))
            .unwrap(),
            Type::TIMESTAMP
        );
        assert_eq!(arrow_type_to_pg(&DataType::Null).unwrap(), Type::UNKNOWN);
    }

    #[test]
    fn test_transpiler_basic() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT 1").unwrap();
        assert!(result.contains("SELECT"), "Expected SELECT in: {result}");

        let result = t.rewrite("SELECT COALESCE(a, b) FROM t").unwrap();
        assert!(result.contains("COALESCE"));
    }

    #[test]
    fn test_transpiler_pg_specific_rewrites() {
        let t = Transpiler::new();
        let result = t.rewrite("SET search_path TO public").unwrap();
        assert!(result.is_empty() || result.trim().is_empty());

        let result = t.rewrite("SHOW TRANSACTION ISOLATION LEVEL").unwrap();
        assert!(result.contains("read committed"));

        let result = t
            .rewrite("SELECT * FROM t WHERE id = 'foo'::regclass")
            .unwrap();
        assert!(!result.contains("::regclass"));

        let result = t
            .rewrite("SELECT * FROM t WHERE id = 'bar'::regtype")
            .unwrap();
        assert!(!result.contains("::regtype"));
    }

    #[test]
    fn test_session_create_insert_select() {
        let mut session = make_session();

        let result = session
            .execute("CREATE TABLE test_int (id INTEGER, name VARCHAR)")
            .unwrap();
        match result {
            DuckDBQueryResult::Affected(n) => assert_eq!(n, 0),
            other => panic!("Expected Affected, got {other:?}"),
        }

        session
            .execute("INSERT INTO test_int VALUES (1, 'Alice'), (2, 'Bob')")
            .unwrap();

        let result = session.execute("SELECT id, name FROM test_int").unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(data.len(), 2);
                assert_eq!(arrow_type_to_pg(&columns[0].1).unwrap(), Type::INT4);
                assert_eq!(arrow_type_to_pg(&columns[1].1).unwrap(), Type::TEXT);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_session_multiple_types() {
        let mut session = make_session();

        session
            .execute("CREATE TABLE types_test (b BOOLEAN, i INTEGER, bi BIGINT, fl FLOAT, db DOUBLE, tx TEXT)")
            .unwrap();

        session
            .execute("INSERT INTO types_test VALUES (true, 42, 9999999999, 3.14, 2.718, 'hello')")
            .unwrap();

        let result = session.execute("SELECT * FROM types_test").unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 6);
                assert_eq!(arrow_type_to_pg(&columns[0].1).unwrap(), Type::BOOL);
                assert_eq!(arrow_type_to_pg(&columns[1].1).unwrap(), Type::INT4);
                assert_eq!(arrow_type_to_pg(&columns[2].1).unwrap(), Type::INT8);
                assert_eq!(arrow_type_to_pg(&columns[3].1).unwrap(), Type::FLOAT4);
                assert_eq!(arrow_type_to_pg(&columns[4].1).unwrap(), Type::FLOAT8);
                assert_eq!(arrow_type_to_pg(&columns[5].1).unwrap(), Type::TEXT);
                assert_eq!(data.len(), 1);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_session_empty_sql() {
        let mut session = make_session();
        let result = session.execute("SET search_path TO public").unwrap();
        match result {
            DuckDBQueryResult::Empty => {}
            other => panic!("Expected Empty for SET, got {other:?}"),
        }
    }

    #[test]
    fn test_session_transaction() {
        let mut session = make_session();

        let result = session.execute("BEGIN").unwrap();
        match result {
            DuckDBQueryResult::Status(s) => assert_eq!(s, "BEGIN"),
            other => panic!("Expected Status(BEGIN), got {other:?}"),
        }

        let result = session.execute("COMMIT").unwrap();
        match result {
            DuckDBQueryResult::Status(s) => assert_eq!(s, "COMMIT"),
            other => panic!("Expected Status(COMMIT), got {other:?}"),
        }
    }

    #[test]
    fn test_current_schemas_list_result_does_not_panic() {
        let mut session = make_session();

        let result = session
            .execute("select current_database() as a, current_schemas(false) as b")
            .unwrap();

        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(data.len(), 1);
                let schema = build_schema_from_columns(&columns).unwrap();
                let mut encoder = DataRowEncoder::new(schema);
                for value in &data[0] {
                    encode_duckdb_owned_value(&mut encoder, value).unwrap();
                }
                let _row = encoder.take_row();
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_build_schema_and_encode() {
        let columns = vec![
            ("id".to_string(), DataType::Int32),
            ("name".to_string(), DataType::Utf8),
            ("score".to_string(), DataType::Float64),
        ];

        let schema = build_schema_from_columns(&columns).unwrap();
        assert_eq!(schema.len(), 3);
        assert_eq!(schema[0].name(), "id");
        assert_eq!(schema[0].datatype(), &Type::INT4);
        assert_eq!(schema[1].name(), "name");
        assert_eq!(schema[1].datatype(), &Type::TEXT);
        assert_eq!(schema[2].name(), "score");
        assert_eq!(schema[2].datatype(), &Type::FLOAT8);

        let mut encoder = DataRowEncoder::new(schema.clone());
        encode_duckdb_value(&mut encoder, ValueRef::Int(42)).unwrap();
        encode_duckdb_value(&mut encoder, ValueRef::Text(b"Alice".as_slice())).unwrap();
        encode_duckdb_value(&mut encoder, ValueRef::Double(3.14)).unwrap();
        let _row = encoder.take_row();

        let mut encoder = DataRowEncoder::new(schema.clone());
        encode_duckdb_value(&mut encoder, ValueRef::Null).unwrap();
        encode_duckdb_value(&mut encoder, ValueRef::Boolean(true)).unwrap();
        encode_duckdb_value(&mut encoder, ValueRef::BigInt(9999999999i64)).unwrap();
        let _row = encoder.take_row();
    }

    #[test]
    fn test_full_pipeline() {
        let mut session = make_session();

        session
            .execute("CREATE TABLE users (id INTEGER, name VARCHAR, age INTEGER)")
            .unwrap();

        session
            .execute(
                "INSERT INTO users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Charlie', 35)",
            )
            .unwrap();

        let result = session
            .execute("SELECT name, age FROM users WHERE age > 26")
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(data.len(), 2);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }

        let result = session
            .execute("SELECT COUNT(*) as total FROM users")
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { data, .. } => {
                assert_eq!(data.len(), 1);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_pg_database() {
        let mut session = make_session();
        let result = session.execute("SELECT datname FROM pg_database").unwrap();
        match result {
            DuckDBQueryResult::Rows { data, .. } => {
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_pg_database_join_tablespace() {
        let mut session = make_session();
        let sql = "SELECT d.datname, t.spcname FROM pg_database AS d LEFT JOIN pg_tablespace AS t ON d.dattablespace = t.oid";
        let result = session.execute(sql).unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_pg_settings() {
        let mut session = make_session();
        let result = session
            .execute("SELECT name, setting FROM pg_settings")
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { data, .. } => {
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_pg_roles() {
        let mut session = make_session();
        let result = session.execute("SELECT rolname FROM pg_roles").unwrap();
        match result {
            DuckDBQueryResult::Rows { data, .. } => {
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }
}
