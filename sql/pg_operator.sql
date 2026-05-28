CREATE OR REPLACE VIEW pg_compat.pg_operator AS
SELECT 0::BIGINT AS oid, 1::BIGINT AS xmin, ''::VARCHAR AS oprname,
       0::BIGINT AS oprnamespace, 10::BIGINT AS oprowner,
       'b'::VARCHAR AS oprkind, false::BOOLEAN AS oprcanmerge,
       false::BOOLEAN AS oprcanhash, 0::BIGINT AS oprleft,
       0::BIGINT AS oprright, 0::BIGINT AS oprresult,
       0::BIGINT AS oprcom, 0::BIGINT AS oprnegate,
       0::BIGINT AS oprcode, 0::BIGINT AS oprrest, 0::BIGINT AS oprjoin LIMIT 0
;
