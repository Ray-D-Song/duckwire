CREATE OR REPLACE VIEW pg_compat.pg_language AS
SELECT oid, lanname, lanispl, lanpltrusted, lanplcallfoid,
       laninline, lanvalidator, lanacl
FROM (VALUES
    (12::BIGINT, 'sql'::VARCHAR, true::BOOLEAN, true::BOOLEAN, 0::BIGINT, 0::BIGINT, 2246::BIGINT, NULL::VARCHAR[]),
    (13, 'c', false, false, 0, 0, 2279, NULL),
    (14, 'internal', false, false, 0, 0, 2246, NULL)
) AS t(oid, lanname, lanispl, lanpltrusted, lanplcallfoid, laninline, lanvalidator, lanacl)
;
