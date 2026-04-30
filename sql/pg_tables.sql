CREATE OR REPLACE VIEW pg_compat.pg_tables AS
SELECT schemaname AS table_schema, tablename AS table_name, tableowner,
       NULL::VARCHAR AS tablespace
FROM (
    SELECT
        CASE WHEN table_schema = 'main' THEN 'public' ELSE table_schema END AS schemaname,
        table_name::VARCHAR AS tablename,
        'postgres'::VARCHAR AS tableowner
    FROM information_schema.tables
    WHERE table_schema IN ('public', 'main')
) AS sub
;
