// SPDX-License-Identifier: MPL-2.0
//! Structural aggregate only. No project name, URL, row/session ID or raw record.
use super::*;
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCounts {
    pub projects: u64,
    pub runs: u64,
    pub baselines: u64,
    pub artifacts: u64,
    pub sampled_targets: u64,
    pub captured: u64,
    pub failed: u64,
    pub skipped: u64,
    pub cancelled: u64,
    pub pending: u64,
    pub unrecognized: u64,
}
impl Store {
    pub fn diagnostic_counts(&self) -> Result<DiagnosticCounts> {
        // Bounded observations rather than a database/log export. Only the most
        // recent 100 runs / 10,000 targets contribute; report this in the preview.
        let count = |table: &str| -> Result<u64> {
            Ok(self.connection.query_row(
                &format!("SELECT count(*) FROM (SELECT 1 FROM {table} LIMIT 100001)"),
                [],
                |r| r.get(0),
            )?)
        };
        let mut counts = DiagnosticCounts {
            projects: count("projects")?,
            runs: count("runs")?,
            baselines: count("baselines")?,
            artifacts: count("artifacts")?,
            ..Default::default()
        };
        let mut query = self.connection.prepare("SELECT json_type(t.value,'$.outcome.Captured'), json_type(t.value,'$.outcome.Failed'), json_type(t.value,'$.outcome.Skipped'), json_extract(t.value,'$.outcome') FROM (SELECT record FROM runs ORDER BY rowid DESC LIMIT 100) r, json_each(r.record,'$.run.targets') t LIMIT 10000")?;
        let rows = query.query_map([], |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?;
        for row in rows {
            let (captured, failed, skipped, scalar) = row?;
            counts.sampled_targets += 1;
            if captured.as_deref() == Some("object") {
                counts.captured += 1;
            } else if failed.as_deref() == Some("object") {
                counts.failed += 1;
            } else if skipped.as_deref() == Some("object") {
                counts.skipped += 1;
            } else {
                match scalar.as_deref() {
                    Some("Cancelled") => counts.cancelled += 1,
                    Some("Pending") => counts.pending += 1,
                    _ => counts.unrecognized += 1,
                }
            }
        }
        Ok(counts)
    }
}
