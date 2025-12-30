--------------------------------------------------
-- asset schema 全局 asset 资源表
CREATE TYPE asset_status AS ENUM (
    'init',
    'uploading',
    'available',
    'failed',
    'aborted',
    'deleted'
);

CREATE TYPE owner_type AS ENUM (
    'user',
    'article',
    'question',

    -- 所有的 owner_type
    'any'
);

CREATE TABLE asset(
    "id"            uuid            PRIMARY KEY,
    "newest_key"    TEXT            NOT NULL,
    "status"        asset_status    NOT NULL        DEFAULT 'init',

    "owner"         uuid            NOT NULL,
    "owner_type"    owner_type     NOT NULL,

    "history"       TEXT[]          NOT NULL        DEFAULT ARRAY[]::TEXT[],

    "created_at"    timestamptz     NOT NULL        DEFAULT now(),
    "updated_at"    timestamptz     NOT NULL        DEFAULT now(),
    "deleted_at"    timestamptz                     DEFAULT NULL
);

---------------------------------------------------
-- user schema，USER 的一切信息都在这个 schema 里面
CREATE SCHEMA "user" AUTHORIZATION postgres;

CREATE TABLE "user"."user_info" (
    "id"            UUID            PRIMARY KEY,
    "name"          VARCHAR(32)     NOT NULL,
    "email"         VARCHAR(256)    UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone"         VARCHAR(16)     UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "password_hash"   TEXT            NOT NULL,

    CONSTRAINT login_method CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX btree_index_name ON "user"."user_info" USING btree (LOWER(name));
CREATE INDEX btree_index_email ON "user"."user_info" USING btree (LOWER(email));
CREATE INDEX btree_index_phone ON "user"."user_info" USING btree (phone);

CREATE TABLE "user"."user_profile" (
    "user_id"       UUID          PRIMARY KEY REFERENCES "user"."user_info"(id) ON DELETE CASCADE,
    "biography"     UUID            NOT NULL REFERENCES "asset",
    "avatar"        UUID            NOT NULL REFERENCES "asset",
    "banner"        UUID            NOT NULL REFERENCES "asset",
    "contact_me"    JSONB           NOT NULL DEFAULT '[]'::JSONB,
    "updated_at"    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TRIGGER "auto_insert_userprofile"
AFTER INSERT ON "user"."user_info"
FOR EACH ROW
BEGIN
    INSERT INTO "user"."user_profile" ("user_id") VALUE NEW."id";
END
