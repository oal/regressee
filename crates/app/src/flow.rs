// SPDX-License-Identifier: MPL-2.0
//! Headless entry to the same product services (not the fixture spike).
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(
        args.len() >= 3,
        "usage: regressee-flow DATA_ROOT setup|status|new URL|capture PROJECT_ID|check PROJECT_ID|activate RUN_ID REVISION|evidence PROJECT_ID"
    );
    let paths = regressee_storage::AppPaths::isolated(std::path::Path::new(&args[1]));
    let mut store = regressee_storage::Store::open(paths.clone())?;
    store.recover_runs()?;
    let mut product = regressee_app::product::Product::configured(&paths.cache)?;
    // Explicit headless-verifier input only; still fully signature/inventory/
    // scope verified. Native product settings never expose executable paths.
    if let Some(cache) = std::env::var_os("REGRESSEE_TEST_RUNTIME_CACHE") {
        product.runtime_cache = cache.into();
    }
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    match args[2].as_str() {
        "setup" => println!(
            "{}",
            product.setup(cancel, &mut |bytes, total| eprintln!(
                "received {bytes} / {total:?}"
            ))?
        ),
        "status" => println!("{}", product.status(&cancel)?),
        "new" => {
            let p = regressee_core::domain::Project::draft(
                "Single-page preview".into(),
                args.get(3).ok_or_else(|| anyhow::anyhow!("URL required"))?,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.save_project(&p, None)?)?
            );
        }
        "capture" | "check" => {
            let id = serde_json::from_value(serde_json::json!(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("project ID required"))?
            ))?;
            let r = product.capture(
                &mut store,
                id,
                args[2] == "check",
                cancel,
                &mut |message, _, _| eprintln!("{message}"),
            )?;
            println!("{}", serde_json::to_string_pretty(&r)?);
        }
        "resume" => {
            let run = serde_json::from_value(serde_json::json!(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("run ID required"))?
            ))?;
            let record = store.load_run(run)?;
            let r = product.capture_inventory(
                &mut store,
                record.run.project,
                None,
                false,
                Some(run),
                cancel,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                &mut |message, _, _| eprintln!("{message}"),
            )?;
            println!("{}", serde_json::to_string_pretty(&r)?);
        }
        "activate" => {
            let id = serde_json::from_value(serde_json::json!(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("run ID required"))?
            ))?;
            let revision = args
                .get(4)
                .ok_or_else(|| anyhow::anyhow!("revision required"))?
                .parse()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&product.activate(&mut store, id, revision)?)?
            );
        }
        "evidence" => {
            let id = serde_json::from_value(serde_json::json!(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("project ID required"))?
            ))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&store.project_evidence(id)?)?
            );
        }
        _ => anyhow::bail!("unknown command"),
    }
    Ok(())
}
