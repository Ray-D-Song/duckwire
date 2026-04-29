CREATE OR REPLACE VIEW pg_compat.pg_shdescription AS
SELECT 0::BIGINT AS objoid, 0::BIGINT AS classoid,
       ''::VARCHAR AS description LIMIT 0
;
