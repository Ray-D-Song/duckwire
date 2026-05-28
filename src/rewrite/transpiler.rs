use polyglot_sql::{DialectType, transpile};

use crate::errors::DuckWireError;

const PG_COMPAT_TABLES: &[&str] = &[
    "pg_database",
    "pg_tablespace",
    "pg_timezone_names",
    "pg_timezone_abbrevs",
    "pg_roles",
    "pg_authid",
    "pg_namespace",
    "pg_class",
    "pg_am",
    "pg_opclass",
    "pg_operator",
    "pg_attribute",
    "pg_type",
    "pg_description",
    "pg_settings",
    "pg_locks",
    "pg_stat_activity",
    "pg_inherits",
    "pg_index",
    "pg_constraint",
    "pg_proc",
    "pg_language",
    "pg_extension",
    "pg_auth_members",
    "pg_shdepend",
    "pg_depend",
    "pg_tables",
    "pg_views",
    "pg_matviews",
    "pg_user",
    "pg_shadow",
    "pg_policies",
    "pg_foreign_table",
    "pg_foreign_server",
    "pg_foreign_data_wrapper",
    "pg_user_mappings",
    "pg_collation",
    "pg_shdescription",
    "pg_stat_user_tables",
    "pg_enum",
    "pg_conversion",
    "pg_attrdef",
    "pg_event_trigger",
    "pg_cast",
    "pg_sequence",
    "pg_aggregate",
    "pg_opfamily",
    "pg_amop",
    "pg_amproc",
    "pg_rewrite",
    "pg_policy",
    "pg_trigger",
];

pub struct Transpiler;

impl Transpiler {
    pub fn new() -> Self {
        Self
    }

    // Two-stage pipeline: (1) custom Postgres→DuckDB rewrites for known patterns,
    // then (2) polyglot-sql dialect transpilation. If polyglot fails, we fall back
    // to the custom-rewritten SQL so the query still has a chance to execute.
    pub fn rewrite(&self, sql: &str) -> Result<String, DuckWireError> {
        let rewritten = self.rewrite_pg_specific(sql);
        if rewritten.trim().is_empty() {
            return Ok(String::new());
        }
        match transpile(&rewritten, DialectType::PostgreSQL, DialectType::DuckDB) {
            Ok(results) => Ok(self.rewrite_duckdb_post_transpile(&results.join("; "))),
            Err(_) => Ok(rewritten),
        }
    }

    fn rewrite_duckdb_post_transpile(&self, sql: &str) -> String {
        let result = self.rewrite_pg_functions(sql);
        let result = self.rewrite_bracketed_select_arrays(&result);
        let result = self.rewrite_unnest_column_aliases(&result);
        let result = self.rewrite_pg_pseudo_type_casts(&result);
        let result = self.rewrite_datagrip_extension_version_arrays(&result);
        let result = self.rewrite_public_schema(&result);
        self.rewrite_ctid_projection(&result)
    }

    fn rewrite_show(&self, sql: &str) -> String {
        let upper = sql.trim().to_uppercase();
        let var = upper.strip_prefix("SHOW ").unwrap().trim();
        let key = var.to_lowercase();

        let (val, alias) = match var {
            "TRANSACTION ISOLATION LEVEL" => ("'read committed'", "transaction_isolation"),
            "SEARCH_PATH" => ("'public'", "search_path"),
            "DATESTYLE" => ("'ISO'", "datestyle"),
            "TIMEZONE" => ("'UTC'", "timezone"),
            "SERVER_VERSION" => ("'16.0'", "server_version"),
            "SERVER_ENCODING" => ("'UTF8'", "server_encoding"),
            "CLIENT_ENCODING" => ("'UTF8'", "client_encoding"),
            "LC_MESSAGES" => ("'en_US.UTF-8'", "lc_messages"),
            "LC_MONETARY" => ("'en_US.UTF-8'", "lc_monetary"),
            "LC_NUMERIC" => ("'en_US.UTF-8'", "lc_numeric"),
            "LC_TIME" => ("'en_US.UTF-8'", "lc_time"),
            "STANDARD_CONFORMING_STRINGS" => ("'on'", "standard_conforming_strings"),
            "INTEGER_DATETIMES" => ("'on'", "integer_datetimes"),
            "IS_SUPERUSER" => ("'off'", "is_superuser"),
            "LOCK_TIMEOUT" => ("'0'", "lock_timeout"),
            _ => ("''", key.as_str()),
        };

        format!("SELECT {val} AS {alias}")
    }

