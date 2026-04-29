-- Hardcoded Postgres type catalog entries (OID, typelen, typbyval, typalign, etc.)
-- required by JDBC/ODBC drivers and ORMs for schema introspection.
CREATE OR REPLACE VIEW pg_compat.pg_type AS
SELECT oid, typname, typnamespace, typtype, typelem, typrelid, typlen, typbyval,
       typacl, typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typalign, typstorage, typdefault, typdefaultbin, typcategory, typispreferred,
       typisdefined, typdelim, typnotnull, typbasetype, typtypmod, typndims, typcollation
FROM (VALUES
    (16::BIGINT, 'bool'::VARCHAR, 11::BIGINT, 'b'::VARCHAR, 0::BIGINT, 0::BIGINT, 1::INTEGER, true::BOOLEAN, NULL::VARCHAR[], 'boolin'::VARCHAR, 'boolout'::VARCHAR, 'boolrecv'::VARCHAR, 'boolsend'::VARCHAR, '-'::VARCHAR, '-'::VARCHAR, 'c'::VARCHAR, 'p'::VARCHAR, NULL::VARCHAR, NULL::VARCHAR, 'B'::VARCHAR, false::BOOLEAN, true::BOOLEAN, ','::VARCHAR, false::BOOLEAN, 0::BIGINT, -1::INTEGER, 0::INTEGER, 0::BIGINT),
    (21, 'int2', 11, 'b', 0, 0, 2, true, NULL, 'int2in', 'int2out', 'int2recv', 'int2send', '-', '-', 's', 'p', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (23, 'int4', 11, 'b', 0, 0, 4, true, NULL, 'int4in', 'int4out', 'int4recv', 'int4send', '-', '-', 'i', 'p', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (20, 'int8', 11, 'b', 0, 0, 8, false, NULL, 'int8in', 'int8out', 'int8recv', 'int8send', '-', '-', 'd', 'p', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (700, 'float4', 11, 'b', 0, 0, 4, true, NULL, 'float4in', 'float4out', 'float4recv', 'float4send', '-', '-', 'i', 'p', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (701, 'float8', 11, 'b', 0, 0, 8, false, NULL, 'float8in', 'float8out', 'float8recv', 'float8send', '-', '-', 'd', 'p', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (25, 'text', 11, 'b', 0, 0, -1, false, NULL, 'textin', 'textout', 'textrecv', 'textsend', '-', '-', 'i', 'x', NULL, NULL, 'S', true, true, ',', false, 0, -1, 0, 100),
    (1043, 'varchar', 11, 'b', 0, 0, -1, false, NULL, 'varcharin', 'varcharout', 'varcharrecv', 'varcharsend', 'varchartypmodin', 'varchartypmodout', 'i', 'x', NULL, NULL, 'S', false, true, ',', false, 0, -1, 0, 100),
    (1082, 'date', 11, 'b', 0, 0, 4, true, NULL, 'date_in', 'date_out', 'date_recv', 'date_send', '-', '-', 'i', 'p', NULL, NULL, 'D', false, true, ',', false, 0, -1, 0, 0),
    (1114, 'timestamp', 11, 'b', 0, 0, 8, false, NULL, 'timestamp_in', 'timestamp_out', 'timestamp_recv', 'timestamp_send', '-', '-', 'd', 'p', NULL, NULL, 'D', false, true, ',', false, 0, -1, 0, 0),
    (1700, 'numeric', 11, 'b', 0, 0, -1, false, NULL, 'numeric_in', 'numeric_out', 'numeric_recv', 'numeric_send', 'numerictypmodin', 'numerictypmodout', 'i', 'm', NULL, NULL, 'N', false, true, ',', false, 0, -1, 0, 0),
    (17, 'bytea', 11, 'b', 0, 0, -1, false, NULL, 'byteain', 'byteaout', 'bytearecv', 'byteasend', '-', '-', 'i', 'x', NULL, NULL, 'U', false, true, ',', false, 0, -1, 0, 0),
    (1186, 'interval', 11, 'b', 0, 0, 16, false, NULL, 'interval_in', 'interval_out', 'interval_recv', 'interval_send', '-', '-', 'd', 'p', NULL, NULL, 'T', false, true, ',', false, 0, -1, 0, 0),
    (114, 'json', 11, 'b', 0, 0, -1, false, NULL, 'json_in', 'json_out', 'json_recv', 'json_send', '-', '-', 'i', 'x', NULL, NULL, 'U', false, true, ',', false, 0, -1, 0, 0),
    (3802, 'jsonb', 11, 'b', 0, 0, -1, false, NULL, 'jsonb_in', 'jsonb_out', 'jsonb_recv', 'jsonb_send', '-', '-', 'i', 'x', NULL, NULL, 'U', false, true, ',', false, 0, -1, 0, 0),
    (19, 'name', 11, 'b', 0, 0, 64, false, NULL, 'namein', 'nameout', 'namerecv', 'namesend', '-', '-', 'c', 'p', NULL, NULL, 'S', false, true, ',', false, 0, -1, 0, 100),
    (18, 'char', 11, 'b', 0, 0, 1, true, NULL, 'charin', 'charout', 'charrecv', 'charsend', '-', '-', 'c', 'p', NULL, NULL, 'S', false, true, ',', false, 0, -1, 0, 0),
    (1042, 'bpchar', 11, 'b', 0, 0, -1, false, NULL, 'bpcharin', 'bpcharout', 'bpcharrecv', 'bpcharsend', 'bpchartypmodin', 'bpchartypmodout', 'i', 'x', NULL, NULL, 'S', false, true, ',', false, 0, -1, 0, 100),
    (29, 'cidr', 11, 'b', 0, 0, -1, false, NULL, 'cidr_in', 'cidr_out', 'cidr_recv', 'cidr_send', '-', '-', 'i', 'p', NULL, NULL, 'U', false, true, ',', false, 0, -1, 0, 0),
    (650, 'inet', 11, 'b', 0, 0, -1, false, NULL, 'inet_in', 'inet_out', 'inet_recv', 'inet_send', '-', '-', 'i', 'p', NULL, NULL, 'I', false, true, ',', false, 0, -1, 0, 0),
    (1083, 'time', 11, 'b', 0, 0, 8, false, NULL, 'time_in', 'time_out', 'time_recv', 'time_send', '-', '-', 'd', 'p', NULL, NULL, 'D', false, true, ',', false, 0, -1, 0, 0),
    (1266, 'timetz', 11, 'b', 0, 0, 12, false, NULL, 'timetz_in', 'timetz_out', 'timetz_recv', 'timetz_send', '-', '-', 'd', 'p', NULL, NULL, 'D', false, true, ',', false, 0, -1, 0, 0),
    (1184, 'timestamptz', 11, 'b', 0, 0, 8, false, NULL, 'timestamptz_in', 'timestamptz_out', 'timestamptz_recv', 'timestamptz_send', '-', '-', 'd', 'p', NULL, NULL, 'D', false, true, ',', false, 0, -1, 0, 0),
    (705, 'unknown', 11, 'b', 0, 0, -2, false, NULL, 'unknownin', 'unknownout', 'unknownrecv', 'unknownsend', '-', '-', 'c', 'p', NULL, NULL, 'X', false, false, ',', false, 0, -1, 0, 0)
) AS t(oid, typname, typnamespace, typtype, typelem, typrelid, typlen, typbyval,
       typacl, typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typalign, typstorage, typdefault, typdefaultbin, typcategory, typispreferred,
       typisdefined, typdelim, typnotnull, typbasetype, typtypmod, typndims, typcollation)
;
