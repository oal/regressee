-- SPDX-License-Identifier: MPL-2.0
-- Rebuild both sides of the CAS foreign key without disabling enforcement.
-- Historical evidence JSON is deliberately never rewritten.
CREATE TABLE artifacts_v10 (
    hash TEXT PRIMARY KEY NOT NULL CHECK(length(hash) = 64 AND hash NOT GLOB '*[^0-9a-f]*'),
    bytes INTEGER NOT NULL CHECK(bytes > 0 AND bytes <= 134217728),
    kind TEXT NOT NULL CHECK(kind IN ('Screenshot', 'Thumbnail', 'Tile', 'Difference', 'DomSnapshot'))
);
INSERT INTO artifacts_v10 SELECT * FROM artifacts;
CREATE TABLE artifact_refs_v10 (
    owner_kind TEXT NOT NULL CHECK(owner_kind IN ('Run', 'Baseline', 'Analysis', 'DomAnalysis')),
    owner_id TEXT NOT NULL,
    hash TEXT NOT NULL REFERENCES artifacts_v10(hash) ON DELETE RESTRICT,
    PRIMARY KEY(owner_kind, owner_id, hash)
);
INSERT INTO artifact_refs_v10 SELECT * FROM artifact_refs;
DROP TABLE artifact_refs;
DROP TABLE artifacts;
ALTER TABLE artifacts_v10 RENAME TO artifacts;
ALTER TABLE artifact_refs_v10 RENAME TO artifact_refs;
CREATE INDEX artifact_refs_by_hash ON artifact_refs(hash);
CREATE TRIGGER artifacts_immutable BEFORE UPDATE ON artifacts
BEGIN SELECT RAISE(ABORT, 'artifacts are immutable'); END;

-- Separate kind identity, immutable revisions, independent display/review state.
CREATE TABLE dom_analyses (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id),
    baseline_id TEXT NOT NULL REFERENCES baselines(id),
    record TEXT NOT NULL CHECK(length(CAST(record AS BLOB)) <= 67108864)
);
CREATE INDEX dom_analyses_latest ON dom_analyses(run_id);
CREATE TRIGGER dom_analyses_immutable BEFORE UPDATE ON dom_analyses
BEGIN SELECT RAISE(ABORT, 'DOM analyses are immutable'); END;
CREATE TRIGGER dom_analysis_distinct BEFORE INSERT ON dom_analyses
WHEN EXISTS(SELECT 1 FROM analyses WHERE id=NEW.id)
BEGIN SELECT RAISE(ABORT, 'analysis kind identity conflicts'); END;
CREATE TRIGGER image_analysis_distinct BEFORE INSERT ON analyses
WHEN EXISTS(SELECT 1 FROM dom_analyses WHERE id=NEW.id)
BEGIN SELECT RAISE(ABORT, 'analysis kind identity conflicts'); END;
CREATE TABLE dom_analysis_display (
    run_id TEXT PRIMARY KEY REFERENCES runs(id),
    analysis_id TEXT NOT NULL REFERENCES dom_analyses(id),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0)
);
CREATE TABLE dom_review_decisions (
    id INTEGER PRIMARY KEY,
    analysis_id TEXT NOT NULL REFERENCES dom_analyses(id),
    record TEXT NOT NULL
);
CREATE INDEX dom_reviews_analysis ON dom_review_decisions(analysis_id,id);
-- Independent slot payload: a later sibling must not replace/discard a success.
ALTER TABLE run_target_deltas ADD COLUMN dom_capture TEXT;
