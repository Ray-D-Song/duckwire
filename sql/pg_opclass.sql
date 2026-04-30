CREATE OR REPLACE VIEW pg_compat.pg_opclass AS
SELECT 0::BIGINT AS oid, ''::VARCHAR AS opcname, 0::BIGINT AS opcnamespace LIMIT 0
;