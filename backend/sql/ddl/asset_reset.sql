DROP TABLE asset CASCADE;
DROP TYPE asset_status CASCADE;
DROP TYPE owner_type CASCADE;

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
    'question'
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