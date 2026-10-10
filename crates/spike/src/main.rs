// SPDX-License-Identifier: MPL-2.0
use anyhow::{Result, bail};
use regressee_spike::{images, runtime, supervisor::Worker};
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("pack") => runtime::pack(Path::new(".runtime/source"), Path::new(".runtime/pack.zip")),
        Some("repair") => {
            let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            if let Some(index) = args.iter().position(|s| s == "--cancel-after-ms") {
                let ms: u64 = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing milliseconds"))?
                    .parse()?;
                let c = cancel.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                    c.store(true, std::sync::atomic::Ordering::Relaxed);
                });
            }
            runtime::install_cancellable(
                Path::new(".runtime/pack.zip"),
                Path::new(".runtime/installed"),
                cancel,
            )
        }
        Some("tiles") => {
            let input = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("tiles INPUT.png"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&images::tile(
                    Path::new(input),
                    Path::new("artifacts/tiles")
                )?)?
            );
            Ok(())
        }
        Some("smoke") => {
            let mut worker = Worker::spawn(Path::new(".runtime/installed"))?;
            println!("{}", worker.request("handshake", serde_json::json!({}))?);
            println!(
                "{}",
                worker.request(
                    "selfTest",
                    serde_json::json!({"headed": args.iter().any(|a| a == "--headed")})
                )?
            );
            worker.shutdown()
        }
        Some("protocol-test") => {
            regressee_spike::supervisor::fault_tests(Path::new(".runtime/installed"))
        }
        #[cfg(target_os = "linux")]
        Some("actor-test") => {
            regressee_spike::actor::lifecycle_tests(Path::new(".runtime/installed"))
        }
        Some("parent-hang") => {
            let mut worker = Worker::spawn(Path::new(".runtime/installed"))?;
            worker.request("handshake", serde_json::json!({}))?;
            println!("{}", worker.pid());
            worker.request("hang", serde_json::json!({}))?;
            Ok(())
        }
        _ => bail!(
            "Phase 0 commands: pack | repair | smoke [--headed] | protocol-test | tiles INPUT.png"
        ),
    }
}
