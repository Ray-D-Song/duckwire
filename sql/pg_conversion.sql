CREATE OR REPLACE VIEW pg_compat.pg_conversion AS
SELECT 0::BIGINT AS oid, ''::VARCHAR AS conname, 11::BIGINT AS connamespace,
       10::BIGINT AS conowner, 0::BIGINT AS conforencoding,
       0::BIGINT AS contoencoding, ''::BIGINT AS conproc,
       true::BOOLEAN AS condefault LIMIT 0
;
