CREATE OR REPLACE VIEW pg_compat.pg_stat_user_tables AS
SELECT schemaname::VARCHAR, tablename::VARCHAR AS relname,
       0::BIGINT AS seq_scan, 0::BIGINT AS seq_tup_read,
       0::BIGINT AS idx_scan, 0::BIGINT AS idx_tup_fetch,
       0::BIGINT AS n_tup_ins, 0::BIGINT AS n_tup_upd,
       0::BIGINT AS n_tup_del, 0::BIGINT AS n_tup_hot_upd,
       0::BIGINT AS n_live_tup, 0::BIGINT AS n_dead_tup,
       0::BIGINT AS last_vacuum, 0::BIGINT AS last_analyze
FROM (
    SELECT
        CASE WHEN table_schema = 'main' THEN 'public' ELSE table_schema END AS schemaname,
        table_name::VARCHAR AS tablename
    FROM information_schema.tables
    WHERE table_schema IN ('public', 'main')
) AS sub
;
