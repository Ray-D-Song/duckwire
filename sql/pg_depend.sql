CREATE OR REPLACE VIEW pg_compat.pg_depend AS
SELECT 0::BIGINT AS classid, 0::BIGINT AS objid, 0::INTEGER AS objsubid,
       0::BIGINT AS refclassid, 0::BIGINT AS refobjid,
       0::INTEGER AS refobjsubid, 'n'::VARCHAR AS deptype LIMIT 0
;
