CREATE OR REPLACE VIEW pg_compat.pg_policies AS
SELECT ''::VARCHAR AS schemaname, ''::VARCHAR AS policyname,
       ''::VARCHAR AS tablename, ''::VARCHAR[] AS permissive,
       ''::VARCHAR[] AS roles, ''::VARCHAR[] AS cmd,
       ''::VARCHAR AS qual, ''::VARCHAR AS with_check LIMIT 0
;
