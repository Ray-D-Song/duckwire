CREATE OR REPLACE VIEW pg_compat.pg_attribute AS
SELECT attrelid, attname, atttypid, attlen, attnum, attndims, atttypmod,
       attnotnull, attisdropped, attacl
FROM (
    SELECT
        t.oid AS attrelid,
        c.column_name AS attname,
        CASE UPPER(c.data_type)
            WHEN 'BIGINT' THEN 20::BIGINT
            WHEN 'INTEGER' THEN 23::BIGINT
            WHEN 'SMALLINT' THEN 21::BIGINT
            WHEN 'TINYINT' THEN 20::BIGINT
            WHEN 'BOOLEAN' THEN 16::BIGINT
            WHEN 'DOUBLE' THEN 701::BIGINT
            WHEN 'FLOAT' THEN 700::BIGINT
            WHEN 'REAL' THEN 700::BIGINT
            WHEN 'VARCHAR' THEN 1043::BIGINT
            WHEN 'CHARACTER VARYING' THEN 1043::BIGINT
            WHEN 'TEXT' THEN 25::BIGINT
            WHEN 'DATE' THEN 1082::BIGINT
            WHEN 'TIMESTAMP' THEN 1114::BIGINT
            WHEN 'TIMESTAMP WITHOUT TIME ZONE' THEN 1114::BIGINT
            WHEN 'TIMESTAMP WITH TIME ZONE' THEN 1184::BIGINT
            WHEN 'TIME' THEN 1083::BIGINT
            WHEN 'TIME WITHOUT TIME ZONE' THEN 1083::BIGINT
            WHEN 'TIME WITH TIME ZONE' THEN 1266::BIGINT
            WHEN 'INTERVAL' THEN 1186::BIGINT
            WHEN 'NUMERIC' THEN 1700::BIGINT
            WHEN 'DECIMAL' THEN 1700::BIGINT
            WHEN 'BLOB' THEN 17::BIGINT
            WHEN 'BYTEA' THEN 17::BIGINT
            WHEN 'JSON' THEN 114::BIGINT
            WHEN 'UUID' THEN 2950::BIGINT
            WHEN 'BIT' THEN 1560::BIGINT
            WHEN 'BPCHAR' THEN 1042::BIGINT
            WHEN 'CHAR' THEN 18::BIGINT
            ELSE 25::BIGINT
        END AS atttypid,
        -1::INTEGER AS attlen,
        c.ordinal_position::INTEGER AS attnum,
        0::INTEGER AS attndims,
        -1::INTEGER AS atttypmod,
        CASE WHEN c.is_nullable = 'YES' THEN false ELSE true END AS attnotnull,
        false::BOOLEAN AS attisdropped,
        NULL::VARCHAR[] AS attacl
    FROM information_schema.columns c
    JOIN (
        SELECT
            row_number() OVER (ORDER BY table_schema, table_name)::BIGINT + 16383 AS oid,
            table_name,
            table_schema
        FROM information_schema.tables
        WHERE table_schema IN ('public', 'main')
    ) t ON c.table_name = t.table_name AND c.table_schema = t.table_schema
) AS sub
;
