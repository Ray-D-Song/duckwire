CREATE OR REPLACE VIEW pg_compat.pg_description AS
SELECT 0::BIGINT AS objoid, 0::BIGINT AS classoid, 0::INTEGER AS objsubid,
       ''::VARCHAR AS description LIMIT 0
;
