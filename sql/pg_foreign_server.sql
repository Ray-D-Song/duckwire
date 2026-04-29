CREATE OR REPLACE VIEW pg_compat.pg_foreign_server AS
SELECT 0::BIGINT AS oid, ''::VARCHAR AS srvname, 0::BIGINT AS srvowner,
       NULL::VARCHAR AS srvtype, NULL::VARCHAR AS srvversion,
       NULL::VARCHAR[] AS srvacl, NULL::VARCHAR[] AS srvoptions LIMIT 0
;
