CREATE OR REPLACE VIEW pg_compat.pg_am AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS amname,
       0::BIGINT AS amhandler, 'i'::VARCHAR AS amtype LIMIT 0
;
