-- SPDX-License-Identifier: MPL-2.0
CREATE TABLE runs (
 id TEXT PRIMARY KEY NOT NULL,
 project_id TEXT NOT NULL REFERENCES projects(id),
 state TEXT NOT NULL,
 record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 2097152),
 created_at INTEGER NOT NULL
 ,owner_pid INTEGER NOT NULL
);
CREATE INDEX runs_project ON runs(project_id, created_at);
CREATE TABLE baselines (
 id TEXT PRIMARY KEY NOT NULL,
 project_id TEXT NOT NULL REFERENCES projects(id),
 source_run TEXT NOT NULL REFERENCES runs(id),
 record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 2097152)
);
CREATE TABLE analyses (
 id TEXT PRIMARY KEY NOT NULL,
 run_id TEXT UNIQUE NOT NULL REFERENCES runs(id),
 baseline_id TEXT NOT NULL REFERENCES baselines(id),
 record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 2097152)
);
