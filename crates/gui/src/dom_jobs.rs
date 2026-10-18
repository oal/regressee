// SPDX-License-Identifier: MPL-2.0
//! Coalesced, display-only DOM read lane. Every byte comes through engine IPC.
use super::{Operation, Outcome, Remote};
use crate::ui::dom_diff::{DomDiffDocument, DomDiffError};
use anyhow::{Context, Result, ensure};
use regressee_app::engine::{dom, text};
use regressee_core::{dom_snapshot::*, domain::*};
use std::{
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DomPresentation {
    #[default]
    Diff,
    Before,
    After,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomViewerIdentity {
    pub project: ProjectId,
    pub run: RunId,
    pub analysis: Option<DomAnalysisId>,
    pub key: TargetKey,
    pub presentation: DomPresentation,
}
impl DomViewerIdentity {
    fn selection(&self) -> dom::Selection {
        dom::Selection {
            project: self.project,
            run: Some(self.run),
            analysis: self.analysis,
        }
    }
}
pub struct DomViewerFrame {
    pub identity: DomViewerIdentity,
    pub artifacts: Box<dom::Artifacts>,
    pub document: Arc<DomDiffDocument>,
}
pub struct DomViewerUpdate {
    pub id: u64,
    pub identity: DomViewerIdentity,
    pub result: Result<DomViewerFrame, DomDiffError>,
}
pub(super) struct State {
    generation: AtomicU64,
    intent: Mutex<Option<(u64, u64, DomViewerIdentity)>>,
    updates: SyncSender<DomViewerUpdate>,
}
impl State {
    pub(super) fn new() -> (Self, Receiver<DomViewerUpdate>) {
        let (updates, events) = mpsc::sync_channel(4);
        (
            Self {
                generation: AtomicU64::new(0),
                intent: Mutex::new(None),
                updates,
            },
            events,
        )
    }
    pub(super) fn submit(&self, id: u64, identity: DomViewerIdentity) {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        *self.intent.lock().unwrap() = Some((generation, id, identity));
    }
    pub(super) fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.intent.lock().unwrap().take();
    }
}
pub(super) fn start(remote: Weak<Remote>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(remote) = remote.upgrade() {
            if remote.stopped.load(Ordering::Acquire) {
                break;
            }
            let intent = remote.dom.intent.lock().unwrap().take();
            let Some((generation, id, identity)) = intent else {
                drop(remote);
                thread::sleep(Duration::from_millis(10));
                continue;
            };
            let result = remote
                .read_dom_viewer(&identity, generation)
                .map_err(|error| {
                    DomDiffError::new(
                        "DOM source could not be loaded.",
                        format!(
                            "{} Refresh the saved result to try again. No website was visited.",
                            error.to_string().chars().take(4096).collect::<String>()
                        ),
                    )
                });
            let mut update = DomViewerUpdate {
                id,
                identity,
                result,
            };
            while remote.dom_current(generation) {
                match remote.dom.updates.try_send(update) {
                    Ok(()) | Err(TrySendError::Disconnected(_)) => break,
                    Err(TrySendError::Full(value)) => {
                        update = value;
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        }
    })
}
impl Remote {
    fn dom_current(&self, generation: u64) -> bool {
        !self.stopped.load(Ordering::Acquire)
            && self.dom.generation.load(Ordering::Acquire) == generation
    }
    fn dom_query(&self, command: dom::Command, generation: u64) -> Result<Box<dom::Output>> {
        ensure!(self.dom_current(generation), "DOM read superseded");
        let job = self.query(Operation::Dom(command))?;
        let _ = self
            .client
            .request(regressee_app::protocol::Request::Acknowledge { id: job.id });
        ensure!(self.dom_current(generation), "DOM read superseded");
        match job.outcome {
            Some(Outcome::Dom(output)) => Ok(output),
            Some(Outcome::ProductFailure(error)) => anyhow::bail!("{error}"),
            _ => anyhow::bail!("The engine did not return the requested DOM evidence."),
        }
    }
    fn read_dom_source(
        &self,
        identity: &DomViewerIdentity,
        source: &DomSourceIdentity,
        pane: dom::SourcePane,
        generation: u64,
    ) -> Result<String> {
        let mut output = String::new();
        let mut total = None;
        loop {
            let page = self.dom_query(
                dom::Command::Source {
                    selection: identity.selection(),
                    key: identity.key.clone(),
                    pane,
                    format: dom::SourceFormat::Plaintext,
                    offset: output.len(),
                    limit: text::MAX_SOURCE_PAGE_BYTES,
                },
                generation,
            )?;
            let dom::Output::Source(page) = *page else {
                anyhow::bail!("Unexpected DOM source response")
            };
            ensure!(
                page.source == *source
                    && page.key == identity.key
                    && page.selection.project == identity.project
                    && page.selection.run == Some(identity.run)
                    && page.selection.analysis == identity.analysis
                    && page.page.offset == output.len()
                    && page.page.text.len() <= text::MAX_SOURCE_PAGE_BYTES
                    && page.page.total_bytes <= MAX_DOM_SOURCE_BYTES
                    && total.is_none_or(|total| total == page.page.total_bytes),
                "DOM source identity or page changed during reading"
            );
            total = Some(page.page.total_bytes);
            ensure!(
                output
                    .len()
                    .checked_add(page.page.text.len())
                    .is_some_and(|n| n <= page.page.total_bytes),
                "DOM source page exceeds its recorded length"
            );
            output.push_str(&page.page.text);
            match page.page.next_offset {
                Some(next) => ensure!(
                    next == output.len() && next > page.page.offset && next < page.page.total_bytes,
                    "DOM source paging did not advance"
                ),
                None => {
                    ensure!(
                        output.len() == page.page.total_bytes,
                        "Incomplete DOM source page"
                    );
                    break;
                }
            }
        }
        Ok(output)
    }
    fn read_dom_diff(
        &self,
        identity: &DomViewerIdentity,
        before: &DomSourceIdentity,
        after: &DomSourceIdentity,
        generation: u64,
    ) -> Result<DomTextDiff> {
        let mut offset = 0;
        let mut summary = None;
        let mut headers: Vec<(DomDiffHunk, usize)> = Vec::new();
        let mut bytes = 0usize;
        loop {
            let response = self.dom_query(
                dom::Command::Diff {
                    selection: identity.selection(),
                    key: identity.key.clone(),
                    options: DomDiffOptions::default(),
                    offset,
                    limit: text::MAX_DIFF_PAGE_ROWS,
                },
                generation,
            )?;
            let dom::Output::Diff(response) = *response else {
                anyhow::bail!("Unexpected DOM diff response")
            };
            ensure!(
                response.before == *before
                    && response.after == *after
                    && response.key == identity.key
                    && response.selection.project == identity.project
                    && response.selection.run == Some(identity.run)
                    && response.selection.analysis == identity.analysis,
                "DOM diff source identity changed"
            );
            let page = response.page;
            ensure!(
                page.offset == offset
                    && page.total_rows <= MAX_DOM_DIFF_OUTPUT_LINES
                    && page.summary.total_rows == page.total_rows,
                "Invalid DOM diff page extent"
            );
            if let Some(expected) = &summary {
                ensure!(
                    summary_key(expected) == summary_key(&page.summary),
                    "DOM diff summary changed between pages"
                );
            } else {
                summary = Some(page.summary.clone());
            }
            let mut received = 0;
            for hunk in page.hunks {
                received += hunk.rows.len();
                ensure!(
                    hunk.total_rows <= MAX_DOM_DIFF_OUTPUT_LINES
                        && hunk.row_offset + hunk.rows.len() <= hunk.total_rows,
                    "Invalid DOM hunk page extent"
                );
                bytes = hunk
                    .rows
                    .iter()
                    .try_fold(bytes, |bytes, row| bytes.checked_add(row.text.len() + 64))
                    .context("DOM diff size overflow")?;
                ensure!(
                    bytes <= MAX_DOM_DIFF_OUTPUT_BYTES,
                    "DOM diff exceeds the bounded output limit"
                );
                if hunk.row_offset == 0 {
                    headers.push((
                        DomDiffHunk {
                            old_start: hunk.old_start,
                            old_lines: hunk.old_lines,
                            new_start: hunk.new_start,
                            new_lines: hunk.new_lines,
                            rows: hunk.rows,
                        },
                        hunk.total_rows,
                    ));
                } else {
                    let (last, total_rows) =
                        headers.last_mut().context("Missing first DOM hunk page")?;
                    ensure!(
                        last.old_start == hunk.old_start
                            && last.old_lines == hunk.old_lines
                            && last.new_start == hunk.new_start
                            && last.new_lines == hunk.new_lines
                            && *total_rows == hunk.total_rows
                            && last.rows.len() == hunk.row_offset,
                        "DOM hunk changed between pages"
                    );
                    last.rows.extend(hunk.rows);
                }
            }
            ensure!(
                received <= text::MAX_DIFF_PAGE_ROWS && offset + received <= page.total_rows,
                "DOM diff page exceeds its row limit"
            );
            let received_end = offset + received;
            match page.next_offset {
                Some(next) => {
                    ensure!(
                        next == received_end && received > 0 && next < page.total_rows,
                        "DOM diff paging did not advance"
                    );
                    offset = next;
                }
                None => {
                    ensure!(received_end == page.total_rows, "Incomplete DOM diff page");
                    break;
                }
            }
        }
        let summary = summary.context("No DOM diff summary")?;
        ensure!(
            headers.iter().all(|(h, total)| h.rows.len() == *total),
            "Incomplete DOM hunk"
        );
        let diff = DomTextDiff {
            changed: summary.changed,
            old_line_count: summary.old_line_count,
            new_line_count: summary.new_line_count,
            added_lines: summary.added_lines,
            removed_lines: summary.removed_lines,
            fallback: summary.fallback,
            display_limited: summary.display_limited,
            hunks: headers.into_iter().map(|(h, _)| h).collect(),
        };
        diff.validate()?;
        Ok(diff)
    }
    fn read_dom_viewer(
        &self,
        identity: &DomViewerIdentity,
        generation: u64,
    ) -> Result<DomViewerFrame> {
        let output = self.dom_query(
            dom::Command::Artifacts {
                selection: identity.selection(),
                key: identity.key.clone(),
            },
            generation,
        )?;
        let dom::Output::Artifacts(artifacts) = *output else {
            anyhow::bail!("Unexpected DOM artifact response")
        };
        ensure!(
            artifacts.key == identity.key
                && artifacts.selection.project == identity.project
                && artifacts.selection.run == Some(identity.run)
                && artifacts.selection.analysis == identity.analysis,
            "DOM viewer selection changed"
        );
        let comparable = artifacts
            .result
            .as_ref()
            .is_some_and(|r| r.outcome.comparable());
        let document = match identity.presentation {
            DomPresentation::Diff if comparable => {
                let before = &artifacts
                    .before
                    .as_ref()
                    .context("Before DOM source is absent")?
                    .source;
                let after = &artifacts
                    .after
                    .as_ref()
                    .context("After DOM source is absent")?
                    .source;
                let before_text =
                    self.read_dom_source(identity, before, dom::SourcePane::Before, generation)?;
                let after_text =
                    self.read_dom_source(identity, after, dom::SourcePane::After, generation)?;
                let diff = self.read_dom_diff(identity, before, after, generation)?;
                ensure!(
                    artifacts
                        .result
                        .as_ref()
                        .is_some_and(|r| r.before.as_ref() == Some(before)
                            && r.after.as_ref() == Some(after)
                            && diff.changed == (r.outcome == DomComparisonOutcome::Changed)),
                    "DOM diff does not match its recorded result"
                );
                crate::dom_source::comparison(&before_text, &after_text, &diff)?
            }
            presentation => {
                let before = presentation == DomPresentation::Before
                    || (presentation == DomPresentation::Diff && artifacts.after.is_none());
                let (original, pane) = if before {
                    (artifacts.before.as_ref(), dom::SourcePane::Before)
                } else {
                    (artifacts.after.as_ref(), dom::SourcePane::After)
                };
                let original = original.context("This DOM original was not captured or was omitted from the baseline. Inspect the per-kind outcome in Details.")?;
                let source = self.read_dom_source(identity, &original.source, pane, generation)?;
                let notice = (!comparable)
                    .then(|| Arc::<str>::from(artifacts.warnings.join(" ")))
                    .filter(|s| !s.is_empty());
                crate::dom_source::original(&source, before, notice)?
            }
        };
        ensure!(self.dom_current(generation), "DOM read superseded");
        Ok(DomViewerFrame {
            identity: identity.clone(),
            artifacts,
            document: Arc::new(document),
        })
    }
}
fn summary_key(
    summary: &text::DiffSummary,
) -> (
    bool,
    u32,
    u32,
    u32,
    u32,
    Option<DomDiffFallback>,
    bool,
    usize,
) {
    (
        summary.changed,
        summary.old_line_count,
        summary.new_line_count,
        summary.added_lines,
        summary.removed_lines,
        summary.fallback,
        summary.display_limited,
        summary.total_rows,
    )
}
