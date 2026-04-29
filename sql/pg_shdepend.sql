CREATE OR REPLACE VIEW pg_compat.pg_shdepend AS
SELECT 0::BIGINT AS dbid, 0::BIGINT AS classid, 0::BIGINT AS objid,
       0::INTEGER AS objsubid, 0::BIGINT AS refclassid,
       0::BIGINT AS refobjid, 'n'::VARCHAR AS deptype LIMIT 0
;
