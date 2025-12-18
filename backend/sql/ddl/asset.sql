CREATE TYPE asset_status AS ENUM (
    'init',
    'uploading',
    'available',
    'failed',
    'aborted',
    'deleted'
);

CREATE TABLE asset(
    "id"            UUID        PRIMARY KEY,
    "newest_key"    TEXT        NOT NULL,
    "history"       TEXT[]      NOT NULL        DEFAULT '{}',
    "deleted_at"    TIMESTAMPTZ                 DEFAULT NULL
);