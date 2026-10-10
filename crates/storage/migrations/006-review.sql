-- Immutable post-capture analysis, geometric masks, and review history.
CREATE TABLE analyses_v6 (
 id TEXT PRIMARY KEY NOT NULL,
 run_id TEXT NOT NULL REFERENCES runs(id),
 baseline_id TEXT NOT NULL REFERENCES baselines(id),
 record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 67108864)
);
INSERT INTO analyses_v6 SELECT * FROM analyses;
DROP TABLE analyses;
ALTER TABLE analyses_v6 RENAME TO analyses;
CREATE INDEX analyses_latest ON analyses(run_id);
CREATE TABLE analysis_display (
 run_id TEXT PRIMARY KEY REFERENCES runs(id), analysis_id TEXT NOT NULL REFERENCES analyses(id)
);
CREATE TABLE ignore_masks (
 id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
 page_id TEXT NOT NULL, profile_id TEXT NOT NULL, record TEXT NOT NULL
);
CREATE TABLE review_decisions (
 id INTEGER PRIMARY KEY, analysis_id TEXT NOT NULL REFERENCES analyses(id),
 record TEXT NOT NULL
);
