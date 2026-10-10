-- Imported snapshots are provenance, never independently verified local captures.
CREATE TABLE imported_origins (
 run_id TEXT PRIMARY KEY REFERENCES runs(id), imported_at INTEGER NOT NULL,
 archive_hash TEXT NOT NULL
);
-- Explicit deletion journal: a crash must not lose the user's GC authorization.
CREATE TABLE pending_gc (hash TEXT PRIMARY KEY NOT NULL);
