-- 获取所有用户的基本认证相关信息
SELECT * FROM "usr"."usr_info";

-- 获取所有用户的完整信息
SELECT * FROM "usr"."usr_info", "usr"."user_profiles"
WHERE "usr"."usr_info"."id" = "usr"."user_profiles"."usr_id";

-- 获取用户可展示的信息
SELECT
    "base"."id", "base"."name", "base"."email", "base"."phone",
    "prof"."biography", "prof"."avator", "prof"."contact_me", "prof"."updated_at"
FROM "usr"."usr_info" "base", "usr"."user_profiles" "prof"
WHERE "base"."id" = "prof"."usr_id";