-- SPDX-License-Identifier: MPL-2.0
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 120),
    config TEXT NOT NULL CHECK(json_valid(config)),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    updated_at INTEGER NOT NULL
);
CREATE INDEX projects_by_name ON projects(name COLLATE NOCASE, id);
