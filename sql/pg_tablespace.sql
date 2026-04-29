CREATE OR REPLACE VIEW pg_compat.pg_tablespace AS
SELECT
    1663::BIGINT AS oid,
    'pg_default'::VARCHAR AS spcname,
    10::BIGINT AS spcowner,
    NULL::VARCHAR AS spclocation,
    NULL::VARCHAR[] AS spcacl,
    NULL::VARCHAR[] AS spcoptions
;
