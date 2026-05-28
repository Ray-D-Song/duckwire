CREATE OR REPLACE VIEW pg_compat.pg_extension AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS extname,
       '1.0'::VARCHAR AS extversion, 0::BIGINT AS extowner,
       2200::BIGINT AS extnamespace, false::BOOLEAN AS extrelocatable,
       ['1.0']::VARCHAR[] AS available_versions,
       NULL::VARCHAR[] AS extconfig,
       NULL::VARCHAR[] AS extcondition LIMIT 0
;
