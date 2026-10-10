-- SPDX-License-Identifier: MPL-2.0
CREATE TABLE inventories (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    record TEXT NOT NULL
);
-- Retain every historical JSON byte, while expanding the bounded records for
-- whole-site snapshots. Child tables are rebuilt together under the migration's
-- transaction; no foreign-key disabling or evidence rewriting is required.
CREATE TABLE runs_v4 (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    state TEXT NOT NULL,
    record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 67108864),
    created_at INTEGER NOT NULL,
    owner_pid INTEGER NOT NULL
);
CREATE TABLE baselines_v4 (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    source_run TEXT NOT NULL REFERENCES runs_v4(id),
    record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 67108864)
);
CREATE TABLE analyses_v4 (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT UNIQUE NOT NULL REFERENCES runs_v4(id),
    baseline_id TEXT NOT NULL REFERENCES baselines_v4(id),
    record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 67108864)
);
INSERT INTO runs_v4 SELECT * FROM runs;
INSERT INTO baselines_v4 SELECT * FROM baselines;
INSERT INTO analyses_v4 SELECT * FROM analyses;
DROP TABLE analyses;
DROP TABLE baselines;
DROP TABLE runs;
ALTER TABLE runs_v4 RENAME TO runs;
ALTER TABLE baselines_v4 RENAME TO baselines;
ALTER TABLE analyses_v4 RENAME TO analyses;
CREATE INDEX runs_project ON runs(project_id, created_at);
