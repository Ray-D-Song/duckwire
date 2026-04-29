CREATE OR REPLACE VIEW pg_compat.pg_settings AS
SELECT name, setting, unit, category, short_desc, extra_desc,
       context, vartype, source, min_val, max_val
FROM (
    SELECT 'server_version' AS name, '16.0' AS setting, NULL::VARCHAR AS unit, NULL::VARCHAR AS category, NULL::VARCHAR AS short_desc, NULL::VARCHAR AS extra_desc, NULL::VARCHAR AS context, NULL::VARCHAR AS vartype, NULL::VARCHAR AS source, NULL::VARCHAR AS min_val, NULL::VARCHAR AS max_val
    UNION ALL SELECT 'server_version_num', '160000', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'server_encoding', 'UTF8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'lc_collate', 'en_US.UTF-8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'lc_ctype', 'en_US.UTF-8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'timezone', 'UTC', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'datestyle', 'ISO', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'client_encoding', 'UTF8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'standard_conforming_strings', 'on', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'integer_datetimes', 'on', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'is_superuser', 'off', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'default_transaction_isolation', 'read committed', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
    UNION ALL SELECT 'max_connections', '100', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL
) AS sub
;
