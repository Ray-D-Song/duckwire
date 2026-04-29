CREATE OR REPLACE VIEW pg_compat.pg_stat_activity AS
SELECT
    0::BIGINT AS pid,
    current_database()::VARCHAR AS datname,
    10::BIGINT AS datid,
    10::BIGINT AS usrid,
    'postgres'::VARCHAR AS usename,
    'active'::VARCHAR AS state,
    NULL::VARCHAR AS query,
    now()::VARCHAR AS query_start,
    NULL::VARCHAR AS state_change,
    NULL::VARCHAR AS application_name,
    NULL::VARCHAR AS client_addr
;
