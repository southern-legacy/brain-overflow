DROP SCHEMA "usr" CASCADE;

CREATE SCHEMA "usr" AUTHORIZATION postgres;

CREATE TABLE "usr"."usr_info" (
    "id"            BIGSERIAL       PRIMARY KEY,
    "name"          VARCHAR(32)     NOT NULL,
    "email"         VARCHAR(256)    UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone"         VARCHAR(16)     UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "salt"          CHAR(22)        NOT NULL,
    "passwd_hash"   TEXT NOT        NULL,

    CONSTRAINT login_method CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX btree_idx_name ON "usr"."usr_info" USING btree (LOWER(name));

CREATE TABLE "usr"."user_profiles" (
    "usr_id"        INT             PRIMARY KEY REFERENCES "usr"."usr_info"(id),
    "biography"     TEXT            DEFAULT '# 默认用户说明',
    "avator"        TEXT            DEFAULT '默认头像图片路径（开发中）',
    "background"    TEXT            DEFAULT '默认背景图片路径（开发中）',
    "updated_at"    TIMESTAMPTZ     DEFAULT NOW()
);