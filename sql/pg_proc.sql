CREATE OR REPLACE VIEW pg_compat.pg_proc AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, 'version'::VARCHAR AS proname,
       11::BIGINT AS pronamespace, 10::BIGINT AS proowner,
       12::BIGINT AS prolang, false::BOOLEAN AS proisagg,
       true::BOOLEAN AS prosecdef, false::BOOLEAN AS proisstrict,
       true::BOOLEAN AS proretset, 'v'::VARCHAR AS provolatile,
       'f'::VARCHAR AS propricer, 0::FLOAT AS procost,
       0::FLOAT AS prorows, NULL::VARCHAR[] AS proconfig,
       false::BOOLEAN AS proleakproof,
       NULL::VARCHAR[] AS proacl, 'i'::VARCHAR AS proargmodes,
       NULL::VARCHAR[] AS proargnames, NULL::VARCHAR[] AS proargdefaults,
       0::BIGINT AS prorettype, '{25}'::VARCHAR[] AS proargtypes,
       NULL::BIGINT[] AS proallargtypes,
       0::BIGINT AS pronargs, 0::BIGINT AS pronargdefaults,
       0::BIGINT AS provariadic,
       'f'::VARCHAR AS prokind, false::BOOLEAN AS proposehandler,
       NULL::VARCHAR AS prosrc, NULL::VARCHAR AS probin,
       'f'::VARCHAR AS proparallel LIMIT 0
;
