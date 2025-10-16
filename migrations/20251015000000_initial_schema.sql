CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    role TEXT NOT NULL
);

CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    owner_id TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE policies (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL
);
