CREATE OR REPLACE MACRO pg_compat.age(x) AS 0;

CREATE OR REPLACE VIEW pg_compat.pg_locks AS
SELECT NULL::VARCHAR AS locktype,
       NULL::BIGINT AS database,
       NULL::BIGINT AS relation,
       NULL::INTEGER AS page,
       NULL::SMALLINT AS tuple,
       NULL::VARCHAR AS virtualxid,
       NULL::BIGINT AS transactionid,
       NULL::BIGINT AS classid,
       NULL::BIGINT AS objid,
       NULL::SMALLINT AS objsubid,
       NULL::VARCHAR AS virtualtransaction,
       NULL::INTEGER AS pid,
       NULL::VARCHAR AS mode,
       false::BOOLEAN AS granted,
       false::BOOLEAN AS fastpath,
       NULL::TIMESTAMP AS waitstart
LIMIT 0
;
