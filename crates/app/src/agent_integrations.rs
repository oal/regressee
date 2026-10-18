// SPDX-License-Identifier: MPL-2.0
//! User-local skill installation. No engine, browser, host process or permission edits.
use anyhow::{Context, Result, bail, ensure};
use regressee_core::platform::ordinary;
use regressee_storage::AppPaths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const MANIFEST: &str = ".regressee-install.json";
const FILES: &[(&str, &str)] = &[
    (
        "SKILL.md",
        include_str!("../../../skills/regressee/SKILL.md"),
    ),
    (
        "references/commands.md",
        include_str!("../../../skills/regressee/references/commands.md"),
    ),
    (
        "references/workflows.md",
        include_str!("../../../skills/regressee/references/workflows.md"),
    ),
    (
        "references/safety-and-evidence.md",
        include_str!("../../../skills/regressee/references/safety-and-evidence.md"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Host {
    ClaudeCode,
    Codex,
    GithubCopilot,
}
impl Host {
    pub const ALL: [Self; 3] = [Self::ClaudeCode, Self::Codex, Self::GithubCopilot];
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "claude-code" => Ok(Self::ClaudeCode),
            "codex" => Ok(Self::Codex),
            "github-copilot" => Ok(Self::GithubCopilot),
            _ => bail!("invalid_arguments: host must be claude-code, codex or github-copilot"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::GithubCopilot => "GitHub Copilot",
        }
    }
    fn directory(self) -> &'static str {
        match self {
            Self::ClaudeCode => ".claude",
            Self::Codex => ".agents",
            Self::GithubCopilot => ".copilot",
        }
    }
    pub fn next_step(self) -> &'static str {
        match self {
            Self::ClaudeCode => {
                "Open Claude Code and ask it to use /regressee. Reload skills or restart if it is not listed."
            }
            Self::Codex => {
                "Open Codex and ask it to use $regressee. Restart if the skill is not listed."
            }
            Self::GithubCopilot => {
                "Open a local Copilot agent session and ask it to use the regressee skill. Restart if it is not listed."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    NotInstalled,
    Installed,
    UpdateAvailable,
    Conflict,
    Unavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub host: Host,
    pub state: State,
    pub path: PathBuf,
    pub message: String,
    pub removable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Connection {
    executable: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    cli_schema_version: u32,
    protocol_version: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    owner: String,
    schema: u32,
    files: BTreeMap<String, String>,
}

pub struct Installer {
    home: PathBuf,
    connection: Connection,
}
impl Installer {
    /// Resolve only the user's real home; never infer it from the project directory.
    pub fn for_user(paths: &AppPaths, executable: PathBuf) -> Result<Self> {
        #[cfg(windows)]
        let home = std::env::var_os("USERPROFILE");
        #[cfg(not(windows))]
        let home = std::env::var_os("HOME");
        Self::new(
            PathBuf::from(
                home.context("Home folder is unavailable; no agent files were changed.")?,
            ),
            paths,
            executable,
        )
    }
    pub fn new(home: PathBuf, paths: &AppPaths, executable: PathBuf) -> Result<Self> {
        ensure!(home.is_absolute(), "Home folder must be an absolute path");
        let home = home
            .canonicalize()
            .context("Cannot read your home folder")?;
        let absolute = |p: &Path| -> Result<PathBuf> {
            let p = std::path::absolute(p)?;
            ensure!(
                p.to_str().is_some(),
                "Installation paths must be valid Unicode"
            );
            Ok(p)
        };
        Ok(Self {
            home,
            connection: Connection {
                executable: absolute(&executable)?,
                data_dir: absolute(&paths.data)?,
                cache_dir: absolute(&paths.cache)?,
                cli_schema_version: 1,
                protocol_version: crate::protocol::VERSION,
            },
        })
    }
    pub fn target(&self, host: Host) -> PathBuf {
        self.home.join(host.directory()).join("skills/regressee")
    }
    fn content(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        let mut files = FILES
            .iter()
            .map(|(name, text)| (name.to_string(), text.as_bytes().to_vec()))
            .collect::<BTreeMap<_, _>>();
        let skill = FILES[0].1.replace("## Start\n", "## Local installation\n\nRead [connection.json](connection.json) before running commands. Use its exact executable, data_dir and cache_dir for every command, passing paths as individual arguments (quote them for your shell). Do not substitute another executable or infer paths from the current repository. These paths are configuration, not permission to access unrelated projects.\n\nWhen the user wants to watch your work, start a named renewable agent session, pass --session to mutations and use agent select with the exact project/run/page IDs as your investigation moves. The human must explicitly choose Follow agent; installation never opts them in.\n\n## Start\n");
        files.insert("SKILL.md".into(), skill.into_bytes());
        files.insert(
            "connection.json".into(),
            serde_json::to_vec_pretty(&self.connection)?,
        );
        Ok(files)
    }
    fn executable_ready(&self) -> bool {
        fs::metadata(&self.connection.executable).is_ok_and(|m| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                m.is_file() && m.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                m.is_file()
            }
        })
    }
    pub fn status(&self, host: Host) -> Status {
        let path = self.target(host);
        let inspect = || -> Result<State> {
            self.check_parents(host, false)?;
            match fs::symlink_metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::NotInstalled),
                Err(e) => Err(e.into()),
                Ok(_) => {
                    let actual = read_managed(&path)?;
                    Ok(if actual == self.content()? {
                        State::Installed
                    } else {
                        State::UpdateAvailable
                    })
                }
            }
        };
        let inspected = inspect();
        let removable = matches!(&inspected, Ok(State::Installed | State::UpdateAvailable));
        let (state, message) = match inspected {
            Ok(state) if !self.executable_ready() && state != State::Installed => (State::Unavailable, "The Regressee CLI is missing or cannot run. Keep regressee beside regressee-desktop, then check again.".into()),
            Ok(State::NotInstalled) => (State::NotInstalled, "Not installed".into()),
            Ok(State::Installed) if !self.executable_ready() => (State::Unavailable, "The installed skill points to an unavailable CLI. Restore the executable, then check again.".into()),
            Ok(State::Installed) => (State::Installed, "Skill files verified. Host access is not tested.".into()),
            Ok(State::UpdateAvailable) => (State::UpdateAvailable, "Update the skill or its local connection paths.".into()),
            Ok(state) => (state, String::new()),
            Err(e) => (State::Conflict, format!("{e:#}. Existing files were kept.")),
        };
        Status {
            host,
            state,
            path,
            message,
            removable,
        }
    }
    pub fn statuses(&self) -> Vec<Status> {
        Host::ALL.into_iter().map(|h| self.status(h)).collect()
    }
    fn check_parents(&self, host: Host, create: bool) -> Result<()> {
        let mut path = self.home.clone();
        for component in [host.directory(), "skills"] {
            path.push(component);
            match fs::symlink_metadata(&path) {
                Ok(m) => ensure!(
                    ordinary(&m) && m.is_dir(),
                    "{} is not an ordinary folder; linked folders are not changed",
                    path.display()
                ),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    if create {
                        fs::create_dir(&path)?;
                    } else {
                        return Ok(());
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    /// Stage all files before publishing; refuse unknown/edited installations.
    pub fn install(&self, host: Host) -> Result<Status> {
        ensure!(
            self.executable_ready(),
            "The Regressee CLI is missing or cannot run; no skill was installed"
        );
        self.check_parents(host, true)?;
        let path = self.target(host);
        let parent = path.parent().unwrap();
        let _lock = InstallLock::take(parent)?;
        let previous = match fs::symlink_metadata(&path) {
            Ok(_) => Some(
                read_managed(&path)
                    .context("Cannot replace this skill; existing files were kept")?,
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let files = self.content()?;
        if previous.as_ref() == Some(&files) {
            return Ok(self.status(host));
        }
        let stage = tempfile::Builder::new()
            .prefix(".regressee-stage-")
            .tempdir_in(parent)?;
        fs::create_dir(stage.path().join("references"))?;
        for (name, bytes) in &files {
            fs::write(stage.path().join(name), bytes)?;
        }
        let manifest = Manifest {
            owner: "regressee".into(),
            schema: 1,
            files: hashes(&files),
        };
        fs::write(
            stage.path().join(MANIFEST),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        read_managed(stage.path())?;
        let backup = parent.join(format!(".regressee-backup-{}", uuid::Uuid::new_v4()));
        if previous.is_some() {
            fs::rename(&path, &backup)?;
        }
        if let Err(e) = fs::rename(stage.path(), &path) {
            if previous.is_some() {
                fs::rename(&backup, &path).with_context(|| format!("Could not restore the previous skill from {} after failed installation: {e}", backup.display()))?;
            }
            return Err(e).context("Could not publish the skill; previous files were restored");
        }
        let mut status = self.status(host);
        if previous.is_some()
            && let Err(e) = remove_managed(&backup)
        {
            status.message = format!(
                "Skill installed. Previous files remain at {}: {e:#}",
                backup.display()
            );
        }
        Ok(status)
    }
    pub fn remove(&self, host: Host) -> Result<Status> {
        self.check_parents(host, false)?;
        let path = self.target(host);
        if fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
            return Ok(self.status(host));
        }
        let _lock = InstallLock::take(path.parent().unwrap())?;
        read_managed(&path).context(
            "Cannot remove this skill; edited or unrelated files are never removed automatically",
        )?;
        let removed = path
            .parent()
            .unwrap()
            .join(format!(".regressee-removed-{}", uuid::Uuid::new_v4()));
        fs::rename(&path, &removed)
            .context("Could not remove the skill from the host's search folder")?;
        let mut status = self.status(host);
        if let Err(e) = remove_managed(&removed) {
            status.message = format!(
                "Skill removed from the host's search folder. Cleanup remains at {}: {e:#}",
                removed.display()
            );
        }
        Ok(status)
    }
}

struct InstallLock(PathBuf);
impl InstallLock {
    fn take(parent: &Path) -> Result<Self> {
        let path = parent.join(".regressee-install.lock");
        fs::create_dir(&path).context("Agent setup is locked. If another setup crashed, inspect and remove the empty .regressee-install.lock folder before retrying")?;
        Ok(Self(path))
    }
}
impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}
fn hashes(files: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    files
        .iter()
        .map(|(n, b)| (n.clone(), hex::encode(Sha256::digest(b))))
        .collect()
}
fn read_file(path: &Path, max: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let m = fs::symlink_metadata(path)?;
    ensure!(
        ordinary(&m) && m.is_file() && m.len() <= max,
        "{} is linked, unexpected or too large",
        path.display()
    );
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(max + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= max,
        "{} grew beyond the installation limit",
        path.display()
    );
    Ok(bytes)
}
fn read_managed(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let m = fs::symlink_metadata(path)?;
    ensure!(
        ordinary(&m) && m.is_dir(),
        "Skill folder is linked or is not an ordinary folder"
    );
    let manifest: Manifest = serde_json::from_slice(
        &read_file(&path.join(MANIFEST), 16384)
            .context("This folder is not a managed Regressee installation")?,
    )?;
    let expected = FILES
        .iter()
        .map(|(n, _)| n.to_string())
        .chain(["connection.json".into()])
        .collect::<std::collections::BTreeSet<_>>();
    ensure!(
        manifest.owner == "regressee"
            && manifest.schema == 1
            && manifest
                .files
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>()
                == expected,
        "Unrecognized installation record"
    );
    let refs = fs::symlink_metadata(path.join("references"))?;
    ensure!(
        ordinary(&refs) && refs.is_dir(),
        "Skill references folder is linked or unexpected"
    );
    // Exact inventory: updates/removal never discard a user's additional files.
    for (dir, allowed) in [
        (
            path.to_path_buf(),
            vec![MANIFEST, "SKILL.md", "connection.json", "references"],
        ),
        (
            path.join("references"),
            vec!["commands.md", "workflows.md", "safety-and-evidence.md"],
        ),
    ] {
        for entry in fs::read_dir(dir)? {
            let name = entry?.file_name();
            ensure!(
                name.to_str().is_some_and(|n| allowed.contains(&n)),
                "The skill contains additional files; keep or move them before updating or removing it"
            );
        }
    }
    let files = expected
        .into_iter()
        .map(|name| Ok((name.clone(), read_file(&path.join(name), 262144)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    ensure!(
        hashes(&files) == manifest.files,
        "The skill has local edits; keep or move it before updating or removing it"
    );
    Ok(files)
}
fn remove_managed(path: &Path) -> Result<()> {
    let files = read_managed(path)?;
    for name in files.keys() {
        fs::remove_file(path.join(name))?;
    }
    fs::remove_file(path.join(MANIFEST))?;
    fs::remove_dir(path.join("references"))?;
    fs::remove_dir(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Installer) {
        let dir = tempfile::tempdir().unwrap();
        let exe = std::env::current_exe().unwrap();
        let installer = Installer::new(
            dir.path().into(),
            &AppPaths::isolated(&dir.path().join("data with spaces")),
            exe,
        )
        .unwrap();
        (dir, installer)
    }
    #[test]
    fn all_hosts_roundtrip_and_local_connection() {
        let (_dir, i) = fixture();
        for host in Host::ALL {
            assert_eq!(i.status(host).state, State::NotInstalled);
            assert!(!i.target(host).exists(), "status must not create files");
            assert_eq!(i.install(host).unwrap().state, State::Installed);
            assert_eq!(i.install(host).unwrap().state, State::Installed);
            let c: Connection =
                serde_json::from_slice(&fs::read(i.target(host).join("connection.json")).unwrap())
                    .unwrap();
            assert_eq!(c, i.connection);
            let skill = fs::read_to_string(i.target(host).join("SKILL.md")).unwrap();
            assert!(skill.contains("connection.json") && skill.contains("Follow agent"));
            assert_eq!(i.remove(host).unwrap().state, State::NotInstalled);
        }
    }
    #[test]
    fn edits_foreign_files_and_extra_files_are_kept() {
        for kind in ["edited", "extra", "foreign"] {
            let (_dir, i) = fixture();
            let h = Host::Codex;
            i.install(h).unwrap();
            let path = i.target(h);
            match kind {
                "edited" => fs::write(path.join("SKILL.md"), "My own skill").unwrap(),
                "extra" => fs::write(path.join("my-notes.txt"), "Keep this").unwrap(),
                _ => fs::remove_file(path.join(MANIFEST)).unwrap(),
            }
            assert_eq!(i.status(h).state, State::Conflict);
            let before = fs::read(path.join("SKILL.md")).unwrap();
            assert!(i.install(h).is_err());
            assert!(i.remove(h).is_err());
            assert_eq!(fs::read(path.join("SKILL.md")).unwrap(), before);
        }
    }
    #[test]
    fn new_connection_requires_explicit_update_and_locks_fail_closed() {
        let (_dir, mut i) = fixture();
        let h = Host::ClaudeCode;
        i.install(h).unwrap();
        i.connection.cache_dir.push("new cache");
        assert_eq!(i.status(h).state, State::UpdateAvailable);
        let lock = InstallLock::take(i.target(h).parent().unwrap()).unwrap();
        assert!(i.install(h).is_err());
        drop(lock);
        assert_eq!(i.install(h).unwrap().state, State::Installed);
    }
    #[test]
    fn absent_cli_never_installs_and_installed_files_can_still_be_removed() {
        let (_dir, mut i) = fixture();
        i.install(Host::Codex).unwrap();
        i.connection.executable = i.home.join("missing-cli");
        assert_eq!(i.status(Host::Codex).state, State::Unavailable);
        assert!(i.install(Host::ClaudeCode).is_err());
        i.remove(Host::Codex).unwrap();
        assert!(!i.target(Host::Codex).exists());
    }
    #[cfg(unix)]
    #[test]
    fn linked_parent_target_and_reference_are_never_followed() {
        use std::os::unix::fs::symlink;
        for kind in ["parent", "target", "reference"] {
            let (dir, i) = fixture();
            let outside = dir.path().join("outside");
            fs::create_dir(&outside).unwrap();
            fs::write(outside.join("keep"), "untouched").unwrap();
            let h = Host::ClaudeCode;
            match kind {
                "parent" => symlink(&outside, dir.path().join(".claude")).unwrap(),
                "target" => {
                    i.check_parents(h, true).unwrap();
                    symlink(&outside, i.target(h)).unwrap();
                }
                _ => {
                    i.install(h).unwrap();
                    fs::remove_file(i.target(h).join("references/commands.md")).unwrap();
                    symlink(
                        outside.join("keep"),
                        i.target(h).join("references/commands.md"),
                    )
                    .unwrap();
                }
            }
            assert_eq!(i.status(h).state, State::Conflict);
            assert!(i.install(h).is_err());
            assert!(i.remove(h).is_err());
            assert_eq!(
                fs::read_to_string(outside.join("keep")).unwrap(),
                "untouched"
            );
        }
    }
}
