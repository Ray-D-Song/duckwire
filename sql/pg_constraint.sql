CREATE OR REPLACE VIEW pg_compat.pg_constraint AS
SELECT 0::BIGINT AS oid, ''::VARCHAR AS conname, ''::VARCHAR AS contype,
       0::BOOLEAN AS condeferrable, 0::BOOLEAN AS condeferred,
       0::BIGINT AS conrelid, NULL::BIGINT[] AS conkey,
       ''::VARCHAR AS confupdtype, ''::VARCHAR AS confdeltype,
       ''::VARCHAR AS confmatchtype, 0::BIGINT AS conindid,
       0::BIGINT AS connamespace, 0::BOOLEAN AS connoinherit
LIMIT 0
;
