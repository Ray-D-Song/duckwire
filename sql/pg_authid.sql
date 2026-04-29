CREATE OR REPLACE VIEW pg_compat.pg_authid AS
SELECT
    10::BIGINT AS oid,
    'postgres'::VARCHAR AS rolname,
    true::BOOLEAN AS rolsuper,
    true::BOOLEAN AS rolcreaterole,
    true::BOOLEAN AS rolcreatedb,
    true::BOOLEAN AS rolcanlogin,
    true::BOOLEAN AS rolinherit,
    false::BOOLEAN AS rolreplication,
    NULL::VARCHAR AS rolpassword,
    false::BOOLEAN AS rolbypassrls
;
