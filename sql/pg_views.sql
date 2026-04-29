CREATE OR REPLACE VIEW pg_compat.pg_views AS
SELECT schemaname::VARCHAR, viewname::VARCHAR, viewowner::VARCHAR,
       definition::VARCHAR
FROM (
    SELECT
        CASE WHEN table_schema = 'main' THEN 'public' ELSE table_schema END AS schemaname,
        table_name::VARCHAR AS viewname,
        'postgres'::VARCHAR AS viewowner,
        view_definition::VARCHAR AS definition
    FROM information_schema.views
    WHERE table_schema IN ('public', 'main')
) AS sub
;
