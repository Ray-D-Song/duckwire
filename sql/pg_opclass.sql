CREATE OR REPLACE VIEW pg_compat.pg_opclass AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS opcname,
       0::BIGINT AS opcnamespace, 10::BIGINT AS opcowner,
       0::BIGINT AS opcmethod, 0::BIGINT AS opcfamily,
       0::BIGINT AS opcintype, false::BOOLEAN AS opcdefault,
       0::BIGINT AS opckeytype LIMIT 0
;
