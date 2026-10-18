// SPDX-License-Identifier: MPL-2.0
//! Isolated old-wire lifecycle peers; never operate on the user's engine/store.
use super::*;
use std::thread;

struct Fixture {
    _root: tempfile::TempDir,
    _owner: ownership::Ownership,
    client: Client,
    engine: Uuid,
}
impl Fixture {
    fn new(version: u32) -> Result<Self> {
        let root = tempfile::tempdir()?;
        let owner = ownership::Ownership::acquire(&AppPaths::isolated(root.path()))?;
        let engine = Uuid::new_v4();
        ownership::write_json(
            &owner.directory.join("endpoint.json"),
            &serde_json::json!({
                "version":version,"engine":engine,"cache":owner.paths.cache,
            }),
        )?;
        let client = Client::existing(owner.paths.clone())?;
        Ok(Self {
            _root: root,
            _owner: owner,
            client,
            engine,
        })
    }
    fn peer(
        &self,
        version: u32,
        engine: Uuid,
        response: Response,
    ) -> thread::JoinHandle<Result<bool>> {
        let paths = self.client.paths.clone();
        let (ready, wait) = std::sync::mpsc::channel();
        let task = thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    let mut listener = ipc::Listener::bind(&paths)?;
                    ready.send(())?;
                    let mut stream = listener.accept().await?;
                    let hello: Request = ipc::read(&mut stream).await?;
                    ensure!(
                        matches!(hello,Request::Hello {version:v} if v==version),
                        "Wrong negotiated version"
                    );
                    ipc::write(&mut stream, &Response::Hello { version, engine }).await?;
                    match ipc::read::<Request>(&mut stream).await {
                        Ok(Request::Stop) => {
                            ipc::write(&mut stream, &response).await?;
                            Ok(true)
                        }
                        Err(_) => Ok(false),
                        other => anyhow::bail!("Unexpected old-wire command: {other:?}"),
                    }
                })
        });
        wait.recv_timeout(Duration::from_secs(5)).unwrap();
        task
    }
}

#[test]
fn upgrade_stop_is_explicit_and_only_uses_the_known_lifecycle_envelope() -> Result<()> {
    for version in [6, 7, 8] {
        let f = Fixture::new(version)?;
        let error = f.client.snapshot().unwrap_err().to_string();
        assert!(error.contains("version_mismatch") && error.contains("regressee engine stop"));
        let endpoint = f.client.paths.data.join("engine-v1/endpoint.json");
        let before = std::fs::read(&endpoint)?;
        assert!(
            Client::autostart(
                f.client.paths.clone(),
                Launcher {
                    executable: f.client.paths.data.join("must-not-be-launched")
                }
            )
            .is_err()
        );
        assert_eq!(std::fs::read(endpoint)?, before);
        let peer = f.peer(version, f.engine, Response::Ok);
        assert!(matches!(f.client.request(Request::Stop)?, Response::Ok));
        assert!(peer.join().unwrap()?);
    }
    Ok(())
}

#[test]
fn upgrade_stop_retains_busy_refusal_and_checks_identity_and_cache() -> Result<()> {
    for version in [7, 8] {
        for case in ["busy", "identity", "cache"] {
            let f = Fixture::new(version)?;
            let engine = if case == "identity" {
                Uuid::new_v4()
            } else {
                f.engine
            };
            if case == "cache" {
                ownership::write_json(
                    &f._owner.directory.join("endpoint.json"),
                    &serde_json::json!({
                        "version":version,"engine":f.engine,"cache":f.client.paths.data.join("other-cache"),
                    }),
                )?;
            }
            let peer = f.peer(
                version,
                engine,
                Response::Error {
                    code: "busy".into(),
                    message: "Jobs, login or agent still need the engine".into(),
                },
            );
            let result = f.client.request(Request::Stop);
            if case == "busy" {
                assert!(matches!(result?,Response::Error {code,..} if code=="busy"));
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains(if case == "identity" {
                            "identity_mismatch"
                        } else {
                            "cache_directory_conflict"
                        })
                );
            }
            assert_eq!(peer.join().unwrap()?, case == "busy");
            assert!(ownership::Ownership::acquire(&f.client.paths).is_err());
        }
    }
    Ok(())
}

#[test]
fn upgrade_stop_does_not_guess_unknown_past_or_future_protocols() -> Result<()> {
    for version in [1, 2, 3, 4, 5, 999] {
        let f = Fixture::new(version)?;
        assert!(
            f.client
                .request(Request::Stop)
                .unwrap_err()
                .to_string()
                .contains("version_mismatch")
        );
    }
    Ok(())
}
