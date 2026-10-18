// SPDX-License-Identifier: MPL-2.0
use crate::control::*;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub(super) struct Agent {
    pub info: Session,
    secret: Uuid,
    deadline: Instant,
}
pub(super) struct Frontend {
    secret: Uuid,
    deadline: Instant,
}
#[derive(Default)]
pub(super) struct Authority {
    pub agents: BTreeMap<Uuid, Agent>,
    frontends: BTreeMap<Uuid, Frontend>,
    pub controller: Option<Uuid>,
    bootstrap: Option<FrontendCapability>,
}
impl Authority {
    pub fn with_bootstrap(bootstrap: FrontendCapability) -> Self {
        Self {
            bootstrap: Some(bootstrap),
            ..Default::default()
        }
    }
    pub fn register_frontend(
        &mut self,
        capability: &FrontendCapability,
        engine: Uuid,
    ) -> Result<crate::protocol::Response> {
        ensure!(
            capability.engine == engine,
            "stale_epoch: frontend bootstrap incarnation changed"
        );
        ensure!(
            self.bootstrap
                .as_ref()
                .is_some_and(|c| c.engine == engine && c.secret == capability.secret),
            "authority_conflict: private frontend bootstrap capability required"
        );
        ensure!(self.frontends.len() < 16, "busy: frontend capacity reached");
        let id = Uuid::new_v4();
        let secret = Uuid::new_v4();
        self.frontends.insert(
            id,
            Frontend {
                secret,
                deadline: Instant::now() + Duration::from_secs(FRONTEND_LEASE),
            },
        );
        Ok(crate::protocol::Response::Credential(Credential {
            engine,
            owner: Owner::Frontend(id),
            secret,
        }))
    }
    pub fn expire(&mut self, epoch: &mut u64, now: Instant) {
        let expired = self.controller.is_some_and(|id| {
            let a = &self.agents[&id];
            now >= a.deadline
                || a.info
                    .follow
                    .is_some_and(|f| self.frontends.get(&f).is_none_or(|f| now >= f.deadline))
        });
        if expired {
            let id = self.controller.take().unwrap();
            let a = self.agents.get_mut(&id).unwrap();
            a.info.status = Status::Expired;
            a.info.follow = None;
            *epoch += 1;
        }
        self.frontends.retain(|_, f| f.deadline > now);
    }
    pub fn authenticate(&self, credential: &Credential, engine: Uuid) -> Result<Owner> {
        ensure!(credential.engine == engine, "stale_epoch: engine restarted");
        let secret = match credential.owner {
            Owner::Agent(id) => self.agents.get(&id).map(|a| a.secret),
            Owner::Frontend(id) => self.frontends.get(&id).map(|f| f.secret),
        };
        ensure!(
            secret == Some(credential.secret),
            "authority_conflict: unknown or expired client credential"
        );
        Ok(credential.owner)
    }
    pub fn authorize(&self, owner: Option<Owner>) -> Result<()> {
        if let Some(Owner::Frontend(id)) = owner {
            ensure!(
                self.frontends.contains_key(&id),
                "authority_conflict: frontend lease expired"
            );
        }
        if let Some(id) = self.controller {
            let a = &self.agents[&id].info;
            let allowed = match a.status {
                Status::Active => owner == Some(Owner::Agent(id)),
                Status::Human => a.follow.is_some_and(|f| owner == Some(Owner::Frontend(f))),
                _ => false,
            };
            ensure!(
                allowed,
                "authority_conflict: exclusive control belongs to another session or frontend; inspect control status or take control in the GUI"
            );
        } else if let Some(Owner::Agent(_)) = owner {
            anyhow::bail!(
                "authority_conflict: agent lease ended, expired or was revoked; start a new session deliberately"
            );
        }
        Ok(())
    }
    pub fn snapshot(&self, engine: Uuid, epoch: u64) -> Snapshot {
        Snapshot {
            engine,
            epoch,
            controller: self.controller,
            sessions: self
                .agents
                .values()
                .map(|a| {
                    let mut info = a.info.clone();
                    info.remaining_seconds = if info.status.live() {
                        a.deadline
                            .saturating_duration_since(Instant::now())
                            .as_secs()
                    } else {
                        0
                    };
                    info
                })
                .collect(),
        }
    }
    pub fn command(
        &mut self,
        command: Command,
        owner: Option<Owner>,
        engine: Uuid,
        epoch: &mut u64,
    ) -> Result<crate::protocol::Response> {
        use crate::protocol::Response;
        validate(&command)?;
        match command {
            Command::List => {}
            Command::Show { session } => {
                ensure!(
                    self.agents.contains_key(&session),
                    "not_found: unknown agent session"
                );
            }
            Command::RegisterFrontend => {
                anyhow::bail!(
                    "authority_conflict: private frontend bootstrap capability required; anonymous role registration is refused"
                )
            }
            Command::Start { name, seconds } => {
                ensure!(
                    self.controller.is_none(),
                    "authority_conflict: another agent session owns this data directory"
                );
                ensure!(
                    self.agents.len() < 128,
                    "busy: session history capacity reached; restart after ending sessions"
                );
                let id = Uuid::new_v4();
                let secret = Uuid::new_v4();
                self.agents.insert(
                    id,
                    Agent {
                        secret,
                        deadline: Instant::now() + Duration::from_secs(seconds),
                        info: Session {
                            id,
                            name,
                            status: Status::Active,
                            remaining_seconds: seconds,
                            follow: None,
                            selection: None,
                            selection_revision: 0,
                            handoff: None,
                        },
                    },
                );
                self.controller = Some(id);
                *epoch += 1;
                return Ok(Response::Credential(Credential {
                    engine,
                    owner: Owner::Agent(id),
                    secret,
                }));
            }
            Command::Heartbeat => {
                let Some(Owner::Frontend(id)) = owner else {
                    anyhow::bail!("authority_conflict: frontend required")
                };
                self.frontends
                    .get_mut(&id)
                    .context("authority_conflict: frontend expired")?
                    .deadline = Instant::now() + Duration::from_secs(FRONTEND_LEASE);
            }
            Command::Renew { .. } | Command::End | Command::Handoff { .. } => {
                let Some(Owner::Agent(id)) = owner else {
                    anyhow::bail!("authority_conflict: owning agent required")
                };
                ensure!(
                    self.controller == Some(id),
                    "authority_conflict: session no longer owns control"
                );
                let a = self.agents.get_mut(&id).unwrap();
                match command {
                    Command::Renew { seconds } => {
                        a.deadline = Instant::now() + Duration::from_secs(seconds)
                    }
                    Command::End => {
                        a.info.status = Status::Ended;
                        a.info.follow = None;
                        self.controller = None;
                        *epoch += 1;
                    }
                    Command::Handoff { message } => {
                        ensure!(
                            a.info.status == Status::Active,
                            "authority_conflict: already handed off"
                        );
                        a.info.status = Status::Human;
                        a.info.handoff = Some(message);
                        *epoch += 1;
                    }
                    _ => unreachable!(),
                }
            }
            Command::Follow { session } => {
                let Some(Owner::Frontend(id)) = owner else {
                    anyhow::bail!("authority_conflict: GUI frontend consent required")
                };
                ensure!(
                    self.controller == Some(session),
                    "authority_conflict: session is not active"
                );
                let a = self.agents.get_mut(&session).unwrap();
                ensure!(
                    a.info.follow.is_none() || a.info.follow == Some(id),
                    "authority_conflict: another GUI follows this session"
                );
                a.info.follow = Some(id);
                *epoch += 1;
            }
            Command::Detach => {
                let Some(Owner::Frontend(id)) = owner else {
                    anyhow::bail!("authority_conflict: GUI frontend required")
                };
                if let Some(session) = self.controller
                    && self.agents[&session].info.follow == Some(id)
                {
                    let a = self.agents.get_mut(&session).unwrap();
                    a.info.status = Status::Revoked;
                    a.info.follow = None;
                    self.controller = None;
                    *epoch += 1;
                }
                // Closing an unrelated GUI cannot revoke another frontend's
                // session, but must still retire its own registered credential.
                self.frontends.remove(&id);
            }
            Command::Takeover | Command::Return => {
                let Some(Owner::Frontend(id)) = owner else {
                    anyhow::bail!("authority_conflict: GUI frontend required")
                };
                if let Some(session) = self.controller {
                    let a = self.agents.get_mut(&session).unwrap();
                    match command {
                        Command::Takeover => {
                            a.info.status = Status::Revoked;
                            a.info.follow = None;
                            self.controller = None;
                        }
                        Command::Return => {
                            ensure!(
                                a.info.follow == Some(id) && a.info.status == Status::Human,
                                "authority_conflict: no human handoff for this frontend"
                            );
                            a.info.status = Status::Active;
                            a.info.handoff = None;
                        }
                        _ => unreachable!(),
                    }
                    *epoch += 1;
                }
            }
        }
        Ok(Response::Control(self.snapshot(engine, *epoch)))
    }
}

