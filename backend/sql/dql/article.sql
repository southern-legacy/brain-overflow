-- 根据文章 id 查
explain analyze SELECT x.id, x.title, x.likes, x.views, x.tags, a.owner as author, a.created_at
FROM article AS x JOIN asset AS a
ON a.id = x.id AND a.status = 'available' AND a.deleted_at IS NULL
WHERE x.id = '019d71e5-9fd4-7dc0-92d1-6eb2039ce2bc';


-- 根据一个人的 id 查他所有的文章
explain analyze SELECT x.id, x.title, x.likes, x.views, x.tags, a.owner as author, a.created_at
FROM article AS x JOIN asset AS a 
ON a.owner = '019d71e5-9ccf-7cd4-a0d4-dfec57627512' AND a.status = 'available' AND a.id = x.id AND a.deleted_at IS NULL
ORDER BY a.created_at DESC
LIMIT 20 OFFSET 0;

-- 控制索引变量
create index "idx_asset_alive" on asset(owner, status) where deleted_at IS NULL;
drop index "idx_asset_alive";

-- 一个人发了 26 篇文章
-- 从总共的 50 万篇文章、10 万用户、100 万 asset 中
-- 没有索引会用 20-30ms 左右，加了索引最低可以达到 70us，最高 110 us
-- 主要是因为在未使用索引时在 asset 的 owner 列上进行了 sequential scan
-- 有索引前单点 250 并发，有索引后最终单点 5 万并发
-- (性能评估用的目测)

-- 获取发布文章最多的 100 个用户
SELECT count(A.id) AS "article_count", U.name, U.id, U.email, U.phone
FROM "user"."user_info" AS U JOIN "asset" AS A
ON U.id = A.owner
WHERE A.status = 'available' AND A.deleted_at IS NULL
GROUP BY U.id
ORDER BY count(A.id) DESC LIMIT 100;
