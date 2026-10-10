// SPDX-License-Identifier: MPL-2.0
//! Versioned, conservative URL identity. Never reorder meaningful query pairs.
use crate::domain::*;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use url::Url;

pub const MAX_CANDIDATES: usize = 10_000;
pub fn sources(text: &str) -> Result<Vec<Url>> {
    ensure!(text.len() <= 128 * 1024, "Address list is too long");
    let mut seen = BTreeSet::new();
    let mut urls = Vec::new();
    for entry in text.split_whitespace() {
        ensure!(
            entry.to_ascii_lowercase().starts_with("https://")
                || entry.to_ascii_lowercase().starts_with("http://"),
            "Use complete http:// or https:// starting addresses"
        );
        ensure!(
            !entry.chars().any(char::is_control) && !entry.contains('\\'),
            "Malformed website address"
        );
        let u = Url::parse(entry)?;
        validate_address(&u)?;
        if seen.insert(u.to_string()) {
            urls.push(u);
        }
    }
    ensure!(
        !urls.is_empty() && urls.len() <= 1000,
        "Paste 1–1,000 complete addresses separated by spaces or new lines"
    );
    Ok(urls)
}
pub fn origin(u: &Url) -> String {
    u.origin().ascii_serialization()
}
pub fn canonical(u: &Url, settings: &DiscoverySettings) -> Result<Url> {
    validate_address(u)?;
    ensure!(
        !u.as_str().contains(['\r', '\n', '\\']) && !u.path().to_ascii_lowercase().contains("%00"),
        "Malformed website address"
    );
    let mut result = u.clone();
    if settings.advanced.strip_tracking
        && let Some(query) = u.query()
    {
        let kept: Vec<_> = query
            .split('&')
            .filter(|pair| {
                let key = pair.split('=').next().unwrap_or("");
                let decoded = url::form_urlencoded::parse(key.as_bytes())
                    .next()
                    .map(|p| p.0.to_ascii_lowercase())
                    .unwrap_or_default();
                !decoded.starts_with("utm_")
                    && !["gclid", "fbclid", "msclkid", "_ga", "mc_cid", "mc_eid"]
                        .contains(&decoded.as_str())
            })
            .collect();
        let query = kept.join("&");
        result.set_query(if kept.is_empty() { None } else { Some(&query) });
    }
    if !settings.advanced.preserve_fragments
        && !u
            .fragment()
            .is_some_and(|f| f.starts_with('/') || f.starts_with("!/"))
    {
        result.set_fragment(None);
    }
    Ok(result)
}
pub fn exclusion(u: &Url, project: &Project, depth: u32) -> Option<String> {
    let s = &project.discovery;
    if !project.sources.iter().any(|a| origin(a) == origin(u)) {
        return Some("Website not approved — add and confirm it as a starting address".into());
    }
    if s.advanced.max_depth.is_some_and(|d| depth > d) {
        return Some("Link depth limit".into());
    }
    if let Some(section) = &s.section {
        let section = section.trim_end_matches('/');
        if !section.is_empty()
            && u.path() != section
            && !u.path().starts_with(&format!("{section}/"))
        {
            return Some("Outside the selected website section".into());
        }
    }
    if s.skipped_text
        .iter()
        .any(|t| !t.is_empty() && u.as_str().contains(t))
    {
        return Some("Matches skipped address text".into());
    }
    let path = u.path().to_ascii_lowercase();
    if [
        ".pdf", ".zip", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".mp4", ".exe", ".dmg", ".csv",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
    {
        return Some("Download or non-page link".into());
    }
    if !s.advanced.allow_action_links {
        let query = u.query().unwrap_or("").to_ascii_lowercase();
        let tokens: Vec<_> = path
            .split(['/', '-', '_', '.'])
            .chain(query.split(['&', '=']))
            .collect();
        if tokens.iter().any(|t| {
            [
                "logout",
                "signout",
                "delete",
                "remove",
                "destroy",
                "unsubscribe",
                "action",
            ]
            .contains(t)
        }) {
            return Some(
                "Potentially destructive or sign-out address — visits can have effects".into(),
            );
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryPreview {
    pub project_revision: u64,
    pub settings: Project,
    pub inventory: InventoryRevision,
    pub aliases: Vec<(Url, Url)>,
    pub notes: Vec<String>,
    pub visited: u32,
}
impl DiscoveryPreview {
    pub fn included(&self) -> usize {
        self.inventory.pages.iter().filter(|p| p.included).count()
    }
    pub fn estimate(&self) -> String {
        let targets = self.included() * self.settings.profiles.len();
        format!(
            "{} pages · {} screenshots · planning range {}–{} MiB and {}–{} minutes. Full-page height and website speed can exceed these estimates.",
            self.included(),
            targets,
            targets / 2,
            targets * 8,
            targets / 60 + 1,
            targets / 6 + 1
        )
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetCounts {
    pub captured: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cancelled: usize,
    pub pending: usize,
}
impl TargetCounts {
    pub fn from_run(run: &CaptureRun) -> Self {
        let mut c = Self::default();
        for t in &run.targets {
            match t.outcome {
                TargetOutcome::Captured { .. } => c.captured += 1,
                TargetOutcome::Failed { .. } => c.failed += 1,
                TargetOutcome::Skipped { .. } => c.skipped += 1,
                TargetOutcome::Cancelled => c.cancelled += 1,
                TargetOutcome::Pending => c.pending += 1,
            }
        }
        c
    }
}
#[derive(Debug, Clone)]
pub struct Robots {
    rules: Vec<(bool, String)>,
    pub delay_ms: u64,
}
type RobotsGroup = (Vec<String>, Vec<(bool, String)>, u64);
impl Robots {
    pub fn parse(text: &str) -> Self {
        let mut groups: Vec<RobotsGroup> = Vec::new();
        let mut agents = Vec::new();
        let mut rules = Vec::new();
        let mut delay = 0;
        let mut body = false;
        for line in text.lines() {
            let clean = line.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = clean.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match key.trim().to_ascii_lowercase().as_str() {
                "user-agent" => {
                    if body {
                        groups.push((
                            std::mem::take(&mut agents),
                            std::mem::take(&mut rules),
                            delay,
                        ));
                        delay = 0;
                        body = false;
                    }
                    agents.push(value.to_ascii_lowercase());
                }
                "allow" | "disallow" if !agents.is_empty() => {
                    body = true;
                    if !value.is_empty() {
                        rules.push((key.trim().eq_ignore_ascii_case("allow"), value.into()));
                    }
                }
                "crawl-delay" if !agents.is_empty() => {
                    body = true;
                    delay = value
                        .parse::<f64>()
                        .ok()
                        .filter(|n| n.is_finite() && *n >= 0.)
                        .map(|n| (n * 1000.).min(60_000.) as u64)
                        .unwrap_or(0);
                }
                _ => (),
            }
        }
        groups.push((agents, rules, delay));
        let specific = groups.iter().any(|g| g.0.iter().any(|a| a == "regressee"));
        let mut selected = Vec::new();
        let mut delay_ms = 0;
        for (agents, rules, delay) in groups {
            if agents
                .iter()
                .any(|a| a == if specific { "regressee" } else { "*" })
            {
                selected.extend(rules);
                delay_ms = delay_ms.max(delay);
            }
        }
        Self {
            rules: selected,
            delay_ms,
        }
    }
    pub fn allows(&self, u: &Url) -> bool {
        let path = format!(
            "{}{}",
            u.path(),
            u.query().map(|q| format!("?{q}")).unwrap_or_default()
        );
        self.rules
            .iter()
            .filter(|(_, p)| robot_match(p, &path))
            .max_by_key(|(allow, p)| {
                (
                    p.strip_suffix('$')
                        .unwrap_or(p)
                        .bytes()
                        .filter(|c| *c != b'*')
                        .count(),
                    *allow,
                )
            })
            .is_none_or(|(allow, _)| *allow)
    }
}
fn robot_match(pattern: &str, text: &str) -> bool {
    let anchored = pattern.ends_with('$');
    let pattern = if anchored {
        &pattern[..pattern.len() - 1]
    } else {
        pattern
    };
    // Byte-wise glob: '*' consumes zero or more bytes; only the final '$'
    // anchors the end. Matching always starts at the path's first byte. Keep
    // encoded reserved bytes and query order literal (no regex metacharacters).
    let (p, t) = (pattern.as_bytes(), text.as_bytes());
    let (mut i, mut j, mut star, mut retry) = (0, 0, None, 0);
    while j < t.len() {
        if i == p.len() && !anchored {
            return true;
        }
        if i < p.len() && p[i] == b'*' {
            star = Some(i);
            i += 1;
            retry = j;
        } else if i < p.len() && p[i] == t[j] {
            i += 1;
            j += 1;
        } else if let Some(s) = star {
            retry += 1;
            j = retry;
            i = s + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == b'*' {
        i += 1;
    }
    i == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn robots_wildcards_anchors_literals_queries_and_precedence() {
        for (pattern, text, expected) in [
            ("/private/*$", "/private/secret", true),
            ("/private/*$", "/private/", true),
            ("/private/*$", "/public/private/secret", false),
            ("/a*b$", "/axbb", true), // must backtrack past the first b
            ("/a*b$", "/axbc", false),
            ("/a**b*c$", "/abc", true),
            ("/a*b", "/abc/more", true),
            ("/a$", "/a?x=1", false),
            ("/*?token=*$", "/a?token=secret&x=1", true),
            ("/a.b[1]+$", "/a.b[1]+", true),
            ("/a.b[1]+$", "/axb1", false),
            ("/encoded/%2F*$", "/encoded/%2Fsecret?q=%26", true),
            ("/encoded/%2F*$", "/encoded//secret", false),
            ("/literal%2A$", "/literal*", false),
            ("/cash$$", "/cash$", true),
        ] {
            assert_eq!(
                robot_match(pattern, text),
                expected,
                "{pattern} against {text}"
            );
        }
        let r = Robots::parse(
            "User-agent: *\nDisallow: /private/*$\nAllow: /private/public\nDisallow: /same*\nAllow: /same*\nDisallow: /*?token=*$\nAllow: /ok?token=public$\nDisallow: /encoded/%2F*$\n",
        );
        for (path, allowed) in [
            ("/private/secret", false),
            ("/private/public/a", true),
            ("/same/a", true),
            ("/a?token=secret", false),
            ("/ok?token=public", true),
            ("/ok?token=public&x=1", false),
            ("/encoded/%2Fsecret?q=%26", false),
            ("/encoded//secret", true),
        ] {
            assert_eq!(
                r.allows(&Url::parse(&format!("https://x.test{path}")).unwrap()),
                allowed,
                "{path}"
            );
        }
    }
    #[test]
    fn identity_preserves_behavior() {
        let s = DiscoverySettings::default();
        for (raw, expected) in [
            (
                "https://x.test/a/?b=2&a=1&a=0&utm_source=x#heading",
                "https://x.test/a/?b=2&a=1&a=0",
            ),
            (
                "https://x.test/%2F?a=%20&fbclid=x#/route",
                "https://x.test/%2F?a=%20#/route",
            ),
            ("https://x.test/# !/x", "https://x.test/"),
        ] {
            assert_eq!(
                canonical(&Url::parse(raw).unwrap(), &s).unwrap().as_str(),
                expected
            );
        }
        assert_ne!(
            canonical(&Url::parse("https://x.test/a").unwrap(), &s).unwrap(),
            canonical(&Url::parse("https://x.test/a/").unwrap(), &s).unwrap()
        );
        assert!(sources("file:///etc/passwd").is_err());
        assert!(sources("https://user:secret@x.test").is_err());
        assert!(sources("https://x.test\\admin").is_err());
    }
    #[test]
    fn boundaries_and_robots_are_explainable() {
        let mut p = Project::draft("Site".into(), "https://x.test/docs").unwrap();
        p.discovery.section = Some("/docs".into());
        assert!(exclusion(&Url::parse("https://x.test/docsevil").unwrap(), &p, 0).is_some());
        assert!(exclusion(&Url::parse("https://other.test/docs").unwrap(), &p, 0).is_some());
        assert!(
            exclusion(
                &Url::parse("https://x.test/docs/delete?id=3").unwrap(),
                &p,
                0
            )
            .is_some()
        );
        let r = Robots::parse(
            "User-agent: *\nDisallow: /docs\nAllow: /docs/public\nCrawl-delay: 2\nDisallow: /*?secret=*\n",
        );
        assert!(!r.allows(&Url::parse("https://x.test/docs/private").unwrap()));
        assert!(r.allows(&Url::parse("https://x.test/docs/public").unwrap()));
        assert_eq!(r.delay_ms, 2000);
        p.discovery.respect_robots = false;
        assert!(p.validate().is_err());
        p.discovery.advanced.robots_authorized = true;
        p.validate().unwrap();
    }
    #[test]
    fn old_settings_get_safe_defaults() {
        let s: DiscoverySettings = serde_json::from_str(r#"{"mode":"OnlyAddresses","page_limit":10,"section":null,"skipped_text":[],"speed":"Gentle","respect_robots":true}"#).unwrap();
        assert_eq!(s.advanced, CrawlAdvanced::default());
    }
}
