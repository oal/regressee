// SPDX-License-Identifier: MPL-2.0
//! Authenticated bundle links are entries, never extraction paths. Preserve
//! framework aliases and signed file modes; do not materialize signed Mac code.
use crate::distribution::{Manifest, safe_name};
use anyhow::{Context, Result, ensure};
use std::collections::BTreeSet;
pub fn target(name: &str, value: &str) -> Result<String> {
    ensure!(
        !value.is_empty()
            && value.len() <= 1024
            && !value.starts_with('/')
            && !value
                .chars()
                .any(|c| c.is_control() || "\\:<>\"|?*".contains(c)),
        "Unsafe bundle link target"
    );
    let mut parts: Vec<_> = name.split('/').collect();
    parts.pop();
    for part in value.split('/') {
        match part {
            ".." => {
                ensure!(parts.pop().is_some(), "Bundle link escapes root");
            }
            "." => (),
            "" => anyhow::bail!("Empty bundle link component"),
            v => parts.push(v),
        }
    }
    let result = parts.join("/");
    ensure!(safe_name(&result), "Unsafe resolved bundle path");
    Ok(result)
}
pub fn validate(m: &Manifest) -> Result<()> {
    if m.version == 2 {
        ensure!(
            m.links.is_empty() && m.modes.is_empty(),
            "Legacy manifest cannot carry bundle entries"
        );
        return Ok(());
    }
    ensure!(
        m.version == 3 && m.links.len() <= 4096 && m.files.len() + m.links.len() <= 20000,
        "Unsupported bundle entry budget"
    );
    ensure!(
        m.modes.len() == m.files.len(),
        "Complete signed file modes required"
    );
    for (name, e) in &m.files {
        let mode = m.modes.get(name).context("Missing signed file mode")?;
        ensure!(
            *mode & !0o755 == 0 && (*mode & 0o400 != 0) && (mode & 0o111 != 0) == e.executable,
            "Unsafe or inconsistent signed mode"
        );
    }
    let mut names = BTreeSet::new();
    for name in m.files.keys().chain(m.links.keys()) {
        ensure!(
            safe_name(name)
                && !matches!(name.as_str(), "manifest.json" | "manifest.sig" | "READY")
                && names.insert(name.to_lowercase()),
            "Colliding bundle path"
        );
    }
    let mut dirs = std::collections::BTreeMap::new();
    for name in m.files.keys().chain(m.links.keys()) {
        let mut prefix = String::new();
        for part in name.split('/').take(name.split('/').count() - 1) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            ensure!(
                !names.contains(&prefix.to_lowercase()),
                "Bundle leaf used as directory"
            );
            if let Some(old) = dirs.insert(prefix.to_lowercase(), prefix.clone()) {
                ensure!(old == prefix, "Case-colliding bundle directory");
            }
        }
    }
    for (name, value) in &m.links {
        // Outer .app/.framework boundary; arbitrary npm aliases unsupported.
        let parts: Vec<_> = name.split('/').collect();
        let end = parts
            .iter()
            .position(|p| p.ends_with(".app") || p.ends_with(".framework"))
            .context("Link outside signed bundle")?;
        let boundary = parts[..=end].join("/") + "/";
        ensure!(
            name.starts_with(&(m.browser_root.clone() + "/")),
            "Link outside browser tree"
        );
        let mut resolved = target(name, value)?;
        let mut visited = BTreeSet::new();
        loop {
            ensure!(
                resolved.starts_with(&boundary)
                    && visited.insert(resolved.clone())
                    && visited.len() <= 32,
                "Bundle link escape/cycle/depth"
            );
            let prefix = m
                .links
                .keys()
                .find(|p| resolved == **p || resolved.starts_with(&format!("{p}/")));
            if let Some(prefix) = prefix {
                let suffix = resolved[prefix.len()..].to_owned();
                resolved = target(prefix, &m.links[prefix])? + &suffix;
            } else {
                break;
            }
        }
        ensure!(
            m.files.contains_key(&resolved) || dirs.values().any(|d| d == &resolved),
            "Dangling bundle link"
        );
    }
    Ok(())
}