    // Scans for bare pg_* table references and injects the 'pg_compat.' prefix.
    // Uses offset-based matching with delayed insertion to avoid disrupting positions.
    // Insertions are applied in reverse order to keep earlier indices valid.
    // NOTE: this is byte-index based; multi-byte characters in the original SQL
    // could cause index misalignment between the uppercase and original strings.
    fn rewrite_pg_tables(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        result = result.replace("\"pg_catalog\".", "pg_compat.");
        result = result.replace("pg_catalog.", "pg_compat.");

        for table in PG_COMPAT_TABLES {
            let with_scheme = format!("pg_compat.{}", table);
            if result.contains(with_scheme.as_str()) {
                continue;
            }

            let pattern_quoted = format!("\"{}\"", table);
            if result.contains(&pattern_quoted) {
                result = result.replace(&pattern_quoted, table);
                continue;
            }

            let upper = result.to_uppercase();
            let mut offset = 0;
            let table_upper = table.to_uppercase();
            let mut new_result = result.clone();
            let mut adjustments = Vec::new();

            while offset < upper.len() {
                if let Some(pos) = upper[offset..].find(table_upper.as_str()) {
                    let abs_pos = offset + pos;
                    let before_ok = abs_pos == 0
                        || !upper
                            .as_bytes()
                            .get(abs_pos - 1)
                            .map(|&b| b.is_ascii_alphanumeric() || b == b'_')
                            .unwrap_or(false);
                    let after_end = abs_pos + table.len();
                    let after_ok = after_end >= upper.len()
                        || !upper
                            .as_bytes()
                            .get(after_end)
                            .map(|&b| b.is_ascii_alphanumeric() || b == b'_')
                            .unwrap_or(false);

                    if before_ok && after_ok {
                        if !(abs_pos > 0 && result.as_bytes().get(abs_pos - 1) == Some(&b'.')) {
                            adjustments.push(abs_pos);
                        }
                    }
                    offset = abs_pos + table.len();
                } else {
                    break;
                }
            }

            // Insert prefixes in reverse to preserve positions
            for pos in adjustments.into_iter().rev() {
                new_result.insert_str(pos, "pg_compat.");
            }
            result = new_result;
        }

        result
    }

