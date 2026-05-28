CREATE OR REPLACE VIEW pg_compat.pg_timezone_names AS
SELECT
    'UTC'::VARCHAR AS name,
    'UTC'::VARCHAR AS abbrev,
    INTERVAL '0 seconds' AS utc_offset,
    false::BOOLEAN AS is_dst
;

CREATE OR REPLACE VIEW pg_compat.pg_timezone_abbrevs AS
SELECT
    'UTC'::VARCHAR AS abbrev,
    INTERVAL '0 seconds' AS utc_offset,
    false::BOOLEAN AS is_dst
;
