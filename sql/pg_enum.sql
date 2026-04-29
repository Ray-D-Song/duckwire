CREATE OR REPLACE VIEW pg_compat.pg_enum AS
SELECT 0::BIGINT AS oid, 0::BIGINT AS enumtypid,
       0::FLOAT AS enumsortorder, ''::VARCHAR AS enumlabel LIMIT 0
;