    // Pattern-matches pg_*() function calls and replaces the entire call (including
    // arguments) with a hardcoded value. Uses recursive reapplication after each
    // successful replacement to handle queries with multiple pg function calls.
    fn rewrite_pg_functions(&self, sql: &str) -> String {
        let mut result = self.rewrite_pg_builtin_prefixes(sql);

        let pg_funcs = [
            ("current_database", "'postgres'"),
            ("current_schema", "'public'"),
            ("current_schemas", "['public']"),
            ("pg_get_userbyid", "'postgres'"),
            ("pg_encoding_to_char", "'UTF8'"),
            ("pg_get_expr", "NULL"),
            ("pg_table_is_visible", "true"),
            ("has_database_privilege", "true"),
            ("has_schema_privilege", "true"),
            ("has_table_privilege", "true"),
            ("has_column_privilege", "true"),
            ("has_function_privilege", "true"),
            ("has_sequence_privilege", "true"),
            ("pg_has_role", "true"),
            ("pg_get_constraintdef", "NULL"),
            ("pg_get_indexdef", "NULL"),
            ("pg_get_viewdef", "NULL"),
            ("pg_get_ruledef", "NULL"),
            ("pg_get_triggerdef", "NULL"),
            ("pg_get_functiondef", "NULL"),
            ("pg_get_partkeydef", "NULL"),
            ("pg_get_function_result", "NULL"),
            ("pg_get_function_arguments", "''"),
            ("pg_get_function_sqlbody", "NULL"),
            ("quote_ident", "''"),
            ("pg_relation_filenode", "0"),
            ("pg_relation_size", "0"),
            ("pg_database_size", "0"),
            ("pg_total_relation_size", "0"),
            ("pg_indexes_size", "0"),
            ("pg_stat_get_dead_tuples", "0"),
            ("pg_stat_get_live_tuples", "0"),
            ("pg_stat_get_numscans", "0"),
            ("format_type", "''"),
            ("shobj_description", "NULL"),
            ("obj_description", "NULL"),
            ("col_description", "NULL"),
            ("current_query", "''"),
            ("pg_backend_pid", "0"),
            ("pg_notify", "NULL"),
            ("pg_advisory_lock", "true"),
            ("pg_advisory_unlock", "true"),
            ("pg_try_advisory_lock", "true"),
            ("pg_is_in_recovery", "false"),
            ("txid_current", "1"),
            ("age", "0::BIGINT"),
            ("pg_last_xlog_receive_location", "'0/0'"),
            ("pg_last_xlog_replay_location", "'0/0'"),
            ("pg_xlog_name", "'0/0'"),
            ("pg_xlog_location_diff", "'0'"),
            ("pg_stat_file", "NULL"),
            ("pg_read_file", "''"),
            ("pg_ls_dir", "NULL"),
            (
                "pg_available_extensions",
                "(SELECT ''::VARCHAR AS name, ''::VARCHAR AS extversion, ''::VARCHAR AS extowner, false::BOOLEAN AS extrelocatable, ''::VARCHAR[] AS extconfig, ''::VARCHAR[] AS extcondition LIMIT 0)",
            ),
            (
                "pg_available_extension_versions",
                "(SELECT ''::VARCHAR AS name, ''::VARCHAR AS version, ''::VARCHAR AS extversion, ''::VARCHAR AS extowner, false::BOOLEAN AS extrelocatable, ''::VARCHAR[] AS extconfig, ''::VARCHAR[] AS extcondition, ''::VARCHAR AS module, ''::VARCHAR AS schema LIMIT 0)",
            ),
            // NOTE: duplicate entry — this line is unreachable because the first match above always wins.
            // Kept as a safety net in case the first entry is removed or reordered.
            ("pg_available_extensions", "NULL"),
        ];

        for (func, _) in &pg_funcs {
            result = result.replace(
                &format!("pg_compat.\"{}\"(", func),
                &format!("pg_compat.{}(", func),
            );
            result = result.replace(
                &format!("\"pg_compat\".\"{}\"(", func),
                &format!("\"pg_compat\".{}(", func),
            );
            result = result.replace(
                &format!("\"pg_compat\".{}(", func),
                &format!("pg_compat.{}(", func),
            );
            result = result.replace(&format!("\"{}\"(", func), &format!("{}(", func));
        }

        let upper = result.to_uppercase();
        for (func, replacement) in &pg_funcs {
            let func_upper = func.to_uppercase();
            let mut offset = 0;
            while offset < upper.len() {
                if let Some(pos) = upper[offset..].find(func_upper.as_str()) {
                    let abs = offset + pos;
                    if let Some(start) = Self::pg_function_call_start(&result, abs) {
                        let mut open = abs + func.len();
                        while result
                            .as_bytes()
                            .get(open)
                            .map(|b| b.is_ascii_whitespace())
                            .unwrap_or(false)
                        {
                            open += 1;
                        }

                        if open < result.len() && result.as_bytes().get(open) == Some(&b'(') {
                            let close = Self::find_matching_paren(&result, open);
                            if let Some(end) = close {
                                result = format!(
                                    "{}{}{}",
                                    &result[..start],
                                    replacement,
                                    &result[end + 1..]
                                );
                                // Recursive reapplication — handles multiple calls in one query
                                return self.rewrite_pg_functions(&result);
                            }
                        }
                    }
                    offset = abs + func.len();
                } else {
                    break;
                }
            }
        }

        Self::rewrite_unknown_pg_schema_functions(&result)
    }

    fn rewrite_unknown_pg_schema_functions(sql: &str) -> String {
        let mut result = sql.to_string();

        loop {
            let Some((start, open, replacement)) = Self::find_unknown_pg_schema_function(&result)
            else {
                break;
            };
            let Some(end) = Self::find_matching_paren(&result, open) else {
                break;
            };
            result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
        }

        result
    }

