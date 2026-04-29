CREATE OR REPLACE VIEW pg_compat.parameters AS
SELECT ''::VARCHAR AS specific_catalog, ''::VARCHAR AS specific_schema, ''::VARCHAR AS specific_name,
       0::INTEGER AS ordinal_position, 'IN'::VARCHAR AS parameter_mode, 'NO'::VARCHAR AS is_result,
       'NO'::VARCHAR AS as_locators, NULL::VARCHAR AS parameter_name, 'INTEGER'::VARCHAR AS data_type,
       NULL::INTEGER AS character_maximum_length, NULL::INTEGER AS character_octet_length,
       NULL::VARCHAR AS character_set_catalog, NULL::VARCHAR AS character_set_schema,
       NULL::VARCHAR AS character_set_name, NULL::VARCHAR AS collation_catalog,
       NULL::VARCHAR AS collation_schema, NULL::VARCHAR AS collation_name,
       NULL::INTEGER AS numeric_precision, NULL::INTEGER AS numeric_precision_radix,
       NULL::INTEGER AS numeric_scale, NULL::INTEGER AS datetime_precision,
       NULL::VARCHAR AS interval_type, NULL::INTEGER AS interval_precision,
       NULL::INTEGER AS maximum_cardinality, NULL::VARCHAR AS parameter_default,
       '1'::VARCHAR AS dtd_identifier, 'int4'::VARCHAR AS udt_name,
       NULL::VARCHAR AS scope_catalog, NULL::VARCHAR AS scope_schema, NULL::VARCHAR AS scope_name LIMIT 0
;
