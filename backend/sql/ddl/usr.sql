CREATE SCHEMA "user" AUTHORIZATION postgres;

CREATE TABLE "user"."user_info" (
    "id"            UUID       PRIMARY KEY,
    "name"          VARCHAR(32)     NOT NULL,
    "email"         VARCHAR(256)    UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone"         VARCHAR(16)     UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "passwd_hash"   TEXT            NOT NULL,

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