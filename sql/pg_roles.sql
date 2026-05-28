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
    -1::INTEGER AS rolconnlimit,
    NULL::TIMESTAMP AS rolvaliduntil,
    false::BOOLEAN AS rolbypassrls,
    NULL::VARCHAR AS rolpassword,
    NULL::VARCHAR AS rolcomments,
    NULL::VARCHAR[] AS rolconfig
;
