CREATE OR REPLACE VIEW pg_compat.pg_constraint AS
SELECT 0::BIGINT AS oid, 'p'::VARCHAR AS contype,
       0::BIGINT AS conrelid, NULL::BIGINT[] AS conkey LIMIT 0
;
