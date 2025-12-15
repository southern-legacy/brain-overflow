DROP TABLE asset;

CREATE TABLE asset(
    "id"            UUID        PRIMARY KEY,
    "newest_key"    TEXT        NOT NULL,
    "history"       TEXT[]      NOT NULL        DEFAULT '{}',
    "deleted_at"    TIMESTAMPTZ                 DEFAULT NULL
);