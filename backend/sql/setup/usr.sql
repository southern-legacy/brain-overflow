CREATE DATABASE br_ovfl OWNER postgres;

CREATE SCHEMA "usr" AUTHORIZATION postgres;

CREATE TABLE "usr"."usr_info" (
    "id" SERIAL PRIMARY KEY,
    "name" CHAR(32) NOT NULL,
    "email" VARCHAR(256) UNIQUE CHECK ("email" ~* '^[\w\d._%+-]+@[\w\d.-]+\.[\w\d]{2,}$'),
    "phone" CHAR(16) UNIQUE CHECK ("phone" ~* '^\+\d{1,15}$'),
    "passwd_hash" TEXT NOT NULL,

    CONSTRAINT login_method CHECK (("email" IS NOT NULL) OR ("phone" IS NOT NULL))
);

CREATE INDEX btree_idx_name ON "usr"."usr_info" USING btree (LOWER(name));