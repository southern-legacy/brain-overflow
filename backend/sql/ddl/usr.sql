CREATE SCHEMA "user" AUTHORIZATION postgres;

CREATE TABLE "user"."user_info" (
    "id"            BIGSERIAL       PRIMARY KEY,
    "name"          VARCHAR(32)     NOT NULL,
    "email"         VARCHAR(256)    UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone"         VARCHAR(16)     UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "passwd_hash"   TEXT            NOT NULL,

    CONSTRAINT login_method CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX btree_idx_name ON "user"."user_info" USING btree (LOWER(name));

CREATE TABLE "user"."user_profile" (
    "user_id"        BIGINT          PRIMARY KEY REFERENCES "user"."user_info"(id) ON DELETE CASCADE,
    "biography"     TEXT            NOT NULL DEFAULT '# 默认用户说明（开发中）',
    "avatar"        TEXT            NOT NULL DEFAULT '默认头像图片路径（开发中）',
    "background"    TEXT            NOT NULL DEFAULT '默认背景图片路径（开发中）',
    "contact_me"    JSONB           NOT NULL DEFAULT '[]'::JSONB,
    "updated_at"    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);