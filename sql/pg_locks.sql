CREATE OR REPLACE VIEW pg_compat.pg_locks AS
SELECT NULL::VARCHAR AS locktype, NULL::BIGINT AS database,
       NULL::BIGINT AS relation, NULL::BIGINT AS page,
       NULL::VARCHAR AS pid, NULL::VARCHAR AS mode,
       false::BOOLEAN AS granted LIMIT 0
;
