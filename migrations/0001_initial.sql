-- Rivet initial schema. Applied migrations are immutable; add a new numbered file for changes.
-- Rust validation is authoritative (Unicode trimming); these checks are a last line of defense.
-- SQLite length() counts characters, matching Rust's Unicode scalar value count.

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE CHECK (length(email) BETWEEN 3 AND 254),
    password_hash TEXT NOT NULL CHECK (password_hash <> ''),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at)
) STRICT;

CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100 AND trim(name) <> ''),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 10000),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at)
) STRICT;

CREATE INDEX projects_owner_created ON projects (owner_id, created_at, id);

CREATE TABLE issues (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200 AND trim(title) <> ''),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 10000),
    status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'done')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL CHECK (updated_at >= created_at)
) STRICT;

CREATE INDEX issues_project_created ON issues (project_id, created_at, id);
