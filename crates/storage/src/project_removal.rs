// SPDX-License-Identifier: MPL-2.0
//! Whole-project and environment deletion, within retention's confirmed writer.
use super::*;

pub(super) fn sessions(
    c: &Connection,
    project: ProjectId,
    item: DeleteItem,
) -> Result<Vec<(SessionReference, bool)>> {
    let environment = match item {
        DeleteItem::Project => None,
        DeleteItem::Environment(environment) => Some(environment.to_string()),
        _ => return Ok(Vec::new()),
    };
    let mut query = c.prepare("SELECT environment,session,ephemeral FROM local_sessions WHERE project=?1 AND (?2 IS NULL OR environment=?2) ORDER BY environment")?;
    query
        .query_map(params![project.to_string(), environment], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
            ))
        })?
        .map(|row| {
            let (environment, session, ephemeral) = row?;
            let environment = EnvironmentId(uuid::Uuid::parse_str(&environment)?);
            let id = SessionId(uuid::Uuid::parse_str(&session)?);
            ensure!(
                !id.0.is_nil() && !environment.0.is_nil(),
                "Invalid saved login identity; retain the project for recovery"
            );
            Ok((
                SessionReference {
                    project,
                    environment,
                    id,
                },
                ephemeral,
            ))
        })
        .collect()
}

