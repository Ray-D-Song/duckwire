CREATE OR REPLACE VIEW pg_compat.pg_index AS
SELECT 0::BIGINT AS indexrelid, 0::BIGINT AS indrelid,
       0::SMALLINT AS indnatts, false::BOOLEAN AS indisunique,
       false::BOOLEAN AS indisprimary, false::BOOLEAN AS indisclustered,
       NULL::BIGINT[] AS indkey, false::BOOLEAN AS indvalid LIMIT 0
;
