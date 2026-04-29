CREATE OR REPLACE VIEW pg_compat.pg_roles AS
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
    NULL::VARCHAR AS rolcomments,
    false::BOOLEAN AS rolbypassrls,
    NULL::VARCHAR[] AS rolconfig
;
