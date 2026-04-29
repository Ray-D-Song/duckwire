CREATE OR REPLACE VIEW pg_compat.pg_foreign_table AS
SELECT 0::BIGINT AS ftrelid, 0::BIGINT AS ftserver,
       NULL::VARCHAR[] AS ftoptions LIMIT 0
;
