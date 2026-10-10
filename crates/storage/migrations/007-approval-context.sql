-- Approval previews are invalidated even when reanalysis returns a cached ID.
ALTER TABLE analysis_display ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
