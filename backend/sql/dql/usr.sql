-- 获取所有用户的基本认证相关信息
SELECT * FROM "usr"."usr_info";

-- 获取所有用户的完整信息
SELECT * FROM "usr"."usr_info", "usr"."usr_profile"
WHERE "usr"."usr_info"."id" = "usr"."usr_profile"."usr_id";

-- 获取用户可展示的信息
SELECT * FROM "usr"."usr_profile";