CREATE OR REPLACE VIEW pg_compat.pg_collation AS
SELECT 100::BIGINT AS oid, 1::BIGINT AS xmin, 'default'::VARCHAR AS collname,
       11::BIGINT AS collnamespace, 10::BIGINT AS collowner,
       'f'::VARCHAR AS collencoding, 'f'::VARCHAR AS colldefault,
       NULL::VARCHAR AS collcollate, NULL::VARCHAR AS collctype,
       NULL::VARCHAR[] AS collacl LIMIT 0
;
