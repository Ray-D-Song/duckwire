CREATE OR REPLACE VIEW pg_compat.pg_database AS
SELECT
    16384::BIGINT AS oid,
    'postgres'::VARCHAR AS datname,
    10::BIGINT AS datdba,
    6::INTEGER AS encoding,
    'en_US.UTF-8'::VARCHAR AS datcollate,
    'en_US.UTF-8'::VARCHAR AS datctype,
    1663::BIGINT AS dattablespace,
    NULL::VARCHAR AS datacl,
    false::BOOLEAN AS datistemplate,
    true::BOOLEAN AS datallowconn,
    -1::INTEGER AS datconnlimit
;
