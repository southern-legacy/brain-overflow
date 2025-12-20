INSERT INTO asset(
    "id",
    "newest_key",
    "status",

    "owner",
    "owner_type",

    "history",

    "created_at",
    "updated_at",
    "deleted_at"
) VALUES (
    '019b3a9f-de62-7453-b409-e9215689f19f'::uuid,
    'overflow/019b3a9f-de62-7453-b409-e9215689f19f',
    'available',
    '019b3a9f-de62-7453-b409-e9215689f19f',
    'user',
    ARRAY ['overflow/019b3a9f-de62-7453-b409-e9215689f19f'],
    '2024-01-22 00:02:48'::timestamp,
    '2024-01-22 00:02:48'::timestamp,
    '2024-01-22 00:02:48'::timestamp
);

DELETE FROM asset WHERE true;