pub(super) fn location(
    store: &regressee_storage::Store,
    mut selection: Selection,
) -> Result<Location> {
    let project = store.load_project(selection.project)?;
    ensure!(
        selection.run.is_some() || (selection.analysis.is_none() && selection.key.is_none()),
        "invalid_request: result/analysis selection requires an exact run"
    );
    let evidence = if let Some(run) = selection.run {
        if selection.kind == regressee_core::domain::CheckKind::DomSnapshot {
            ensure!(
                matches!(
                    selection.view,
                    View::Compare | View::Before | View::After | View::Difference
                ),
                "invalid_request: DOM text supports compare/before/after/difference, not image overlay or split"
            );
            let (resolved, e) = super::dom::evidence(
                store,
                &super::dom::Selection {
                    project: selection.project,
                    run: Some(run),
                    analysis: selection
                        .analysis
                        .map(|id| regressee_core::domain::DomAnalysisId(id.0)),
                },
            )?;
            selection.analysis = resolved
                .analysis
                .map(|id| regressee_core::domain::AnalysisId(id.0));
            if let Some(key) = &selection.key {
                ensure!(
                    e.latest
                        .as_ref()
                        .is_some_and(|r| r.run.targets.iter().any(|t| &t.key == key)),
                    "invalid_request: DOM result/page/profile absent from selected run"
                );
            }
            return Ok(Location {
                selection,
                project,
                evidence: Some(e),
            });
        }
        let mut e = if selection.analysis.is_some() {
            store.viewer_evidence(selection.project, selection.analysis)?
        } else {
            store.screenshot_evidence_for_run(selection.project, run)?
        };
        ensure!(
            e.latest.as_ref().is_some_and(|r| r.run.id == run),
            "invalid_request: exact screenshot analysis/run mismatch"
        );
        ensure!(
            e.latest
                .as_ref()
                .is_some_and(|r| r.run.capture_plan.mode.screenshots()),
            "invalid_request: this run requested DOM snapshots only; select --kind dom"
        );
        if let Some(id) = selection.analysis {
            let a = store.load_analysis(id)?;
            ensure!(
                a.revision.run == run
                    && e.baseline
                        .as_ref()
                        .is_some_and(|b| b.id == a.revision.baseline),
                "invalid_request: analysis/run/reference mismatch"
            );
            e.comparison = Some(a);
            e.analyses = vec![id];
            e.reviews = store.review_history(run)?;
            e.review_history_truncated = store.review_history_truncated(run)?;
        }
        selection.analysis = e.comparison.as_ref().map(|a| a.revision.id);
        if let Some(key) = &selection.key {
            ensure!(
                e.latest
                    .as_ref()
                    .is_some_and(|r| r.run.targets.iter().any(|t| &t.key == key)),
                "invalid_request: result/page/profile is absent from the selected run"
            );
        }
        Some(e)
    } else {
        None
    };
    Ok(Location {
        selection,
        project,
        evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Response;
    fn start(a: &mut Authority, engine: Uuid, epoch: &mut u64) -> Credential {
        let Response::Credential(c) = a
            .command(
                Command::Start {
                    name: "Owned agent".into(),
                    seconds: 10,
                },
                None,
                engine,
                epoch,
            )
            .unwrap()
        else {
            panic!()
        };
        c
    }
    #[test]
    fn bounded_lease_renewal_expiry_and_no_process_disconnect_semantics() {
        let mut a = Authority::default();
        let engine = Uuid::new_v4();
        let mut epoch = 0;
        for seconds in [0, 9, 301, u64::MAX] {
            assert!(
                a.command(
                    Command::Start {
                        name: "Agent".into(),
                        seconds
                    },
                    None,
                    engine,
                    &mut epoch
                )
                .is_err()
            );
        }
        let c = start(&mut a, engine, &mut epoch);
        let Owner::Agent(id) = c.owner else { panic!() };
        let initial = a.agents[&id].deadline;
        a.command(
            Command::Renew { seconds: 300 },
            Some(c.owner),
            engine,
            &mut epoch,
        )
        .unwrap();
        assert!(a.agents[&id].deadline > initial);
        a.expire(&mut epoch, initial);
        assert_eq!(a.controller, Some(id));
        let deadline = a.agents[&id].deadline;
        a.expire(&mut epoch, deadline);
        assert_eq!(a.controller, None);
        assert_eq!(epoch, 2);
        assert_eq!(a.agents[&id].info.status, Status::Expired);
        assert!(a.authorize(Some(c.owner)).is_err());
        assert!(
            a.command(
                Command::Renew { seconds: 90 },
                Some(c.owner),
                engine,
                &mut epoch
            )
            .is_err()
        );
        assert!(a.authenticate(&c, Uuid::new_v4()).is_err());
    }
    #[test]
    fn frontend_disconnect_unlocks_and_does_not_touch_jobs_or_cancel_flags() {
        let engine = Uuid::new_v4();
        let capability = FrontendCapability {
            engine,
            secret: Uuid::new_v4(),
        };
        let mut a = Authority::with_bootstrap(capability.clone());
        let mut epoch = 0;
        let c = start(&mut a, engine, &mut epoch);
        let Owner::Agent(id) = c.owner else { panic!() };
        let Response::Credential(f) = a.register_frontend(&capability, engine).unwrap() else {
            panic!()
        };
        let Owner::Frontend(frontend) = f.owner else {
            panic!()
        };
        a.command(
            Command::Follow { session: id },
            Some(f.owner),
            engine,
            &mut epoch,
        )
        .unwrap();
        a.command(
            Command::Handoff {
                message: "Human task".into(),
            },
            Some(c.owner),
            engine,
            &mut epoch,
        )
        .unwrap();
        a.agents.get_mut(&id).unwrap().deadline = Instant::now() + Duration::from_secs(300);
        a.expire(
            &mut epoch,
            Instant::now() + Duration::from_secs(FRONTEND_LEASE + 1),
        );
        assert!(a.controller.is_none());
        assert!(a.agents[&id].info.follow.is_none());
        assert!(!a.frontends.contains_key(&frontend));
        assert!(a.authorize(None).is_ok());
        assert!(a.authenticate(&f, engine).is_err());
    }
}
