-- OIDs are derived from row_number() to join with pg_attribute.
-- Both must use the same ORDER BY to keep OID assignments consistent.
CREATE OR REPLACE VIEW pg_compat.pg_class AS
SELECT oid, relname, relnamespace, relkind, relowner, reltablespace,
       relpages, reltuples, relacl, reloptions, relhasindex, relhasrules,
       relhastriggers, relpersistence, relispartition, relpartbound, reltype
FROM (
    SELECT
        row_number() OVER (ORDER BY table_schema, table_name)::BIGINT + 16383 AS oid,
        table_name AS relname,
        2200::BIGINT AS relnamespace,
        CASE table_type
            WHEN 'BASE TABLE' THEN 'r'::VARCHAR
            WHEN 'VIEW' THEN 'v'
            ELSE 'r'::VARCHAR
        END AS relkind,
        10::BIGINT AS relowner,
        0::BIGINT AS reltablespace,
        0::BIGINT AS relpages,
        0::BIGINT AS reltuples,
        NULL::VARCHAR[] AS relacl,
        NULL::VARCHAR[] AS reloptions,
        false::BOOLEAN AS relhasindex,
        false::BOOLEAN AS relhasrules,
        false::BOOLEAN AS relhastriggers,
        'p'::VARCHAR AS relpersistence,
        false::BOOLEAN AS relispartition,
        NULL::VARCHAR AS relpartbound,
        0::BIGINT AS reltype
    FROM information_schema.tables
    WHERE table_schema IN ('public', 'main')
) AS sub
;
