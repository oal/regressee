-- SPDX-License-Identifier: MPL-2.0
CREATE TABLE artifacts (
    hash TEXT PRIMARY KEY NOT NULL CHECK(length(hash) = 64 AND hash NOT GLOB '*[^0-9a-f]*'),
    bytes INTEGER NOT NULL CHECK(bytes > 0 AND bytes <= 134217728),
    kind TEXT NOT NULL CHECK(kind IN ('Screenshot', 'Thumbnail', 'Tile', 'Difference'))
);
CREATE TABLE artifact_refs (
    owner_kind TEXT NOT NULL CHECK(owner_kind IN ('Run', 'Baseline', 'Analysis')),
    owner_id TEXT NOT NULL,
    hash TEXT NOT NULL REFERENCES artifacts(hash) ON DELETE RESTRICT,
    PRIMARY KEY(owner_kind, owner_id, hash)
);
CREATE INDEX artifact_refs_by_hash ON artifact_refs(hash);
CREATE TRIGGER artifacts_immutable BEFORE UPDATE ON artifacts
BEGIN SELECT RAISE(ABORT, 'artifacts are immutable'); END;
