CREATE OR REPLACE VIEW pg_compat.pg_event_trigger AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS evtname,
       ''::VARCHAR AS evtevent, 0::BIGINT AS evtowner, 0::BIGINT AS evtfoid,
       'D'::VARCHAR AS evtenabled, NULL::VARCHAR[] AS evttags LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_foreign_data_wrapper AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS fdwname,
       10::BIGINT AS fdwowner, 0::BIGINT AS fdwhandler, 0::BIGINT AS fdwvalidator,
       NULL::VARCHAR[] AS fdwacl, NULL::VARCHAR[] AS fdwoptions LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_user_mappings AS
SELECT 0::BIGINT AS umid, 0::BIGINT AS srvid, 0::BIGINT AS umuser,
       ''::VARCHAR AS usename, NULL::VARCHAR[] AS umoptions LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_cast AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, 0::BIGINT AS castsource,
       0::BIGINT AS casttarget, 0::BIGINT AS castfunc,
       'i'::VARCHAR AS castcontext, 'f'::VARCHAR AS castmethod LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_sequence AS
SELECT 0::BIGINT AS seqrelid, 0::BIGINT AS seqtypid,
       1::BIGINT AS seqstart, 1::BIGINT AS seqincrement,
       1::BIGINT AS seqmax, 1::BIGINT AS seqmin,
       1::BIGINT AS seqcache, false::BOOLEAN AS seqcycle LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_aggregate AS
SELECT 0::BIGINT AS aggfnoid, 'n'::VARCHAR AS aggkind,
       0::INTEGER AS aggnumdirectargs, 0::BIGINT AS aggtransfn,
       0::BIGINT AS aggfinalfn, 0::BIGINT AS aggcombinefn,
       0::BIGINT AS aggserialfn, 0::BIGINT AS aggdeserialfn,
       0::BIGINT AS aggmtransfn, 0::BIGINT AS aggminvtransfn,
       0::BIGINT AS aggmfinalfn, false::BOOLEAN AS aggfinalextra,
       false::BOOLEAN AS aggmfinalextra, 'r'::VARCHAR AS aggfinalmodify,
       'r'::VARCHAR AS aggmfinalmodify, 0::BIGINT AS aggsortop,
       0::BIGINT AS aggtranstype, 0::BIGINT AS aggtransspace,
       0::BIGINT AS aggmtranstype, 0::BIGINT AS aggmtransspace,
       NULL::VARCHAR AS agginitval, NULL::VARCHAR AS aggminitval LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_opfamily AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, 0::BIGINT AS opfmethod,
       ''::VARCHAR AS opfname, 0::BIGINT AS opfnamespace,
       10::BIGINT AS opfowner LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_amop AS
SELECT 0::BIGINT AS oid, 0::BIGINT AS amopfamily, 0::BIGINT AS amoplefttype,
       0::BIGINT AS amoprighttype, 0::SMALLINT AS amopstrategy,
       's'::VARCHAR AS amoppurpose, 0::BIGINT AS amopopr,
       0::BIGINT AS amopmethod, 0::BIGINT AS amopsortfamily LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_amproc AS
SELECT 0::BIGINT AS oid, 0::BIGINT AS amprocfamily, 0::BIGINT AS amproclefttype,
       0::BIGINT AS amprocrighttype, 0::SMALLINT AS amprocnum,
       0::BIGINT AS amproc, '0'::VARCHAR AS amprocedure LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_rewrite AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS rulename,
       0::BIGINT AS ev_class, '1'::VARCHAR AS ev_type,
       false::BOOLEAN AS ev_enabled, false::BOOLEAN AS is_instead,
       NULL::VARCHAR AS ev_qual, NULL::VARCHAR AS ev_action LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_policy AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS polname,
       0::BIGINT AS polrelid, '*'::VARCHAR AS polcmd,
       false::BOOLEAN AS polpermissive, NULL::BIGINT[] AS polroles,
       NULL::VARCHAR AS polqual, NULL::VARCHAR AS polwithcheck LIMIT 0
;

CREATE OR REPLACE VIEW pg_compat.pg_trigger AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, 0::BIGINT AS tgrelid,
       0::BIGINT AS tgparentid, ''::VARCHAR AS tgname,
       0::BIGINT AS tgfoid, 0::SMALLINT AS tgtype,
       true::BOOLEAN AS tgenabled, false::BOOLEAN AS tgisinternal,
       0::BIGINT AS tgconstrrelid, 0::BIGINT AS tgconstrindid,
       0::BIGINT AS tgconstraint, false::BOOLEAN AS tgdeferrable,
       false::BOOLEAN AS tginitdeferred, 0::SMALLINT AS tgnargs,
       NULL::SMALLINT[] AS tgattr, NULL::BYTEA AS tgargs,
       NULL::VARCHAR AS tgqual, NULL::VARCHAR AS tgoldtable,
       NULL::VARCHAR AS tgnewtable LIMIT 0
;
