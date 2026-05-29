mod integration {
    use std::sync::{Arc, Mutex};

    use arrow::datatypes::DataType;
    use duckdb::types::ValueRef;
    use pgwire::api::Type;
    use pgwire::api::portal::Format;
    use pgwire::api::results::DataRowEncoder;
    use pgwire::api::results::FieldFormat;

    use duckwire::backend::catalog::init_pg_compat;
    use duckwire::backend::result::DuckDBQueryResult;
    use duckwire::backend::session::DuckDBSession;
    use duckwire::rewrite::Transpiler;
    use duckwire::types::mapping::{
        arrow_type_to_pg, build_schema_from_columns, column_type_to_pg, encode_duckdb_owned_value,
        encode_duckdb_value, requested_format_for_type,
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
    fn test_catalog_object_kind_columns_use_pg_char_like_type() {
        assert_eq!(
            column_type_to_pg("object_kind", &DataType::Utf8).unwrap(),
            Type::BPCHAR
        );
        assert_eq!(
            column_type_to_pg("relkind", &DataType::Utf8).unwrap(),
            Type::BPCHAR
        );
        assert_eq!(
            column_type_to_pg("table_name", &DataType::Utf8).unwrap(),
            Type::TEXT
        );
    }

    #[test]
    fn test_result_format_honors_client_request() {
        assert_eq!(
            requested_format_for_type(Some(&Format::UnifiedBinary), 0, &Type::INT8),
            FieldFormat::Binary
        );
        assert_eq!(
            requested_format_for_type(Some(&Format::UnifiedBinary), 0, &Type::NUMERIC),
            FieldFormat::Binary
        );
        let individual = Format::Individual(vec![0, 1]);
        assert_eq!(
            requested_format_for_type(Some(&individual), 0, &Type::INT8),
            FieldFormat::Text
        );
        assert_eq!(
            requested_format_for_type(Some(&individual), 1, &Type::TIMESTAMP),
            FieldFormat::Binary
        );
        assert_eq!(
            requested_format_for_type(Some(&Format::UnifiedText), 0, &Type::INT8),
            FieldFormat::Text
        );
        assert_eq!(
            requested_format_for_type(None, 0, &Type::INT8),
            FieldFormat::Text
        );
    }

    #[test]
    fn test_timestamp_can_encode_as_binary() {
        let schema = std::sync::Arc::new(vec![pgwire::api::results::FieldInfo::new(
            "triggerTime".into(),
            None,
            None,
            Type::TIMESTAMP,
            FieldFormat::Binary,
        )]);
        let mut encoder = DataRowEncoder::new(schema);

        encode_duckdb_value(
            &mut encoder,
            ValueRef::Timestamp(duckdb::types::TimeUnit::Microsecond, 1_779_971_256_000_000),
        )
        .unwrap();

        let _row = encoder.take_row();
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
    fn test_current_schema_uses_postgres_public_view() {
        let mut session = make_session();

        let result = session
            .execute(
                "select current_database() as db, current_schema() as s, current_schemas(false) as schemas",
            )
            .unwrap();

        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 3);
                assert_eq!(data.len(), 1);
                assert_eq!(format!("{:?}", data[0][0]), "Text(\"postgres\")");
                assert_eq!(format!("{:?}", data[0][1]), "Text(\"public\")");
                assert_eq!(format!("{:?}", data[0][2]), "List([Text(\"public\")])");
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
    fn test_catalog_pg_database_datagrip_privilege_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "SELECT d.datname, d.datcollate, d.datconnlimit, d.description, \
                 pg_catalog.has_database_privilege(d.datname, 'CONNECT') AS can_connect \
                 FROM pg_database d",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 5);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_txid_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select case
                   when pg_catalog.pg_is_in_recovery()
                     then null
                   else
                     (pg_catalog.txid_current() % 4294967296)::varchar::bigint
                 end as current_txid",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 1);
                assert_eq!(data.len(), 1);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_database_order_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select N.oid::bigint as id,
                        datname as name,
                        D.description,
                        datistemplate as is_template,
                        datallowconn as allow_connections,
                        pg_catalog.pg_get_userbyid(N.datdba) as \"owner\"
                 from pg_catalog.pg_database N
                   left join pg_catalog.pg_shdescription D on N.oid = D.objoid
                 order by case when datname = pg_catalog.current_database() then -1::bigint else N.oid::bigint end",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 6);
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
    fn test_catalog_pg_locks_datagrip_transactionid_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "SELECT CAST(L.transactionid AS BIGINT) AS transaction_id \
                 FROM pg_compat.pg_locks AS L \
                 WHERE L.transactionid IS NOT NULL \
                 ORDER BY pg_compat.age(L.transactionid)",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 1);
                assert_eq!(data.len(), 0);
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

    #[test]
    fn test_catalog_datagrip_timezone_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select name, is_dst from pg_catalog.pg_timezone_names
                 union distinct
                 select abbrev as name, is_dst from pg_catalog.pg_timezone_abbrevs",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_roles_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select R.oid::bigint as role_id, rolname as role_name,
                   rolsuper is_super, rolinherit is_inherit,
                   rolcreaterole can_createrole, rolcreatedb can_createdb,
                   rolcanlogin can_login, rolreplication /* false */ is_replication,
                   rolconnlimit conn_limit, rolvaliduntil valid_until,
                   rolbypassrls /* false */ bypass_rls, rolconfig config,
                   D.description
                 from pg_catalog.pg_roles R
                   left join pg_catalog.pg_shdescription D on D.objoid = R.oid",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 13);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_namespace_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select N.oid::bigint as id,
                        N.xmin as state_number,
                        nspname as name,
                        D.description,
                        pg_catalog.pg_get_userbyid(N.nspowner) as \"owner\"
                 from pg_catalog.pg_namespace N
                   left join pg_catalog.pg_description D on D.objoid = N.oid",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 5);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_deep_introspection_stubs() {
        let mut session = make_session();
        let queries = [
            "SELECT CASE WHEN t.evtenabled = 'D' THEN 1 ELSE 0 END AS is_disabled FROM pg_catalog.pg_event_trigger AS t",
            "SELECT fdw.oid AS id, fdw.fdwname AS name, fdw.fdwoptions AS options, pg_catalog.pg_get_userbyid(fdw.fdwowner) AS \"owner\" FROM pg_catalog.pg_foreign_data_wrapper AS fdw",
            "SELECT srv.oid AS id, srv.srvfdw AS fdw_id, srv.xmin AS state_number, srv.srvname AS name FROM pg_catalog.pg_foreign_server AS srv",
            "SELECT srvid AS server_id, usename AS \"user\", umoptions AS options FROM pg_catalog.pg_user_mappings ORDER BY server_id",
            "SELECT A.oid AS access_method_id, A.xmin AS state_number, A.amname AS access_method_name FROM pg_catalog.pg_am AS A",
            "SELECT l.oid AS id, l.xmin AS state_number, lanname AS name, lanpltrusted AS is_trusted FROM pg_catalog.pg_language AS l",
            "SELECT T.oid AS object_id, T.fdwacl AS acl FROM pg_catalog.pg_foreign_data_wrapper AS T",
            "SELECT C.castsource, C.casttarget, C.castfunc, C.castcontext, C.castmethod FROM pg_catalog.pg_cast AS C",
            "SELECT sq.seqrelid, sq.seqstart, sq.seqincrement, sq.seqcycle AS cycle_option FROM pg_catalog.pg_sequence AS sq",
            "SELECT T.oid AS type_id, T.xmin AS type_state_number, T.typname AS type_name, T.typtype FROM pg_catalog.pg_type AS T",
            "SELECT T.oid, T.xmin AS table_state_number, T.relname FROM pg_catalog.pg_class AS T",
            "WITH schema_procs AS (SELECT prorettype, proargtypes, proallargtypes FROM pg_catalog.pg_proc WHERE pronamespace = 2200) SELECT * FROM schema_procs",
            "SELECT proname AS r_name, prolang AS lang_oid, oid AS r_id, xmin AS r_state_number, proargnames AS arg_names FROM pg_catalog.pg_proc",
            "SELECT A.aggfnoid, P.proname FROM pg_catalog.pg_aggregate AS A JOIN pg_catalog.pg_proc AS P ON P.oid = A.aggfnoid",
            "SELECT O.oid AS op_id, O.xmin AS state_number, oprname AS op_name, oprkind AS op_kind FROM pg_catalog.pg_operator AS O",
            "SELECT oid AS id, xmin AS state_number, collname AS name, collcollate AS lc_collate FROM pg_catalog.pg_collation",
            "SELECT O.oid, O.opfname, O.opfmethod FROM pg_catalog.pg_opfamily AS O",
            "SELECT pg_amop.oid FROM pg_catalog.pg_amop JOIN pg_catalog.pg_opfamily ON pg_opfamily.oid = pg_amop.amopfamily",
            "SELECT pg_amproc.oid FROM pg_catalog.pg_amproc JOIN pg_catalog.pg_opfamily ON pg_opfamily.oid = pg_amproc.amprocfamily",
            "SELECT ind_stor.oid FROM pg_catalog.pg_index AS ind_stor LEFT JOIN pg_catalog.pg_opclass ON pg_opclass.oid = ANY(indclass)",
            "SELECT RU.oid FROM pg_catalog.pg_rewrite AS RU, pg_catalog.pg_class AS RC WHERE RC.oid = RU.ev_class",
            "SELECT P.oid FROM pg_catalog.pg_policy AS P JOIN pg_catalog.pg_class AS C ON C.oid = P.polrelid",
            "SELECT TG.oid FROM pg_catalog.pg_trigger AS TG, pg_catalog.pg_class AS TC WHERE TC.oid = TG.tgrelid",
            "SELECT oid AS id, pg_catalog.pg_get_function_arguments(oid) AS arguments_def FROM pg_catalog.pg_proc",
            "SELECT A.oid AS access_method_id, A.xmin AS state_number, A.amname AS access_method_name, CAST(A.amhandler AS OID) AS handler_id, pg_catalog.quote_ident(N.nspname) || '.' || pg_catalog.quote_ident(A.amname) AS qualified_name FROM pg_catalog.pg_am AS A LEFT JOIN pg_catalog.pg_namespace AS N ON N.oid = 11",
            "SELECT E.oid AS id, E.extversion AS version, ARRAY(SELECT unnest FROM UNNEST(available_versions) WHERE unnest > extversion) AS available_versions FROM pg_catalog.pg_extension AS E LEFT JOIN pg_catalog.pg_available_extension_versions() AS V ON V.name = E.extname",
            "SELECT C.oid, C.xmin AS state_number, C.castsource AS castsource_id, pg_catalog.quote_ident(SN.nspname) || '.' || pg_catalog.quote_ident(C.castsource::regtype::text) AS source_name FROM pg_catalog.pg_cast AS C LEFT JOIN pg_catalog.pg_namespace AS SN ON SN.oid = 11",
            "SELECT provariadic AS arg_variadic_id, prorettype AS ret_type_id FROM pg_catalog.pg_proc AS P",
            "SELECT CASE WHEN A.aggsortop = 0 THEN NULL ELSE CAST(CAST(A.aggsortop AS REGOPER) AS TEXT) END AS sort_operator_name FROM pg_catalog.pg_aggregate AS A",
            "SELECT CAST(CAST(oprcom AS REGOPER) AS TEXT) AS com_name, CAST(CAST(oprnegate AS REGOPER) AS TEXT) AS neg_name FROM pg_catalog.pg_operator AS O",
            "SELECT CAST(CAST(O.amopopr AS REGOPERATOR) AS TEXT) AS op_sig FROM pg_catalog.pg_amop AS O",
            "SELECT P.amprocnum AS num, CAST(P.amproc AS OID) AS proc_id, CAST(P.amprocedure AS TEXT) AS proc_sig FROM pg_catalog.pg_amproc AS P",
            "SELECT ind_head.indnullsnotdistinct /* false */ AS nulls_not_distinct FROM pg_catalog.pg_index AS ind_head",
            "SELECT CAST(C.oid AS BIGINT) AS con_id, CAST(CAST(C.xmin AS TEXT) AS BIGINT) AS con_state_id, conname AS con_name FROM pg_catalog.pg_constraint AS C",
            "SELECT oid AS id, pg_catalog.pg_get_function_sqlbody(oid) AS sqlbody_def FROM pg_catalog.pg_proc",
            "SELECT proname AS name, procost AS cost, pg_catalog.pg_get_userbyid(proowner) AS \"owner\", prorows AS \"rows\", proleakproof AS is_leakproof, proparallel AS concurrency_kind FROM pg_catalog.pg_proc AS P",
            "SELECT O.amopstrategy AS strategy, O.amopopr AS op_id, CAST(CAST(O.amopopr::regoperator AS TEXT) AS TEXT) AS op_sig, O.amopsortfamily AS sort_family FROM pg_catalog.pg_amop AS O",
            "SELECT conrelid AS table_id, contype AS con_kind, conkey AS con_columns, conindid AS index_id, confrelid AS ref_table_id, confkey AS ref_columns, condeferrable AS is_deferrable FROM pg_catalog.pg_constraint AS C",
            "SELECT C.oid, pg_catalog.pg_get_expr(C.relpartbound, C.oid) AS partition_expr FROM pg_catalog.pg_class AS C WHERE C.relispartition",
            "SELECT I.inhrelid, I.inhparent FROM pg_catalog.pg_inherits AS I ORDER BY pg_catalog.age(I.inhparent)",
            "SELECT conname AS name, pg_catalog.pg_get_expr(NULL, conrelid) AS con_expression /* consrc */, confkey AS ref_columns, CAST(conexclop AS INT[]) AS excl_operators, ARRAY(SELECT CAST(CAST(unnest AS REGOPER) AS TEXT) FROM UNNEST(conexclop)) AS excl_operator_names FROM pg_catalog.pg_constraint AS C",
            "SELECT \"pg_catalog\".pg_get_expr(C.relpartbound, C.oid) AS partition_expr FROM pg_catalog.pg_class AS C WHERE C.relispartition",
            "SELECT pg_compat . pg_get_expr(C.relpartbound, C.oid) AS partition_expr FROM pg_catalog.pg_class AS C WHERE C.relispartition",
            "SELECT pg_compat . format_type(T.oid, NULL) AS type_name FROM pg_catalog.pg_type AS T",
            "SELECT pg_catalog . age(I.inhparent) AS parent_age FROM pg_catalog.pg_inherits AS I",
            "SELECT pg_compat.pg_partition_tree(C.oid) AS partition_tree FROM pg_catalog.pg_class AS C WHERE C.relispartition",
            "SELECT \"pg_compat\" . \"pg_describe_object\"(T.oid, 0, 0) AS object_name FROM pg_catalog.pg_type AS T",
            "SELECT contype AS object_kind FROM pg_catalog.pg_constraint",
            "SELECT aggkind AS object_kind, aggfinalmodify AS final_modify FROM pg_catalog.pg_aggregate",
            "SELECT ev_type AS object_kind FROM pg_catalog.pg_rewrite",
            "SELECT polcmd AS object_kind FROM pg_catalog.pg_policy",
            "SELECT C.attrelid AS table_id, C.attnum AS column_position, C.attname AS column_name, C.xmin AS column_state_number, C.atttypmod AS type_mod FROM pg_catalog.pg_attribute AS C",
            "SELECT RC.oid FROM pg_catalog.pg_rewrite AS R JOIN pg_catalog.pg_class AS RC ON RC.oid = R.ev_class WHERE R.rulename <> CAST('_RETURN' AS NAME) ORDER BY CAST(R.ev_class AS BIGINT), ev_type",
            "SELECT A.table_id FROM (SELECT 0::BIGINT AS table_id) AS A JOIN pg_catalog.pg_rewrite AS R ON A.table_id = R.ev_class WHERE R.rulename <> CAST('_RETURN' AS NAME)",
            "SELECT C.oid FROM pg_catalog.pg_class AS C CROSS JOIN pg_catalog.pg_indexam_has_property(C.relam, 'can_order') AS amcanorder",
            "SELECT C.attrelid AS table_id, C.attnum AS column_position, C.attname AS column_name, C.xmin AS column_state_number, C.atttypmod AS type_mod, NOT C.attislocal AS column_is_inherited, C.attfdwoptions AS options FROM pg_catalog.pg_attribute AS C",
            "SELECT C.attfdwoptions AS options, C.attisdropped AS column_is_dropped, C.attidentity AS identity_kind, C.attgenerated AS generated_kind FROM pg_catalog.pg_attribute AS C",
            "SELECT ind_head.indexrelid AS index_id, k AS col_idx, k <= indnkeyatts AS in_key FROM pg_catalog.pg_index AS ind_head, UNNEST(indkey) AS k",
            "SELECT relname AS object_name, relkind AS object_kind FROM pg_catalog.pg_class WHERE relname = 'log_info_interface_log'",
        ];

        for query in queries {
            session.execute(query).unwrap_or_else(|e| {
                panic!("Expected DataGrip deep introspection query to execute: {query}\n{e}")
            });
        }
    }

    #[test]
    fn test_datagrip_public_table_browser_query() {
        let mut session = make_session();
        session
            .execute("CREATE TABLE xxl_job_log(id BIGINT, app_name VARCHAR)")
            .unwrap();
        session
            .execute("INSERT INTO xxl_job_log VALUES (1, 'auth')")
            .unwrap();

        let result = session
            .execute("SELECT t.*, CTID FROM public.xxl_job_log AS t LIMIT 501")
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 3);
                assert_eq!(data.len(), 1);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_datagrip_update_with_ctid_predicate() {
        let mut session = make_session();
        session
            .execute("CREATE TABLE xxl_job_log(id BIGINT, alarm_status INTEGER)")
            .unwrap();
        session
            .execute("INSERT INTO xxl_job_log VALUES (1, 0)")
            .unwrap();

        let result = session
            .execute("UPDATE xxl_job_log SET alarm_status = 1 WHERE alarm_status = 0 AND CTID = 0")
            .unwrap();
        match result {
            DuckDBQueryResult::Affected(n) => assert_eq!(n, 1),
            other => panic!("Expected Affected, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_user_table_object_kind() {
        let mut session = make_session();
        session
            .execute("CREATE TABLE log_info_interface_log(id BIGINT)")
            .unwrap();

        let result = session
            .execute(
                "SELECT relname AS object_name, relkind AS object_kind
                 FROM pg_catalog.pg_class
                 WHERE relname = 'log_info_interface_log'",
            )
            .unwrap();

        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(data.len(), 1);
                assert_eq!(
                    data[0][0],
                    duckdb::types::Value::Text("log_info_interface_log".into())
                );
                assert_eq!(data[0][1], duckdb::types::Value::Text("r".into()));
            }
            other => panic!("Expected Rows, got {other:?}"),
        }

        let result = session
            .execute(
                "select T.oid as oid,
                        relnamespace as schemaId,
                        pg_catalog.translate(relkind, 'rmvpfS', 'rmvrfS') as kind,
                        relname as name
                 from pg_catalog.pg_class T
                 where relnamespace in ( 2200 )
                   and relkind in ('r', 'm', 'v', 'p', 'f', 'S')
                 order by schemaId",
            )
            .unwrap();

        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 4);
                let row = data
                    .iter()
                    .find(|row| {
                        row.get(3)
                            == Some(&duckdb::types::Value::Text("log_info_interface_log".into()))
                    })
                    .expect("expected DataGrip object list to include user table");
                assert_eq!(row[2], duckdb::types::Value::Text("r".into()));
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_array_select_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "SELECT ARRAY(SELECT unnest FROM UNNEST(available_versions) WHERE unnest <> extversion) AS other_versions FROM pg_catalog.pg_extension",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 1);
                assert_eq!(data.len(), 0);
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }

    #[test]
    fn test_catalog_datagrip_tablespace_query() {
        let mut session = make_session();
        let result = session
            .execute(
                "select T.oid::bigint as id, T.spcname as name,
                        T.xmin as state_number, pg_catalog.pg_get_userbyid(T.spcowner) as owner,
                        pg_catalog.pg_tablespace_location(T.oid) /* null */ as location,
                        T.spcoptions /* null */ as options,
                        D.description as comment
                 from pg_catalog.pg_tablespace T
                   left join pg_catalog.pg_shdescription D on D.objoid = T.oid",
            )
            .unwrap();
        match result {
            DuckDBQueryResult::Rows { columns, data } => {
                assert_eq!(columns.len(), 7);
                assert!(!data.is_empty());
            }
            other => panic!("Expected Rows, got {other:?}"),
        }
    }
}
