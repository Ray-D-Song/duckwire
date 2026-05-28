CREATE OR REPLACE VIEW pg_compat.pg_policies AS
SELECT ''::VARCHAR AS schemaname, ''::VARCHAR AS policyname,
       ''::VARCHAR AS tablename, ['PERMISSIVE']::VARCHAR[] AS permissive,
       ['public']::VARCHAR[] AS roles, ['ALL']::VARCHAR[] AS cmd,
       ''::VARCHAR AS qual, ''::VARCHAR AS with_check LIMIT 0
;