    fn find_unknown_pg_schema_function(sql: &str) -> Option<(usize, usize, &'static str)> {
        let upper = sql.to_uppercase();

        let mut best: Option<(usize, usize, &'static str)> = None;
        for schema in ["PG_COMPAT", "\"PG_COMPAT\"", "PG_CATALOG", "\"PG_CATALOG\""] {
            let mut offset = 0;
            while let Some(pos) = upper[offset..].find(schema) {
                let start = offset + pos;
                let mut i = start + schema.len();
                while sql
                    .as_bytes()
                    .get(i)
                    .map(|b| b.is_ascii_whitespace())
                    .unwrap_or(false)
                {
                    i += 1;
                }
                if sql.as_bytes().get(i) != Some(&b'.') {
                    offset = start + schema.len();
                    continue;
                }
                i += 1;
                while sql
                    .as_bytes()
                    .get(i)
                    .map(|b| b.is_ascii_whitespace())
                    .unwrap_or(false)
                {
                    i += 1;
                }

                let (func_name, mut after_name) = if sql.as_bytes().get(i) == Some(&b'"') {
                    let name_start = i + 1;
                    let Some(end_rel) = sql[name_start..].find('"') else {
                        offset = start + schema.len();
                        continue;
                    };
                    let name_end = name_start + end_rel;
                    (&sql[name_start..name_end], name_end + 1)
                } else {
                    let name_start = i;
                    while sql
                        .as_bytes()
                        .get(i)
                        .map(|&b| b.is_ascii_alphanumeric() || b == b'_')
                        .unwrap_or(false)
                    {
                        i += 1;
                    }
                    if i == name_start {
                        offset = start + schema.len();
                        continue;
                    }
                    (&sql[name_start..i], i)
                };

                while sql
                    .as_bytes()
                    .get(after_name)
                    .map(|b| b.is_ascii_whitespace())
                    .unwrap_or(false)
                {
                    after_name += 1;
                }

                if sql.as_bytes().get(after_name) == Some(&b'(') {
                    let replacement = if func_name.eq_ignore_ascii_case("age") {
                        "0::BIGINT"
                    } else {
                        "NULL"
                    };
                    match best {
                        Some((best_start, _, _)) if best_start < start => {}
                        _ => best = Some((start, after_name, replacement)),
                    }
                }

                offset = start + schema.len();
            }
        }

        best
    }

    fn pg_function_call_start(sql: &str, func_start: usize) -> Option<usize> {
        let bytes = sql.as_bytes();
        if func_start > bytes.len() {
            return None;
        }

        let mut i = func_start;
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }

        if i > 0 && bytes[i - 1] == b'.' {
            let mut schema_end = i - 1;
            while schema_end > 0 && bytes[schema_end - 1].is_ascii_whitespace() {
                schema_end -= 1;
            }

            for schema in ["pg_compat", "\"pg_compat\"", "pg_catalog", "\"pg_catalog\""] {
                if schema_end >= schema.len()
                    && sql[schema_end - schema.len()..schema_end].eq_ignore_ascii_case(schema)
                {
                    return Some(schema_end - schema.len());
                }
            }

            return None;
        }

        if func_start == 0
            || !bytes
                .get(func_start - 1)
                .map(|&b| b.is_ascii_alphanumeric() || b == b'_')
                .unwrap_or(false)
        {
            Some(func_start)
        } else {
            None
        }
    }

    fn rewrite_pg_builtin_prefixes(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        for func in ["current_user"] {
            result = result.replace(&format!("pg_compat.\"{}\"(", func), &format!("{}(", func));
            result = result.replace(&format!("pg_compat.{}(", func), &format!("{}(", func));
            result = result.replace(&format!("\"{}\"(", func), &format!("{}(", func));
        }
        result
    }

