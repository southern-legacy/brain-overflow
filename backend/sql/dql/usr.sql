-- 获取所有用户的基本认证相关信息
SELECT * FROM "user"."user_info";

-- 获取所有用户的完整信息
SELECT * FROM "user"."user_info", "user"."user_profile"
WHERE "user"."user_info"."id" = "user"."user_profile"."user_id";

-- 获取用户可展示的信息
SELECT * FROM "user"."user_profile";

-- 获取所有的 asset
SELECT * FROM "public"."asset";