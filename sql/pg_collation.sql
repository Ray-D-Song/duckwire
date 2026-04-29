CREATE OR REPLACE VIEW pg_compat.pg_collation AS
SELECT 100::BIGINT AS oid, 'default'::VARCHAR AS collname,
       11::BIGINT AS collnamespace, 10::BIGINT AS collowner,
       'f'::VARCHAR AS collencoding, 'f'::VARCHAR AS colldefault,
       NULL::VARCHAR[] AS collacl LIMIT 0
;
