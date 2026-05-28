CREATE OR REPLACE VIEW pg_compat.pg_index AS
SELECT 0::BIGINT AS oid, 0::BIGINT AS indexrelid, 0::BIGINT AS indrelid,
       0::SMALLINT AS indnatts, 0::SMALLINT AS indnkeyatts, false::BOOLEAN AS indisunique,
       false::BOOLEAN AS indisprimary, false::BOOLEAN AS indisclustered,
       false::BOOLEAN AS indnullsnotdistinct,
       NULL::BIGINT[] AS indkey, NULL::BIGINT[] AS indclass,
       NULL::BIGINT[] AS indcollation, NULL::VARCHAR[] AS indoption,
       NULL::VARCHAR AS indexprs, NULL::VARCHAR AS indpred,
       false::BOOLEAN AS indvalid LIMIT 0
;
