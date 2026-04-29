CREATE OR REPLACE VIEW pg_compat.pg_user AS
SELECT 10::BIGINT AS usesysid, 'postgres'::VARCHAR AS usename,
       true::BOOLEAN AS usesuper, true::BOOLEAN AS usecreatedb,
       NULL::VARCHAR AS usepassupd, NULL::VARCHAR AS valuntil,
       NULL::VARCHAR[] AS useconfig
;
