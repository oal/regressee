// SPDX-License-Identifier: MPL-2.0
//! Site mappings preserve logical identity; they are not rendering mismatches.
use crate::domain::*;
use anyhow::{Result, ensure};
use url::Url;

pub fn validate_mapping(m: &OriginMapping) -> Result<()> {
    for u in [&m.source, &m.destination] {
        validate_address(u)?;
        ensure!(
            u.query().is_none() && u.fragment().is_none(),
            "Mapping bases must not contain query parameters or fragments"
        );
    }
    Ok(())
}
pub fn map_url(address: &Url, environment: &Environment) -> Result<Url> {
    validate_address(address)?;
    let mut matches = Vec::new();
    for m in &environment.mappings {
        validate_mapping(m)?;
        let prefix = m.source.path().trim_end_matches('/');
        if address.origin() == m.source.origin()
            && (prefix.is_empty()
                || address.path() == prefix
                || address.path().starts_with(&format!("{prefix}/")))
        {
            matches.push((prefix.len(), m));
        }
    }
    matches.sort_by_key(|(len, _)| std::cmp::Reverse(*len));
    let (length, m) = matches
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No approved mapping for this saved page"))?;
    ensure!(
        matches.get(1).is_none_or(|(len, _)| *len != length),
        "Ambiguous mappings; approve one destination per source prefix"
    );
    let mut mapped = m.destination.clone();
    mapped.set_path(&format!(
        "{}{}",
        m.destination.path().trim_end_matches('/'),
        &address.path()[length..]
    ));
    mapped.set_query(address.query());
    mapped.set_fragment(address.fragment());
    validate_address(&mapped)?;
    Ok(mapped)
}
pub fn guarded_map(
    address: &Url,
    environment: &Environment,
    settings: &DiscoverySettings,
) -> Result<Url> {
    let mapped = map_url(address, environment)?;
    // Reuse the established action/download/skipped-text policy. The logical
    // inventory owns the source section; exact mapped navigation owns destination
    // scope. A mapping does not authorize logout/actions implicitly.
    let mut policy = Project::draft("Mapped capture policy".into(), mapped.as_str())?;
    policy.discovery = settings.clone();
    policy.discovery.section = None;
    if let Some(reason) = crate::crawl::exclusion(&mapped, &policy, 0) {
        anyhow::bail!("Mapped page is excluded before navigation: {reason}");
    }
    Ok(mapped)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    Matching,
    RenderingCaution,
    MaterialMismatch,
}
pub fn compatibility(
    before: &[CaptureProfile],
    after: &[CaptureProfile],
    a: &RenderingEnvironment,
    b: &RenderingEnvironment,
) -> Compatibility {
    if before != after
        || a.browser != b.browser
        || a.worker_version != b.worker_version
        || a.profile_version != b.profile_version
    {
        Compatibility::MaterialMismatch
    } else if a.os != b.os
        || a.architecture != b.architecture
        || a.fonts_fingerprint != b.fonts_fingerprint
    {
        Compatibility::RenderingCaution
    } else {
        Compatibility::Matching
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_ordered_queries_hash_and_multi_origin_are_explicit() {
        let mut p = Project::draft("Site".into(), "https://prod.test/app").unwrap();
        let e = &mut p.environments[0];
        e.mappings = vec![
            OriginMapping {
                source: Url::parse("https://prod.test/app").unwrap(),
                destination: Url::parse("https://stage.test/preview").unwrap(),
            },
            OriginMapping {
                source: Url::parse("https://other.test/").unwrap(),
                destination: Url::parse("https://other-stage.test/root/").unwrap(),
            },
        ];
        assert_eq!(
            map_url(
                &Url::parse("https://prod.test/app/a%2Fb?b=2&a=1&a=0#/orders").unwrap(),
                e
            )
            .unwrap()
            .as_str(),
            "https://stage.test/preview/a%2Fb?b=2&a=1&a=0#/orders"
        );
        assert!(map_url(&Url::parse("https://prod.test/application").unwrap(), e).is_err());
        assert!(map_url(&Url::parse("https://unapproved.test/app").unwrap(), e).is_err());
        assert_eq!(
            map_url(&Url::parse("https://other.test/page/").unwrap(), e)
                .unwrap()
                .as_str(),
            "https://other-stage.test/root/page/"
        );
        e.mappings.push(e.mappings[0].clone());
        assert!(map_url(&p.sources[0], e).is_err());
    }
    #[test]
    fn sessions_do_not_serialize_and_profiles_have_material_integrity() {
        let mut p = Project::draft("Site".into(), "https://prod.test").unwrap();
        p.environments[0].session = Some(SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        });
        assert!(!serde_json::to_string(&p).unwrap().contains("session"));
        let r = RenderingEnvironment {
            os: "linux".into(),
            architecture: "x86_64".into(),
            browser: "1".into(),
            worker_version: "4".into(),
            profile_version: 1,
            fonts_fingerprint: None,
        };
        assert_eq!(
            compatibility(&p.profiles, &p.profiles, &r, &r),
            Compatibility::Matching
        );
        let mut other = r.clone();
        other.os = "macos".into();
        assert_eq!(
            compatibility(&p.profiles, &p.profiles, &r, &other),
            Compatibility::RenderingCaution
        );
        let mut profiles = p.profiles.clone();
        profiles[0].locale = "nb-NO".into();
        assert_eq!(
            compatibility(&p.profiles, &profiles, &r, &r),
            Compatibility::MaterialMismatch
        );
    }
}