    // Converts Postgres array literals '{a,b,c}'::type[] to DuckDB list syntax ['a','b','c'].
    // First handles known hardcoded patterns, then applies a general {}-to-[] conversion
    // with recursive reapplication (same pattern as rewrite_pg_functions).
    fn rewrite_pg_array_literals(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        let patterns = [
            ("'{r,v,m}'::CHAR[]", "['r','v','m']"),
            ("'{r,v,m}'::char[]", "['r','v','m']"),
            ("'{r,v,m}'::VARCHAR[]", "['r','v','m']"),
            ("'{r,v,m}'::varchar[]", "['r','v','m']"),
            ("'{r,v,m}'::TEXT[]", "['r','v','m']"),
            ("'{r,v,m}'::text[]", "['r','v','m']"),
            ("'{r}'::CHAR[]", "['r']"),
            ("'{r}'::char[]", "['r']"),
            ("'{i,s}'::CHAR[]", "['i','s']"),
            ("'{i,s}'::char[]", "['i','s']"),
        ];

        for (pat, repl) in &patterns {
            result = result.replace(pat, repl);
        }

        let mut start = 0;
        while let Some(pos) = result[start..].find("'{") {
            let abs = start + pos;
            if let Some(end_rel) = result[abs + 2..].find("}'") {
                let end = abs + 2 + end_rel;
                let inner = &result[abs + 2..end];
                let rest = &result[end + 2..];
                if let Some(cast_pos) = rest.find("::") {
                    let type_end = rest[cast_pos + 2..]
                        .find(|c: char| !c.is_ascii_alphanumeric() && c != '[' && c != ']')
                        .map(|p| cast_pos + 2 + p)
                        .unwrap_or(rest.len().min(cast_pos + 10));
                    let elements: Vec<&str> = inner.split(',').collect();
                    let array_lit = format!(
                        "[{}]",
                        elements
                            .iter()
                            .map(|e| format!("'{}'", e.trim()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    let skip = type_end;
                    result = format!("{}{}{}", &result[..abs], array_lit, &rest[skip..]);
                    // Recursive — may have multiple array literals in one query
                    return self.rewrite_pg_array_literals(&result);
                }
                start = end + 2;
            } else {
                break;
            }
        }

        result
    }

    fn rewrite_bracketed_select_arrays(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        while let Some(start) = result.to_uppercase().find("[SELECT ") {
            let bytes = result.as_bytes();
            let mut i = start + 1;
            let mut depth = 0i32;
            let mut in_string = false;
            let mut end = None;

            while i < bytes.len() {
                match bytes[i] {
                    b'\'' if i == 0 || bytes[i - 1] != b'\\' => in_string = !in_string,
                    b'(' if !in_string => depth += 1,
                    b')' if !in_string => depth -= 1,
                    b']' if !in_string && depth == 0 => {
                        end = Some(i);
                        break;
                    }
                    _ => {}
                }
                i += 1;
            }

            if let Some(end) = end {
                result = format!(
                    "{}ARRAY({}){}",
                    &result[..start],
                    &result[start + 1..end],
                    &result[end + 1..]
                );
            } else {
                break;
            }
        }

        result
    }

    fn rewrite_unnest_column_aliases(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        loop {
            let upper = result.to_uppercase();
            let Some(unnest_pos) = upper.find("UNNEST(") else {
                break;
            };
            let open = unnest_pos + "UNNEST".len();
            let Some(close) = Self::find_matching_paren(&result, open) else {
                break;
            };

            let after = &result[close + 1..];
            let after_trimmed = after.trim_start();
            let skipped_ws = after.len() - after_trimmed.len();
            let Some(alias_rest) = after_trimmed
                .strip_prefix("AS k")
                .or_else(|| after_trimmed.strip_prefix("as k"))
            else {
                break;
            };

            if alias_rest
                .as_bytes()
                .first()
                .map(|&b| b == b'(' || b.is_ascii_alphanumeric() || b == b'_')
                .unwrap_or(false)
            {
                break;
            }

            let insert_at = close + 1 + skipped_ws + "AS k".len();
            result.insert_str(insert_at, "(k)");
        }

        result
    }

    fn rewrite_pg_pseudo_type_casts(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        for pseudo_type in [
            "REGOPER",
            "REGOPERATOR",
            "REGCLASS",
            "REGTYPE",
            "REGPROC",
            "NAME",
        ] {
            result = result.replace(&format!(" AS {pseudo_type})"), " AS TEXT)");
            result = result.replace(&format!(" AS pg_catalog.{pseudo_type})"), " AS TEXT)");
            result = result.replace(&format!(" AS pg_compat.{pseudo_type})"), " AS TEXT)");
        }
        result
    }

    fn rewrite_datagrip_extension_version_arrays(&self, sql: &str) -> String {
        let upper = sql.to_uppercase();
        if !(upper.contains("PG_COMPAT.PG_EXTENSION AS E")
            || upper.contains("PG_COMPAT.PG_EXTENSION E"))
        {
            return sql.to_string();
        }

        sql.replace("UNNEST(available_versions)", "UNNEST(E.available_versions)")
            .replace(" unnest > extversion", " unnest > E.extversion")
            .replace(" unnest <> extversion", " unnest <> E.extversion")
    }

    // Strips ORDER BY clauses inside STRING_AGG() because DuckDB does not support
    // order-sensitive aggregate syntax. Deletions are applied in reverse to preserve
    // byte positions.
    fn rewrite_string_agg_order_by(&self, sql: &str) -> String {
        let upper = sql.to_uppercase();
        let mut offsets = Vec::new();
        let search = "STRING_AGG";
        let mut pos = 0;
        while let Some(idx) = upper[pos..].find(search) {
            let abs = pos + idx;
            let after = abs + search.len();
            if after < upper.len() && upper.as_bytes()[after] == b'(' {
                if let Some(close) = Self::find_matching_paren(sql, after) {
                    let inner = &sql[after + 1..close];
                    let inner_upper = inner.to_uppercase();
                    if let Some(ob) = inner_upper.find(" ORDER BY ") {
                        let order_start = after + 1 + ob;
                        let order_end = close;
                        offsets.push((order_start, order_end));
                    }
                }
            }
            pos = abs + search.len();
        }

        if offsets.is_empty() {
            return sql.to_string();
        }

        let mut result = sql.to_string();
        // Reverse removal to keep earlier offsets stable
        for (start, end) in offsets.into_iter().rev() {
            result = format!("{}{}", &result[..start], &result[end..]);
        }
        result
    }

    fn rewrite_pg_table_functions(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        result = Self::rewrite_pg_schema_table_function(
            &result,
            "pg_indexam_has_property",
            "(SELECT false AS pg_indexam_has_property)",
        );
        result = result.replace(
            "pg_compat.pg_get_keywords()",
            "(SELECT ''::VARCHAR AS word LIMIT 0)",
        );
        result = result.replace(
            "pg_catalog.pg_get_keywords()",
            "(SELECT ''::VARCHAR AS word LIMIT 0)",
        );
        result = result.replace("pg_get_keywords()", "(SELECT ''::VARCHAR AS word LIMIT 0)");
        result
    }

    fn rewrite_pg_schema_table_function(sql: &str, func: &str, replacement: &str) -> String {
        let mut result = sql.to_string();

        loop {
            let upper = result.to_uppercase();
            let func_upper = func.to_uppercase();
            let Some(pos) = upper.find(&func_upper) else {
                break;
            };
            let Some(start) = Self::pg_function_call_start(&result, pos) else {
                break;
            };
            let mut open = pos + func.len();
            while result
                .as_bytes()
                .get(open)
                .map(|b| b.is_ascii_whitespace())
                .unwrap_or(false)
            {
                open += 1;
            }
            if result.as_bytes().get(open) != Some(&b'(') {
                break;
            }
            let Some(end) = Self::find_matching_paren(&result, open) else {
                break;
            };
            result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
        }

        result
    }

    fn rewrite_public_schema(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        result = result.replace("\"public\".", "\"main\".");
        result = result.replace("public.", "main.");
        result = result.replace("PUBLIC.", "main.");
        result = result.replace(" FROM public.", " FROM main.");
        result = result.replace(" from public.", " from main.");
        result = result.replace(" JOIN public.", " JOIN main.");
        result = result.replace(" join public.", " join main.");
        result = result.replace("UPDATE public.", "UPDATE main.");
        result = result.replace("INSERT INTO public.", "INSERT INTO main.");
        result
    }

    fn rewrite_ctid_projection(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        for ctid in ["CTID", "ctid"] {
            result = result.replace(&format!(", {ctid} FROM "), ", 0::BIGINT AS CTID FROM ");
            result = result.replace(
                &format!("SELECT {ctid} FROM "),
                "SELECT 0::BIGINT AS CTID FROM ",
            );
            result = result.replace(&format!(", {ctid},"), ", 0::BIGINT AS CTID,");
            result = result.replace(&format!("SELECT {ctid},"), "SELECT 0::BIGINT AS CTID,");
        }
        result
    }

    fn rewrite_info_schema(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        result = result.replace("information_schema.routines", "pg_compat.routines");
        result = result.replace("information_schema.parameters", "pg_compat.parameters");
        result = result.replace("INFORMATION_SCHEMA.ROUTINES", "pg_compat.routines");
        result = result.replace("INFORMATION_SCHEMA.PARAMETERS", "pg_compat.parameters");
        result = result.replace("information_schema.tables", "pg_compat.pg_tables");
        result = result.replace("INFORMATION_SCHEMA.TABLES", "pg_compat.pg_tables");
        // Inline subquery avoids DuckDB v1.1 view binder bug (views referencing
        // information_schema.columns with > 8 columns lose column metadata).
        // Two-step replace avoids recursion: marker -> subquery.
        result = result.replace("information_schema.columns", "____ISC____");
        result = result.replace("INFORMATION_SCHEMA.COLUMNS", "____ISC____");
        result = result.replace(
            "____ISC____",
            "(SELECT *, data_type AS udt_name FROM information_schema.columns)",
        );
        result
    }

    fn rewrite_info_schema_type_aliases(&self, sql: &str) -> String {
        let mut result = sql.to_string();
        for type_name in [
            "information_schema.character_data",
            "INFORMATION_SCHEMA.CHARACTER_DATA",
        ] {
            result = result.replace(type_name, "VARCHAR");
        }
        result
    }

    fn rewrite_pg_specific(&self, sql: &str) -> String {
        let upper = sql.trim().to_uppercase();

        if upper.starts_with("SET ") {
            return String::new();
        }

        if upper.starts_with("SHOW ") {
            return self.rewrite_show(sql.trim());
        }

        let mut result = sql.to_string();
        result = result.replace("::regoperator", "::TEXT");
        result = result.replace("::REGOPERATOR", "::TEXT");
        result = result.replace(" AS NAME)", " AS TEXT)");
        result = result.replace(" AS pg_catalog.NAME)", " AS TEXT)");
        result = result.replace(" AS pg_compat.NAME)", " AS TEXT)");
        result = result.replace("::regclass", "");
        result = result.replace("::regtype", "");
        result = result.replace("::regproc", "");
        result = result.replace("::regoper", "::TEXT");
        result = result.replace("::REGOPER", "::TEXT");
        result = result.replace("\"char\"", "VARCHAR(1)");

        result = self.rewrite_info_schema_type_aliases(&result);
        result = self.rewrite_public_schema(&result);
        result = self.rewrite_ctid_projection(&result);
        result = self.rewrite_pg_tables(&result);
        result = self.rewrite_pg_table_functions(&result);
        result = self.rewrite_pg_functions(&result);
        result = self.rewrite_pg_array_literals(&result);
        result = self.rewrite_any_array_comparisons(&result);
        result = self.rewrite_string_agg_order_by(&result);
        result = self.rewrite_info_schema(&result);

        result
    }

    fn rewrite_any_array_comparisons(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        loop {
            let upper = result.to_uppercase();
            let Some(any_pos) = upper.find("ANY(") else {
                break;
            };
            let open = any_pos + "ANY".len();
            let Some(close) = Self::find_matching_paren(&result, open) else {
                break;
            };
            let inner = &result[open + 1..close];
            result = format!(
                "{}list_extract({}, 1){}",
                &result[..any_pos],
                inner,
                &result[close + 1..]
            );
        }

        result
    }

    // Finds the matching closing parenthesis for the opening paren at open_pos,
    // skipping parens inside single-quoted string literals to avoid false matches.
    fn find_matching_paren(s: &str, open_pos: usize) -> Option<usize> {
        let bytes = s.as_bytes();
        if bytes.get(open_pos) != Some(&b'(') {
            return None;
        }
        let mut depth = 0i32;
        let mut i = open_pos;
        let mut in_string = false;
        while i < bytes.len() {
            match bytes[i] {
                b'\'' if i == 0 || bytes[i - 1] != b'\\' => in_string = !in_string,
                b'(' if !in_string => depth += 1,
                b')' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_basic_select() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT 1").unwrap();
        assert!(result.contains("SELECT"), "Expected SELECT in: {result}");
    }

    #[test]
    fn test_transpile_set_suppressed() {
        let t = Transpiler::new();
        let result = t.rewrite("SET search_path TO public").unwrap();
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_transpile_show_transaction_isolation() {
        let t = Transpiler::new();
        let result = t.rewrite("SHOW TRANSACTION ISOLATION LEVEL").unwrap();
        assert!(result.contains("read committed"), "Result: {result}");
    }

    #[test]
    fn test_transpile_show_datestyle() {
        let t = Transpiler::new();
        let result = t.rewrite("SHOW DATESTYLE").unwrap();
        assert!(result.contains("ISO"), "Result: {result}");
        assert!(result.contains("datestyle"), "Result: {result}");
    }

    #[test]
    fn test_transpile_show_timezone() {
        let t = Transpiler::new();
        let result = t.rewrite("SHOW TIMEZONE").unwrap();
        assert!(result.contains("UTC"), "Result: {result}");
    }

    #[test]
    fn test_transpile_show_unknown() {
        let t = Transpiler::new();
        let result = t.rewrite("SHOW SOME_UNKNOWN_VAR").unwrap();
        assert!(result.contains("some_unknown_var"), "Result: {result}");
    }

    #[test]
    fn test_transpile_regclass_removes() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT * FROM my_table WHERE id = 'foo'::regclass")
            .unwrap();
        assert!(!result.contains("::regclass"));
    }

    #[test]
    fn test_transpile_information_schema_character_data_cast() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT seq.seqcache::information_schema.character_data AS identity_cache FROM pg_sequence seq").unwrap();
        assert!(
            !result.contains("information_schema.character_data"),
            "Result: {result}"
        );
        assert!(
            result.contains("VARCHAR") || result.contains("TEXT"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_information_schema_character_data_cast_function() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT CAST(seq.seqcache AS information_schema.character_data) AS identity_cache FROM pg_sequence seq").unwrap();
        assert!(
            !result.contains("information_schema.character_data"),
            "Result: {result}"
        );
        assert!(
            result.contains("VARCHAR") || result.contains("TEXT"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_coalesce() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT COALESCE(a, b) FROM t").unwrap();
        assert!(result.contains("COALESCE"));
    }

    #[test]
    fn test_transpile_pg_catalog_prefix() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT * FROM pg_catalog.pg_database").unwrap();
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
        assert!(!result.contains("pg_catalog."));
    }

    #[test]
    fn test_transpile_quoted_pg_catalog_prefix() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT * FROM \"pg_catalog\".\"pg_extension\"")
            .unwrap();
        assert!(
            result.contains("pg_compat.pg_extension"),
            "Result: {result}"
        );
        assert!(!result.contains("\"pg_catalog\""));
        assert!(!result.contains("\"pg_extension\""));
    }

    #[test]
    fn test_transpile_bare_pg_table() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT datname FROM pg_database").unwrap();
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_join() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT d.datname, t.spcname FROM pg_database AS d LEFT JOIN pg_tablespace AS t ON d.dattablespace = t.oid").unwrap();
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
        assert!(
            result.contains("pg_compat.pg_tablespace"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_no_double_prefix() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT * FROM pg_compat.pg_database").unwrap();
        assert_eq!(result.matches("pg_compat.").count(), 1, "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_function_replacement() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT pg_get_userbyid(d.datdba) AS databaseowner FROM pg_database d")
            .unwrap();
        assert!(!result.contains("pg_get_userbyid"), "Result: {result}");
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_encoding_to_char() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT pg_encoding_to_char(d.encoding) AS encodingname FROM pg_database d")
            .unwrap();
        assert!(!result.contains("pg_encoding_to_char"), "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_privilege_functions() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "SELECT pg_catalog.has_database_privilege(d.datname, 'CONNECT') AS can_connect \
                 FROM pg_database d",
            )
            .unwrap();
        assert!(
            !result.contains("has_database_privilege"),
            "Result: {result}"
        );
        assert!(!result.contains("pg_compat.has"), "Result: {result}");
        assert!(result.to_uppercase().contains("TRUE"), "Result: {result}");
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
    }

    #[test]
    fn test_transpile_quoted_pg_privilege_functions() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "SELECT \"pg_catalog\".\"has_schema_privilege\"(n.oid, 'CREATE') AS can_create \
                 FROM pg_namespace n",
            )
            .unwrap();
        assert!(!result.contains("has_schema_privilege"), "Result: {result}");
        assert!(!result.contains("pg_compat.has"), "Result: {result}");
        assert!(result.to_uppercase().contains("TRUE"), "Result: {result}");
        assert!(
            result.contains("pg_compat.pg_namespace"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_datagrip_txid_query() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "select case
                   when pg_catalog.pg_is_in_recovery()
                     then null
                   else
                     (pg_catalog.txid_current() % 4294967296)::varchar::bigint
                 end as current_txid",
            )
            .unwrap();
        assert!(!result.contains("txid_current"), "Result: {result}");
        assert!(!result.contains("pg_compat."), "Result: {result}");
    }

    #[test]
    fn test_transpile_datagrip_database_order_query() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
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
        assert!(
            !result.contains("pg_compat.current_database"),
            "Result: {result}"
        );
        assert!(!result.contains("pg_get_userbyid"), "Result: {result}");
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
        assert!(
            result.contains("pg_compat.pg_shdescription"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_current_schema_public_view() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "select current_database() as db, current_schema() as s, current_schemas(false) as schemas",
            )
            .unwrap();
        assert!(!result.contains("current_database"), "Result: {result}");
        assert!(!result.contains("current_schema"), "Result: {result}");
        assert!(!result.contains("current_schemas"), "Result: {result}");
        assert!(result.contains("'postgres'"), "Result: {result}");
        assert!(result.contains("'public'"), "Result: {result}");
    }

    #[test]
    fn test_transpile_array_select_postprocess() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "SELECT ARRAY(SELECT unnest FROM UNNEST(available_versions) WHERE unnest <> extversion) AS other_versions FROM pg_extension",
            )
            .unwrap();
        assert!(!result.contains("[SELECT"), "Result: {result}");
        assert!(result.contains("ARRAY(SELECT"), "Result: {result}");
    }

    #[test]
    fn test_transpile_any_array_comparison() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT * FROM pg_index i LEFT JOIN pg_opclass o ON o.oid = ANY(i.indclass)")
            .unwrap();
        assert!(!result.contains("ANY("), "Result: {result}");
        assert!(
            result.to_uppercase().contains("LIST_EXTRACT"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_shobj_description() {
        let t = Transpiler::new();
        let result = t
            .rewrite(
                "SELECT shobj_description(d.oid, 'pg_database') AS description FROM pg_database d",
            )
            .unwrap();
        assert!(result.contains("pg_compat.pg_database"), "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_array_literal() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT * FROM pg_class WHERE relkind = ANY('{r,v,m}'::char[])")
            .unwrap();
        assert!(!result.contains("'{r,v,m}'"), "Result: {result}");
        assert!(result.contains("['r'"), "Result: {result}");
    }

    #[test]
    fn test_transpile_pg_available_extension_versions() {
        let t = Transpiler::new();
        let result = t
            .rewrite("SELECT * FROM pg_available_extension_versions()")
            .unwrap();
        assert!(
            !result.contains("pg_available_extension_versions"),
            "Result: {result}"
        );
    }

    #[test]
    fn test_transpile_string_agg_order_by() {
        let t = Transpiler::new();
        let result = t.rewrite("SELECT string_agg(p.udt_name, ', ' ORDER BY p.ordinal_position) AS object_info FROM t").unwrap();
        assert!(!result.contains("ORDER BY"), "Result: {result}");
    }
}
