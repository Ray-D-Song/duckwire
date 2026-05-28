CREATE OR REPLACE VIEW pg_compat.pg_namespace AS
SELECT oid, 1::BIGINT AS xmin, nspname, nspowner::BIGINT AS nspowner, NULL::VARCHAR[] AS nspacl
FROM (
    SELECT 11::BIGINT AS oid, 'pg_catalog'::VARCHAR AS nspname, 10::BIGINT AS nspowner
    UNION ALL SELECT 9975::BIGINT, 'information_schema', 10::BIGINT
    UNION ALL SELECT 2200::BIGINT, 'public', 10::BIGINT
    UNION ALL SELECT 13330::BIGINT, 'pg_compat', 10::BIGINT
) AS sub
;
