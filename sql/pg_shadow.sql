CREATE OR REPLACE VIEW pg_compat.pg_shadow AS
SELECT 10::BIGINT AS usesysid, 'postgres'::VARCHAR AS usename,
       NULL::VARCHAR AS passwd, NULL::VARCHAR AS valuntil,
       NULL::VARCHAR[] AS useconfig
;
