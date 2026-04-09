DROP TYPE IF EXISTS "asset_status" CASCADE;
DROP TABLE IF EXISTS "asset" CASCADE;
DROP SCHEMA IF EXISTS "user" CASCADE;
DROP TABLE IF EXISTS "article" CASCADE;
DROP TABLE IF EXISTS "article_asset" CASCADE;
DROP TYPE IF EXISTS "comment_status" CASCADE;
DROP TABLE IF EXISTS "comment" CASCADE;

-- ASSET
CREATE TYPE "asset_status" AS ENUM (
    'init',
    'uploading',
    'available',
    'failed',
    'aborted',
    'deleted'
);

CREATE TABLE "asset"(
    "id"            UUID            PRIMARY KEY     DEFAULT uuidv7(),
    "status"        "asset_status" 	NOT NULL        DEFAULT 'init',

    -- 表示拥有者，这个拥有者是上传这个 asset 的用户
    "owner"         UUID            NOT NULL,

    "created_at"    TIMESTAMPTZ     NOT NULL        DEFAULT now(),
    "updated_at"    TIMESTAMPTZ     NOT NULL        DEFAULT now(),
    "deleted_at"    TIMESTAMPTZ                     DEFAULT NULL
);
-- 用于加速查询 "谁有哪些未删除的 asset"
CREATE INDEX "idx_asset_owner" ON "asset"("owner") WHERE "deleted_at" IS NOT NULL;
CREATE INDEX "idx_asset_status" ON "asset"("status");

-- 用户
CREATE SCHEMA "user" AUTHORIZATION postgres;

CREATE TABLE "user"."user_info" (
    "id"            UUID            PRIMARY KEY     DEFAULT uuidv7(),
    "name"          VARCHAR(32)     NOT NULL,
    "email"         VARCHAR(256)    UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone"         VARCHAR(16)     UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "password_hash" TEXT            NOT NULL,

    CONSTRAINT "login_method" CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX "btree_index_name" ON "user"."user_info" USING btree (LOWER("name"));
CREATE INDEX "btree_index_email" ON "user"."user_info" USING btree (LOWER("email"));
CREATE INDEX "btree_index_phone" ON "user"."user_info" USING btree ("phone");

CREATE TABLE "user"."user_profile" (
    "user_id"       UUID           	PRIMARY KEY REFERENCES "user"."user_info"(id) ON DELETE CASCADE,
    "biography"     UUID            REFERENCES "asset" DEFAULT NULL,
    "avatar"        UUID            REFERENCES "asset" DEFAULT NULL,
    "banner"        UUID            REFERENCES "asset" DEFAULT NULL,
    "contact_me"    JSONB           NOT NULL DEFAULT '[]'::JSONB,
    "updated_at"    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- 文章属于一种 asset
CREATE TABLE "article" (
    "id"              UUID          PRIMARY KEY   REFERENCES "asset"(id) ON DELETE CASCADE,
    "title"           TEXT          NOT NULL,
    "published_at"    TIMESTAMPTZ,                  -- 发布时间，NULL 表示草稿
    "likes"           BIGINT        NOT NULL      DEFAULT 0,
    "views"           BIGINT        NOT NULL      DEFAULT 0,
    "tags"            TEXT[]        NOT NULL      DEFAULT ARRAY[]::TEXT[]
);
CREATE INDEX "idx_article_published_at" ON "article"("published_at");
-- GIN 索引
CREATE INDEX "idx_article_tags" ON "article" USING gin("tags");


-- 文章-资产关联表
CREATE TABLE "article_asset" (
    "article_id"      UUID          NOT NULL,
    "asset_id"        UUID          NOT NULL,
    "position"        INT           NOT NULL    DEFAULT 0,   -- 排序顺序
    "role"            VARCHAR(32)   NOT NULL    DEFAULT 'inline', -- 'cover', 'inline', 'attachment' 等

    PRIMARY KEY ("article_id", "asset_id"),
    CONSTRAINT "fk_article_asset_article" FOREIGN KEY ("article_id")
		REFERENCES "article"("id") ON DELETE CASCADE,

	-- 如果一个 asset 被另外的 asset 引用, 此处是文章, 我们拒绝删除
    CONSTRAINT "fk_article_asset_asset" FOREIGN KEY ("asset_id")
		REFERENCES "asset"("id") ON DELETE RESTRICT
);

CREATE INDEX "idx_article_asset_asset_id" ON "article_asset"("asset_id");
CREATE INDEX "idx_article_asset_role" ON "article_asset"("role");


-- 评论
CREATE TYPE "comment_status" AS ENUM ('active', 'hidden', 'deleted');

CREATE TABLE "comment" (
    "id"                 UUID            	PRIMARY KEY REFERENCES "asset"(id) ON DELETE CASCADE,
    "article_id"         UUID            	NOT NULL,
    "parent_comment_id"  UUID            	DEFAULT NULL,
    "content"            TEXT            	NOT NULL,
    "likes"              INT             	NOT NULL DEFAULT 0,
    "status"             "comment_status"  	NOT NULL DEFAULT 'active',

	CONSTRAINT "fk_article_id" FOREIGN KEY ("article_id")
		REFERENCES "article"("id") ON DELETE CASCADE,

	CONSTRAINT "fk_parent_comment_id" FOREIGN KEY ("parent_comment_id")
		REFERENCES "comment"("id") ON DELETE CASCADE,

    -- 防止评论指向自身
    CONSTRAINT "check_parent_not_self" CHECK ("parent_comment_id" <> id)
);

CREATE INDEX "idx_comment_article_id" ON "comment"("article_id");
CREATE INDEX "idx_comment_parent_id" ON "comment"("parent_comment_id");
