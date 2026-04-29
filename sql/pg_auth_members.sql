CREATE OR REPLACE VIEW pg_compat.pg_auth_members AS
SELECT 0::BIGINT AS roleid, 0::BIGINT AS member,
       0::BIGINT AS grantor, false::BOOLEAN AS admin_option LIMIT 0
;
