CREATE OR REPLACE VIEW pg_compat.pg_attrdef AS
SELECT 0::BIGINT AS oid, 0::BIGINT AS adrelid, 0::INTEGER AS adnum,
       ''::VARCHAR AS adbin, ''::VARCHAR AS adsrc LIMIT 0
;
