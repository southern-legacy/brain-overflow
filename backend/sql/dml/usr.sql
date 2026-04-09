-- 插入十万个用户
INSERT INTO "user"."user_info" (id, name, email, password_hash)
SELECT 
    uuidv7(), 
    'user_' || i, 
    'user' || i || '@example.com', 
    '$argon2i$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$iWh06vD8Fy27wf9npn6FXWiCX4K6pW6Ue1Bnzz07Z8A'
FROM generate_series(1, 100000) s(i);

-- 为每个用户生成 Profile
INSERT INTO "user"."user_profile" (user_id)
SELECT id FROM "user"."user_info";

-- n 是插入个数, l 是最少多少个标签, r 是最多多少个标签
CREATE OR REPLACE FUNCTION insert_mock_articles(n INT, l INT, r INT) RETURNS VOID AS $$
DECLARE
 	chars TEXT[] := ARRAY['a','b','c','d','e','f','g','h','i','j'];
    tag_pool TEXT[];                     -- 存储全部 1000 种标签
    i INT;
    tag_count INT;
    tags_array TEXT[];
    j INT;
    idx INT;
	user_ids UUID[];
	asset_ids UUID[];
BEGIN
    -- 预先取出所有 user.id（用于随机选择）
    SELECT array_agg(id) INTO user_ids FROM "user"."user_info";

	-- 插入一百万个基础资产
	INSERT INTO asset (key, owner, status)
	SELECT 
	    uuidv7(), 
		user_ids[random() * (array_length(user_ids, 1) - 1) + 1],
		'available'::asset_status
	FROM generate_series(1, 1000000) s(i);

    -- 预先取出 n 个 asset.id（按顺序）
    SELECT array_agg(id ORDER BY id) INTO asset_ids
    FROM (SELECT id FROM asset ORDER BY id LIMIT n) t;

	-- 构建 tag_pool
    FOR c1 IN 1..10 LOOP
        FOR c2 IN 1..10 LOOP
            FOR c3 IN 1..10 LOOP
                tag_pool := array_append(tag_pool, chars[c1] || chars[c2] || chars[c3]);
            END LOOP;
        END LOOP;
    END LOOP;

    FOR i IN 1..n LOOP
		tag_count := l + floor(random() * (r-l))::INT;   -- 随机 [l,r]
        tags_array := ARRAY[]::TEXT[];

        FOR j IN 1..tag_count LOOP
            idx := 1 + floor(random() * array_length(tag_pool,1))::INT;   -- 从池中随机选一个标签
            tags_array := array_append(tags_array, tag_pool[idx]);
        END LOOP;
	
        -- 直接使用预取的 asset_id
        INSERT INTO article (id, title, tags)
        VALUES (
            asset_ids[i],
            'Title of article ' || i,
            tags_array
        );
    END LOOP;
END;
$$ LANGUAGE plpgsql;

SELECT insert_mock_articles(500000,3,20);

-- 为前 500 篇文章生成大量评论（模拟热帖）
DO $$
DECLARE
    target_article_id UUID;
BEGIN
    FOR target_article_id IN (SELECT id FROM article LIMIT 500) LOOP
        -- 每篇热帖插入 200-500 条评论
        INSERT INTO "comment" (id, article_id, author_id, content, parent_comment_id, created_at)
        SELECT 
            uuidv7(),
            target_article_id,
            (SELECT id FROM "user"."user_info" ORDER BY random() LIMIT 1),
            'This is a comment number ' || c,
            NULL, -- 先生成一级评论
            now() - (random() * interval '30 days')
        FROM generate_series(1, (random() * 300 + 200)::int) c;
        
        -- 为其中的一部分评论生成回复（二级评论）
        INSERT INTO "comment" (id, article_id, author_id, content, parent_comment_id)
        SELECT 
            uuidv7(),
            target_article_id,
            (SELECT id FROM "user"."user_info" ORDER BY random() LIMIT 1),
            'Reply to comment',
            (SELECT id FROM "comment" WHERE article_id = target_article_id AND parent_comment_id IS NULL ORDER BY random() LIMIT 1)
        FROM generate_series(1, 100);
    END LOOP;
END $$;
