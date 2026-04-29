CREATE OR REPLACE VIEW pg_compat.pg_inherits AS
SELECT 0::BIGINT AS inhrelid, 0::BIGINT AS inhparent,
       0::INTEGER AS inhseqno LIMIT 0
;
