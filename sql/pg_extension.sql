CREATE OR REPLACE VIEW pg_compat.pg_extension AS
SELECT 0::BIGINT AS oid, ''::VARCHAR AS extname,
       '1.0'::VARCHAR AS extversion, 0::BIGINT AS extowner,
       NULL::VARCHAR[] AS extrelocatable, NULL::VARCHAR[] AS extconfig,
       NULL::VARCHAR[] AS extcondition LIMIT 0
;
