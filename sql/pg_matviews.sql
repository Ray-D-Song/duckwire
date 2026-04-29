CREATE OR REPLACE VIEW pg_compat.pg_matviews AS
SELECT ''::VARCHAR AS schemaname, ''::VARCHAR AS matviewname,
       ''::VARCHAR AS matviewowner, false::BOOLEAN AS ispopulated,
       ''::VARCHAR AS definition, NULL::VARCHAR AS tablespace LIMIT 0
;
