CREATE OR REPLACE VIEW pg_compat.pg_constraint AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS conname, 'c'::VARCHAR AS contype,
       0::BOOLEAN AS condeferrable, 0::BOOLEAN AS condeferred,
       0::BIGINT AS conrelid, NULL::BIGINT[] AS conkey,
       0::BIGINT AS confrelid, NULL::BIGINT[] AS confkey,
       NULL::BIGINT[] AS conexclop,
       'a'::VARCHAR AS confupdtype, 'a'::VARCHAR AS confdeltype,
       's'::VARCHAR AS confmatchtype, 0::BIGINT AS conindid,
       0::BIGINT AS connamespace, 0::BOOLEAN AS connoinherit
LIMIT 0
;