/// The manifest includes mutable reviews, masks, private references and page
/// lists, not just runs/baselines. None may change behind an impact confirmation.
pub(super) fn scope_state(
    c: &Connection,
    project: &Project,
    item: DeleteItem,
) -> Result<Option<(ScopeDeletion, Vec<String>)>> {
    if !matches!(item, DeleteItem::Project | DeleteItem::Environment(_)) {
        return Ok(None);
    }
    let active: u64 = c.query_row("SELECT count(*) FROM runs WHERE project_id=?1 AND state IN ('\"Running\"','\"Pausing\"','\"Queued\"')", [project.id.to_string()], |r| r.get(0))?;
    ensure!(
        active == 0,
        "A capture is still active for this project. Stop it before removing the project or an environment."
    );
    let name = if let DeleteItem::Environment(id) = item {
        ensure!(
            project.environments.len() > 1,
            "Keep at least one environment in this project."
        );
        project
            .environments
            .iter()
            .find(|e| e.id == id)
            .context("Environment is no longer in this project; inspect it again")?
            .name
            .clone()
    } else {
        project.name.clone()
    };
    let count = |sql: &str| -> Result<usize> {
        Ok(c.query_row(sql, [project.id.to_string()], |r| r.get::<_, usize>(0))?)
    };
    let whole = item == DeleteItem::Project;
    let details = ScopeDeletion {
        name,
        project_name: project.name.clone(),
        address: project.sources[0].to_string(),
        source_count: project.sources.len(),
        runs: if whole {
            count("SELECT count(*) FROM runs WHERE project_id=?1")?
        } else {
            0
        },
        baselines: if whole {
            count("SELECT count(*) FROM baselines WHERE project_id=?1")?
        } else {
            0
        },
        reviews: if whole {
            count(
                "SELECT (SELECT count(*) FROM review_decisions d JOIN analyses a ON a.id=d.analysis_id JOIN runs r ON r.id=a.run_id WHERE r.project_id=?1)+(SELECT count(*) FROM dom_review_decisions d JOIN dom_analyses a ON a.id=d.analysis_id JOIN runs r ON r.id=a.run_id WHERE r.project_id=?1)",
            )?
        } else {
            0
        },
        ignored_areas: if whole {
            count("SELECT count(*) FROM ignore_masks WHERE project_id=?1")?
        } else {
            0
        },
        logins: sessions(c, project.id, item)?.len(),
    };
    let mut manifest = Vec::new();
    for sql in [
        "SELECT json_array('inventory',id,record) FROM inventories WHERE project_id=?1 ORDER BY id",
        "SELECT json_array('mask',id,record) FROM ignore_masks WHERE project_id=?1 ORDER BY id",
        "SELECT json_array('review',d.id,d.record) FROM review_decisions d JOIN analyses a ON a.id=d.analysis_id JOIN runs r ON r.id=a.run_id WHERE r.project_id=?1 ORDER BY d.id",
        "SELECT json_array('dom-review',d.id,d.record) FROM dom_review_decisions d JOIN dom_analyses a ON a.id=d.analysis_id JOIN runs r ON r.id=a.run_id WHERE r.project_id=?1 ORDER BY d.id",
        "SELECT json_array('login',environment,session,ephemeral) FROM local_sessions WHERE project=?1 ORDER BY environment",
        "SELECT json_array('local',environment,settings) FROM local_environment_settings WHERE project=?1",
        "SELECT json_array('display',d.run_id,d.analysis_id) FROM analysis_display d JOIN runs r ON r.id=d.run_id WHERE r.project_id=?1 ORDER BY d.run_id",
        "SELECT json_array('dom-display',d.run_id,d.analysis_id,d.revision) FROM dom_analysis_display d JOIN runs r ON r.id=d.run_id WHERE r.project_id=?1 ORDER BY d.run_id",
    ] {
        let mut query = c.prepare(sql)?;
        manifest.extend(
            query
                .query_map([project.id.to_string()], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?,
        );
    }
    Ok(Some((details, manifest)))
}

pub(super) fn delete_scope(tx: &Connection, project: ProjectId, item: DeleteItem) -> Result<()> {
    let id = project.to_string();
    match item {
        DeleteItem::Environment(environment) => {
            let json: String =
                tx.query_row("SELECT config FROM projects WHERE id=?1", [&id], |r| {
                    r.get(0)
                })?;
            let mut p: Project = serde_json::from_str(&json)?;
            p.environments.retain(|e| e.id != environment);
            p.validate()?;
            tx.execute(
                "UPDATE projects SET config=?2 WHERE id=?1",
                params![id, serde_json::to_string(&p)?],
            )?;
            tx.execute(
                "DELETE FROM local_environment_settings WHERE project=?1 AND environment=?2",
                params![id, environment.to_string()],
            )?;
            tx.execute(
                "DELETE FROM local_sessions WHERE project=?1 AND environment=?2",
                params![id, environment.to_string()],
            )?;
        }
        DeleteItem::Project => {
            // Analysis children were removed by retention before this operation.
            // FK checks remain on; external/corrupt dependencies fail closed.
            for sql in [
                "DELETE FROM imported_origins WHERE run_id IN (SELECT id FROM runs WHERE project_id=?1)",
                "DELETE FROM ignore_masks WHERE project_id=?1",
                "DELETE FROM baselines WHERE project_id=?1",
                "DELETE FROM runs WHERE project_id=?1",
                "DELETE FROM inventories WHERE project_id=?1",
                "DELETE FROM local_environment_settings WHERE project=?1",
                "DELETE FROM local_sessions WHERE project=?1",
                "DELETE FROM projects WHERE id=?1",
            ] {
                tx.execute(sql, [&id])?;
            }
        }
        _ => bail!("Not a project or environment deletion"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_removal_reports_committed_deletion_with_retryable_file_cleanup() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let p = Project::draft("Pending cleanup".into(), "https://example.test/")?;
        store.save_project(&p, None)?;
        let run = RunId::new();
        store.connection.execute(
            "INSERT INTO runs VALUES(?1,?2,?3,'{}',0,0)",
            params![
                run.to_string(),
                p.id.to_string(),
                serde_json::to_string(&JobState::Completed)?
            ],
        )?;
        let artifact = store.put_artifact(&b"isolated cleanup fixture"[..], ArtifactKind::Tile)?;
        store.add_reference(&ArtifactOwner::Run(run), &artifact.id)?;
        let preview = store.deletion_preview(p.id, DeleteItem::Project)?;
        let artifact_path = store.paths.artifacts().join(artifact.id.as_str());
        let saved_path = root.path().join("owned-artifact-stash");
        // Simulate an entry changing after impact validation. GC must not unlink
        // an unsafe replacement, or pretend the already committed project remains.
        let result = store.delete_confirmed_with_cleanup(&preview, |_| {
            fs::rename(&artifact_path, &saved_path)?;
            fs::create_dir(&artifact_path)?;
            Ok(())
        });
        assert!(
            result?,
            "confirmed metadata deletion must distinguish pending physical cleanup"
        );
        assert!(store.load_project(p.id).is_err());
        assert!(artifact_path.is_dir());
        fs::remove_dir(&artifact_path)?;
        fs::rename(saved_path, &artifact_path)?;
        store.complete_confirmed_gc()?;
        assert!(!store.paths.artifacts().join(artifact.id.as_str()).exists());
        Ok(())
    }

    #[test]
    fn project_removal_rejects_active_stale_and_tampered_impact_before_cleanup() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let p = Project::draft("Test website".into(), "https://example.test/")?;
        store.save_project(&p, None)?;
        let preview = store.deletion_preview(p.id, DeleteItem::Project)?;
        assert_eq!(preview.scope.as_ref().unwrap().runs, 0);
        let mut tampered = preview.clone();
        tampered.scope.as_mut().unwrap().name = "Other website".into();
        assert!(
            store
                .delete_confirmed_with_cleanup(&tampered, |_| panic!(
                    "tampered consent must not touch logins"
                ))
                .is_err()
        );
        store.select_environment(
            p.id,
            p.environments[0].id,
            &LocalLoginSettings {
                ephemeral: true,
                ..Default::default()
            },
        )?;
        assert!(
            store
                .delete_confirmed_with_cleanup(&preview, |_| panic!(
                    "stale consent must not touch logins"
                ))
                .is_err()
        );
        let run = RunId::new();
        store.connection.execute(
            "INSERT INTO runs VALUES(?1,?2,?3,'{}',0,0)",
            params![
                run.to_string(),
                p.id.to_string(),
                serde_json::to_string(&JobState::Running)?
            ],
        )?;
        assert!(store.deletion_preview(p.id, DeleteItem::Project).is_err());
        store
            .connection
            .execute("DELETE FROM runs WHERE id=?1", [run.to_string()])?;
        let preview = store.deletion_preview(p.id, DeleteItem::Project)?;
        store.delete_confirmed(&preview)?;
        assert!(store.load_project(p.id).is_err());
        assert!(store.list_projects(0)?.items.is_empty());
        assert!(store.delete_confirmed(&preview).is_err());
        Ok(())
    }

    #[test]
    fn environment_removal_protects_last_environment_and_rolls_back_on_login_failure() -> Result<()>
    {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let mut p = Project::draft("Environments".into(), "https://example.test/")?;
        let original = p.environments[0].id;
        let mut staging = p.environments[0].clone();
        staging.id = EnvironmentId::new();
        staging.name = "Staging".into();
        p.environments.push(staging.clone());
        let initial = store.save_project(&p, None)?;
        store.select_environment(p.id, staging.id, &LocalLoginSettings::default())?;
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: staging.id,
        };
        store.save_session_reference(&scope, true)?;
        let preview = store.deletion_preview(p.id, DeleteItem::Environment(staging.id))?;
        assert_eq!(preview.scope.as_ref().unwrap().logins, 1);
        assert!(
            store.delete_confirmed(&preview).is_err(),
            "storage alone cannot orphan a private login"
        );
        assert!(
            store
                .delete_confirmed_with_cleanup(&preview, |sessions| {
                    assert_eq!(sessions, &[(scope.clone(), true)]);
                    bail!("locked private storage")
                })
                .is_err()
        );
        assert_eq!(store.load_project(p.id)?.project, p);
        assert!(store.session_reference(p.id, staging.id)?.is_some());
        assert!(!store.delete_confirmed_with_cleanup(&preview, |sessions| {
            assert_eq!(sessions.len(), 1);
            Ok(())
        })?);
        let saved = store.load_project(p.id)?;
        assert_eq!(saved.revision, initial.revision + 1);
        assert_eq!(saved.project.environments.len(), 1);
        assert_eq!(store.selected_environment(&saved.project)?.id, original);
        assert!(store.session_reference(p.id, staging.id)?.is_none());
        assert!(
            store
                .deletion_preview(p.id, DeleteItem::Environment(original))
                .is_err()
        );
        Ok(())
    }
}
