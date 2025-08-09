DROP SCHEMA usr CASCADE;

CREATE SCHEMA "usr" AUTHORIZATION postgres;

CREATE TABLE "usr"."usr_info" (
    "id" BIGSERIAL PRIMARY KEY,
    "name" VARCHAR(32) NOT NULL,
    "email" VARCHAR(256) UNIQUE CHECK ("email" ~* '^[\w._%+-]+@[\w.-]+\.\w{2,}$'),
    "phone" VARCHAR(16) UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "salt" CHAR(22) NOT NULL,
    "passwd_hash" TEXT NOT NULL,

    CONSTRAINT login_method CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX btree_idx_name ON "usr"."usr_info" USING btree (LOWER(name));