use std::sync::{Arc, Mutex};

use duckdb::Connection;
use tracing::info;

fn exec(conn: &Connection, sql: &str) {
    conn.execute_batch(sql).unwrap_or_else(|e| {
        let snippet = sql.lines().next().unwrap_or("");
        tracing::warn!("catalog view init failed: {e} | sql: {snippet}...");
    });
}

pub fn init_pg_compat(conn: &Arc<Mutex<Connection>>) {
    info!("Initializing pg_compat compatibility views");
    let c = conn.lock().unwrap();

    exec(&c, include_str!("../../sql/schema.sql"));
    exec(&c, include_str!("../../sql/pg_database.sql"));
    exec(&c, include_str!("../../sql/pg_tablespace.sql"));
    exec(&c, include_str!("../../sql/pg_timezone.sql"));
    exec(&c, include_str!("../../sql/pg_roles.sql"));
    exec(&c, include_str!("../../sql/pg_authid.sql"));
    exec(&c, include_str!("../../sql/pg_namespace.sql"));
    exec(&c, include_str!("../../sql/pg_class.sql"));
    exec(&c, include_str!("../../sql/pg_am.sql"));
    exec(&c, include_str!("../../sql/pg_opclass.sql"));
    exec(&c, include_str!("../../sql/pg_operator.sql"));
    exec(&c, include_str!("../../sql/pg_type.sql"));
    exec(&c, include_str!("../../sql/pg_attribute.sql"));
    exec(&c, include_str!("../../sql/pg_attrdef.sql"));
    exec(&c, include_str!("../../sql/pg_description.sql"));
    exec(&c, include_str!("../../sql/pg_shdescription.sql"));
    exec(&c, include_str!("../../sql/pg_collation.sql"));
    exec(&c, include_str!("../../sql/pg_settings.sql"));
    exec(&c, include_str!("../../sql/pg_locks.sql"));
    exec(&c, include_str!("../../sql/pg_stat_activity.sql"));
    exec(&c, include_str!("../../sql/pg_inherits.sql"));
    exec(&c, include_str!("../../sql/pg_index.sql"));
    exec(&c, include_str!("../../sql/pg_foreign_table.sql"));
    exec(&c, include_str!("../../sql/pg_foreign_server.sql"));
    exec(&c, include_str!("../../sql/pg_constraint.sql"));
    exec(&c, include_str!("../../sql/pg_proc.sql"));
    exec(&c, include_str!("../../sql/pg_language.sql"));
    exec(&c, include_str!("../../sql/pg_extension.sql"));
    exec(&c, include_str!("../../sql/pg_enum.sql"));
    exec(&c, include_str!("../../sql/pg_conversion.sql"));
    exec(&c, include_str!("../../sql/pg_auth_members.sql"));
    exec(&c, include_str!("../../sql/pg_shdepend.sql"));
    exec(&c, include_str!("../../sql/pg_depend.sql"));
    exec(&c, include_str!("../../sql/pg_tables.sql"));
    exec(&c, include_str!("../../sql/pg_views.sql"));
    exec(&c, include_str!("../../sql/pg_matviews.sql"));
    exec(&c, include_str!("../../sql/pg_columns_view.sql"));
    exec(&c, include_str!("../../sql/pg_stat_user_tables.sql"));
    exec(&c, include_str!("../../sql/pg_user.sql"));
    exec(&c, include_str!("../../sql/pg_shadow.sql"));
    exec(&c, include_str!("../../sql/pg_policies.sql"));
    exec(&c, include_str!("../../sql/pg_routines.sql"));
    exec(&c, include_str!("../../sql/pg_parameters.sql"));
    exec(&c, include_str!("../../sql/pg_deep_introspection.sql"));

    drop(c);
    info!("pg_compat compatibility views initialized");
}
