-- SPDX-License-Identifier: MPL-2.0
-- Historical JSON is unchanged. Active queues overlay bounded target rows;
-- compact only at explicit pause/resume/finish boundaries.
CREATE TABLE run_checkpoint_heads (
    run_id TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK(revision >= 0)
);
CREATE TABLE run_target_deltas (
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    target_index INTEGER NOT NULL CHECK(target_index >= 0),
    sequence INTEGER NOT NULL CHECK(sequence >= 0),
    target TEXT NOT NULL,
    capture TEXT,
    PRIMARY KEY(run_id, target_index)
);
