// SPDX-License-Identifier: MPL-2.0
//! Native pipe contention; isolated names, no user's engine or storage.
use super::*;
use tokio::net::windows::named_pipe::ClientOptions;

#[test]
fn busy_pipe_recovers_when_listener_replenishes_its_instance() -> Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::isolated(root.path());
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let mut listener = ipc::Listener::bind(&paths)?;
            let name = format!(r"\\.\pipe\regressee-engine-{}", ipc::key(&paths));
            // Occupy the only listening instance before the server accepts it.
            let _occupied = ClientOptions::new().open(&name)?;
            assert_eq!(
                ClientOptions::new().open(&name).unwrap_err().raw_os_error(),
                Some(231)
            );
            let waiting = tokio::spawn(async move { ipc::connect(&paths).await });
            tokio::time::sleep(Duration::from_millis(30)).await;
            assert!(
                !waiting.is_finished(),
                "busy must wait, not fail immediately"
            );

            let _first = listener.accept().await?;
            let mut client = tokio::time::timeout(ipc::IO_TIMEOUT, waiting).await???;
            let mut server = listener.accept().await?;
            ipc::write(&mut client, &Request::Status).await?;
            assert!(matches!(ipc::read(&mut server).await?, Request::Status));
            Ok(())
        })
}
