CREATE TABLE branches (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK(length(trim(code)) BETWEEN 1 AND 32),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 160),
    phone TEXT NOT NULL DEFAULT '' CHECK(length(phone) <= 64),
    address TEXT NOT NULL DEFAULT '' CHECK(length(address) <= 500),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
