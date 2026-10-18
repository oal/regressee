// SPDX-License-Identifier: MPL-2.0
//! Version 1 DOM contract. Artifacts are UTF-8 NDJSON, never executable HTML.
//!
//! Exact bytes: `["regressee-dom",1]\n`, then one compact JSON array per node,
//! each followed by LF (including the last). Records are flat preorder:
//! `["document",0]`, `["doctype",depth,name,publicId,systemId]`,
//! `["element",depth,namespaceURI,localName,[[attrNamespace,qualifiedName,value],…]]`,
//! `["text",depth,data]`, `["shadow",depth]`, `["template",depth]`.
//! An element with a non-null DOM prefix appends that nonempty string as field
//! six: `["element",depth,namespaceURI,localName,attributes,prefix]`. A null prefix
//! MUST omit the field (neither null nor "" is canonical). This distinguishes
//! prefix-null HTML `a:b` from XML/namespace-created elements and preserves actual
//! prefix changes without changing existing unprefixed records/golden bytes.
//! Missing namespaces are the empty string. Attributes sort by UTF-8 bytes of
//! (namespace, qualified name). Element names use DOM localName, not tagName.
//! Null-namespace attributes use their WHOLE name as localName, including colons
//! and HTML parser names such as @click, :class, 123, [binding] and a:b:c. Only a
//! non-null-namespace Attr's qualified name has XML prefix/local-name semantics.
//! Light children precede the optional open-shadow boundary, which is last.
//! HTML templates have exactly one template boundary instead of light children.
//! Boundaries participate in depth/node limits; their children have depth + 1.
//! Text nodes are not merged, trimmed or newline-normalized. Comments and other
//! unrepresented node kinds are omitted but count toward the visited-node budget.
//!
//! JSON escaping is JSON.stringify-compatible: escape quote/backslash, use
//! \\b/\\t/\\n/\\f/\\r and lowercase \\u00xx for other U+0000–001F; leave all
//! other Unicode (including U+2028/U+2029), slash and HTML characters literal.
//! JS must call toWellFormed() on every DOM string BEFORE bounds/sorting/encoding
//! (replace unpaired UTF-16 surrogates with U+FFFD). UTF-8 has no BOM. Rust rejects
//! noncanonical escaping, spacing, ordering, missing LF and unknown grammar.
//! Ingestion validates privacy/structure before callers publish to retained CAS.
use crate::domain::*;
use anyhow::{Result, bail, ensure};
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

pub const DOM_POLICY_VERSION: u32 = 1;
pub const DOM_CANONICAL_HEADER: &str = "[\"regressee-dom\",1]\n";
pub const MAX_DOM_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DOM_NODES: usize = 100_000;
pub const MAX_DOM_DEPTH: u16 = 512;
pub const MAX_DOM_ATTRIBUTES: usize = 256;
pub const MAX_DOM_STRING_BYTES: usize = 64 * 1024;
pub const MAX_DOM_ATTRIBUTE_BYTES: usize = 256 * 1024;
pub const MAX_DOM_RECORD_BYTES: usize = 1024 * 1024;
pub const DOM_STABILITY_SAMPLES: u8 = 3;
pub const DOM_QUIET_INTERVAL_MS: u64 = 100;
pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
pub const DOM_DIFF_ALGORITHM: &str = "regressee-dom-lines-v1";
pub const DOM_PRIVACY_NOTICE: &str = "DOM snapshots can contain private text, hidden content, attributes and URLs. Script and form-value exclusions are data minimization, not complete redaction.";
pub const DOM_COVERAGE_NOTICE: &str = "Includes the main document, template content and open shadow roots. Iframe documents and closed shadow roots are excluded; closed roots cannot reliably be counted.";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomSnapshotPolicy {
    pub version: u32,
}
impl Default for DomSnapshotPolicy {
    fn default() -> Self {
        Self {
            version: DOM_POLICY_VERSION,
        }
    }
}
impl DomSnapshotPolicy {
    pub fn validate(self) -> Result<()> {
        ensure!(
            self.version == DOM_POLICY_VERSION,
            "unsupported DOM snapshot policy"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomAttribute {
    pub namespace: String,
    pub name: String,
    pub value: String,
}
impl Serialize for DomAttribute {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        [&self.namespace, &self.name, &self.value].serialize(s)
    }
}
impl<'de> Deserialize<'de> for DomAttribute {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let [namespace, name, value] = <[String; 3]>::deserialize(d)?;
        Ok(Self {
            namespace,
            name,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomNode {
    Document,
    Doctype {
        depth: u16,
        name: String,
        public_id: String,
        system_id: String,
    },
    Element {
        depth: u16,
        namespace: String,
        name: String,
        attributes: Vec<DomAttribute>,
        prefix: Option<String>,
    },
    Text {
        depth: u16,
        text: String,
    },
    OpenShadow {
        depth: u16,
    },
    TemplateContent {
        depth: u16,
    },
}
impl DomNode {
    pub fn depth(&self) -> u16 {
        match self {
            Self::Document => 0,
            Self::Doctype { depth, .. }
            | Self::Element { depth, .. }
            | Self::Text { depth, .. }
            | Self::OpenShadow { depth }
            | Self::TemplateContent { depth } => *depth,
        }
    }
}
impl Serialize for DomNode {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(None)?;
        macro_rules! part {
            ($v:expr) => {
                seq.serialize_element($v)?
            };
        }
        match self {
            Self::Document => {
                part!("document");
                part!(&0u16);
            }
            Self::Doctype {
                depth,
                name,
                public_id,
                system_id,
            } => {
                part!("doctype");
                part!(depth);
                part!(name);
                part!(public_id);
                part!(system_id);
            }
            Self::Element {
                depth,
                namespace,
                name,
                attributes,
                prefix,
            } => {
                part!("element");
                part!(depth);
                part!(namespace);
                part!(name);
                part!(attributes);
                if let Some(prefix) = prefix {
                    part!(prefix);
                }
            }
            Self::Text { depth, text } => {
                part!("text");
                part!(depth);
                part!(text);
            }
            Self::OpenShadow { depth } => {
                part!("shadow");
                part!(depth);
            }
            Self::TemplateContent { depth } => {
                part!("template");
                part!(depth);
            }
        }
        seq.end()
    }
}
impl<'de> Deserialize<'de> for DomNode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct RecordVisitor;
        impl<'de> Visitor<'de> for RecordVisitor {
            type Value = DomNode;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a flat DOM record")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<DomNode, A::Error> {
                fn field<'de, T: Deserialize<'de>, A: SeqAccess<'de>>(
                    seq: &mut A,
                ) -> std::result::Result<T, A::Error> {
                    seq.next_element()?
                        .ok_or_else(|| de::Error::custom("missing DOM record field"))
                }
                let kind: String = field(&mut seq)?;
                let depth: u16 = field(&mut seq)?;
                let node = match kind.as_str() {
                    "document" if depth == 0 => DomNode::Document,
                    "doctype" => DomNode::Doctype {
                        depth,
                        name: field(&mut seq)?,
                        public_id: field(&mut seq)?,
                        system_id: field(&mut seq)?,
                    },
                    "element" => DomNode::Element {
                        depth,
                        namespace: field(&mut seq)?,
                        name: field(&mut seq)?,
                        attributes: field::<BoundedAttributes, _>(&mut seq)?.0,
                        prefix: seq.next_element::<String>()?,
                    },
                    "text" => DomNode::Text {
                        depth,
                        text: field(&mut seq)?,
                    },
                    "shadow" => DomNode::OpenShadow { depth },
                    "template" => DomNode::TemplateContent { depth },
                    _ => return Err(de::Error::custom("unknown DOM record kind or depth")),
                };
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom("extra DOM record fields"));
                }
                Ok(node)
            }
        }
        d.deserialize_seq(RecordVisitor)
    }
}

struct BoundedAttributes(Vec<DomAttribute>);
impl<'de> Deserialize<'de> for BoundedAttributes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct AttributesVisitor;
        impl<'de> Visitor<'de> for AttributesVisitor {
            type Value = BoundedAttributes;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("bounded flat DOM attributes")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut attributes = Vec::new();
                while let Some(attribute) = seq.next_element::<DomAttribute>()? {
                    if attributes.len() >= MAX_DOM_ATTRIBUTES {
                        return Err(de::Error::custom("DOM attribute limit exceeded"));
                    }
                    attributes.push(attribute);
                }
                Ok(BoundedAttributes(attributes))
            }
        }
        d.deserialize_seq(AttributesVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomSnapshot {
    pub nodes: Vec<DomNode>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomCoverage {
    pub iframe_hosts: u32,
    pub open_shadow_roots: u32,
    pub template_contents: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomSnapshotStats {
    pub node_count: u32,
    pub max_depth: u16,
    pub coverage: DomCoverage,
}

fn bounded_string(s: &str) -> Result<()> {
    ensure!(s.len() <= MAX_DOM_STRING_BYTES, "DOM string exceeds limit");
    Ok(())
}
fn dom_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 1024 && !s.contains('\0')
}
// Namespace-created names use XML NCName, unlike HTML tokenizer names. XML 1.0
// NameStartChar/NameChar ranges intentionally include supplementary characters.
fn xml_ncname(s: &str) -> bool {
    fn start(c: char) -> bool {
        matches!(c, 'A'..='Z' | '_' | 'a'..='z' | '\u{c0}'..='\u{d6}' | '\u{d8}'..='\u{f6}' | '\u{f8}'..='\u{2ff}' | '\u{370}'..='\u{37d}' | '\u{37f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
    }
    let mut chars = s.chars();
    dom_name(s) && chars.next().is_some_and(start)
        && chars.all(|c| start(c) || matches!(c, '-' | '.' | '0'..='9' | '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}'))
}
/// Recover the *actual* DOM Attr.localName from the canonical namespace/name
/// pair. Never strip colons from null-namespace HTML attributes. The grammar
/// retains qualified names, so real prefixes remain distinguishable as well.
pub fn dom_attribute_local_name<'a>(namespace: &str, name: &'a str) -> Result<&'a str> {
    ensure!(
        namespace.len() <= 4096 && dom_name(name),
        "invalid DOM attribute name/namespace"
    );
    if namespace.is_empty() {
        return Ok(name);
    }
    let local = if let Some((prefix, local)) = name.split_once(':') {
        ensure!(
            xml_ncname(prefix) && xml_ncname(local),
            "invalid namespaced DOM attribute name"
        );
        local
    } else {
        ensure!(xml_ncname(name), "invalid namespaced DOM attribute name");
        name
    };
    Ok(local)
}
/// Shared exclusion predicate. Both names are actual DOM *localName* strings,
/// not qualified names. Worker callers use Attr.localName; Rust ingestion uses
/// `dom_attribute_local_name(namespace, name)` to recover that exact identity.
pub fn excluded_attribute(element: &str, attribute: &str, value: &str) -> bool {
    let tag = element.to_ascii_lowercase();
    let name = attribute.to_ascii_lowercase();
    if name.starts_with("on") || name == "nonce" {
        return true;
    }
    if name == "value"
        && matches!(
            tag.as_str(),
            "input" | "select" | "option" | "button" | "textarea"
        )
    {
        return true;
    }
    if (tag == "input" && name == "checked")
        || (tag == "option" && name == "selected")
        || (tag == "iframe" && name == "srcdoc")
    {
        return true;
    }
    let url_attribute = matches!(
        name.as_str(),
        "href"
            | "src"
            | "action"
            | "formaction"
            | "data"
            | "poster"
            | "cite"
            | "background"
            | "longdesc"
            | "manifest"
            | "codebase"
            | "profile"
            | "usemap"
            | "icon"
            | "base"
    );
    url_attribute
        && value
            .chars()
            .filter(|c| !matches!(c, '\t' | '\r' | '\n'))
            .skip_while(|c| *c <= '\u{20}')
            .take(11)
            .collect::<String>()
            .eq_ignore_ascii_case("javascript:")
}

impl DomSnapshot {
    /// Never includes input bytes in errors (they can contain private data).
    pub fn from_canonical(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() <= MAX_DOM_BYTES,
            "DOM snapshot exceeds byte limit"
        );
        let text =
            std::str::from_utf8(bytes).map_err(|_| anyhow::anyhow!("DOM snapshot is not UTF-8"))?;
        ensure!(
            text.starts_with(DOM_CANONICAL_HEADER) && text.ends_with('\n'),
            "unsupported DOM format or missing final newline"
        );
        let body = &text[DOM_CANONICAL_HEADER.len()..];
        let mut nodes = Vec::new();
        for line in body.split_terminator('\n') {
            ensure!(
                nodes.len() < MAX_DOM_NODES && line.len() <= MAX_DOM_RECORD_BYTES,
                "DOM record/node limit exceeded"
            );
            let node = serde_json::from_str::<DomNode>(line)
                .map_err(|_| anyhow::anyhow!("invalid flat DOM record"))?;
            nodes.push(node);
        }
        let snapshot = Self { nodes };
        snapshot.validate()?;
        ensure!(
            snapshot.canonical_bytes()? == bytes,
            "noncanonical DOM encoding"
        );
        Ok(snapshot)
    }
    pub fn validate(&self) -> Result<DomSnapshotStats> {
        ensure!(
            !self.nodes.is_empty()
                && self.nodes.len() <= MAX_DOM_NODES
                && self.nodes[0] == DomNode::Document,
            "invalid DOM document/node count"
        );
        // The only stack is bounded by depth, not attacker-controlled JSON nesting.
        let mut ancestors: Vec<usize> = Vec::new();
        let mut child_counts = vec![0u32; self.nodes.len()];
        let mut boundary_seen = vec![false; self.nodes.len()];
        let mut coverage = DomCoverage::default();
        let mut max_depth = 0;
        let mut document_element = false;
        let mut doctype_seen = false;
        for (index, node) in self.nodes.iter().enumerate() {
            let depth = node.depth() as usize;
            ensure!(
                depth <= MAX_DOM_DEPTH as usize && (index == 0 || depth > 0),
                "invalid DOM depth/document placement"
            );
            max_depth = max_depth.max(depth as u16);
            if index > 0 {
                ensure!(depth <= ancestors.len(), "DOM preorder skips a depth");
                ancestors.truncate(depth);
                let parent = *ancestors
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("missing DOM parent"))?;
                ensure!(
                    !boundary_seen[parent],
                    "DOM child follows a terminal boundary"
                );
                child_counts[parent] += 1;
                match &self.nodes[parent] {
                    DomNode::Element {
                        name, namespace, ..
                    } => {
                        ensure!(
                            !name.eq_ignore_ascii_case("script")
                                && !name.eq_ignore_ascii_case("textarea")
                                && !name.eq_ignore_ascii_case("iframe"),
                            "excluded DOM element contents"
                        );
                        if namespace == HTML_NAMESPACE && name == "template" {
                            ensure!(
                                matches!(node, DomNode::TemplateContent { .. }),
                                "template content requires a boundary"
                            );
                        }
                    }
                    DomNode::Document
                    | DomNode::OpenShadow { .. }
                    | DomNode::TemplateContent { .. } => {}
                    _ => bail!("DOM leaf has children"),
                }
                match node {
                    DomNode::OpenShadow { .. } => {
                        ensure!(
                            matches!(&self.nodes[parent], DomNode::Element { namespace, name, .. } if !(namespace == HTML_NAMESPACE && name == "template")),
                            "invalid open-shadow boundary"
                        );
                        boundary_seen[parent] = true;
                        coverage.open_shadow_roots += 1;
                    }
                    DomNode::TemplateContent { .. } => {
                        ensure!(
                            matches!(&self.nodes[parent], DomNode::Element { namespace, name, .. } if namespace == HTML_NAMESPACE && name == "template")
                                && child_counts[parent] == 1,
                            "invalid template boundary"
                        );
                        boundary_seen[parent] = true;
                        coverage.template_contents += 1;
                    }
                    DomNode::Doctype { .. } => {
                        ensure!(
                            parent == 0 && !doctype_seen && !document_element,
                            "invalid doctype placement"
                        );
                        doctype_seen = true;
                    }
                    DomNode::Element { .. } if parent == 0 => {
                        ensure!(!document_element, "multiple document elements");
                        document_element = true;
                    }
                    DomNode::Text { .. } => ensure!(parent != 0, "text directly inside document"),
                    DomNode::Document => bail!("duplicate DOM document"),
                    _ => {}
                }
            }
            match node {
                DomNode::Element {
                    namespace,
                    name,
                    attributes,
                    prefix,
                    ..
                } => {
                    ensure!(
                        namespace.len() <= 4096 && dom_name(name),
                        "invalid DOM element name/namespace"
                    );
                    if let Some(prefix) = prefix {
                        ensure!(
                            !namespace.is_empty() && xml_ncname(prefix) && xml_ncname(name),
                            "invalid DOM element prefix/local name"
                        );
                    }
                    ensure!(
                        attributes.len() <= MAX_DOM_ATTRIBUTES,
                        "DOM attribute count exceeds limit"
                    );
                    let mut total = 0usize;
                    let mut previous: Option<(&str, &str)> = None;
                    let mut expanded_names = BTreeSet::new();
                    for attribute in attributes {
                        bounded_string(&attribute.value)?;
                        let local =
                            dom_attribute_local_name(&attribute.namespace, &attribute.name)?;
                        total += attribute.namespace.len()
                            + attribute.name.len()
                            + attribute.value.len();
                        let key = (attribute.namespace.as_str(), attribute.name.as_str());
                        ensure!(
                            previous.is_none_or(|old| old < key)
                                && expanded_names.insert((attribute.namespace.as_str(), local)),
                            "unsorted or duplicate DOM attributes"
                        );
                        previous = Some(key);
                        ensure!(
                            !excluded_attribute(name, local, &attribute.value),
                            "excluded DOM attribute"
                        );
                    }
                    ensure!(
                        total <= MAX_DOM_ATTRIBUTE_BYTES,
                        "DOM element attributes exceed byte limit"
                    );
                    if name.eq_ignore_ascii_case("iframe") {
                        coverage.iframe_hosts += 1;
                    }
                }
                DomNode::Text { text, .. } => bounded_string(text)?,
                DomNode::Doctype {
                    name,
                    public_id,
                    system_id,
                    ..
                } => {
                    // HTML's doctype tokenizer is not the XML Name production.
                    ensure!(
                        name.len() <= 1024 && !name.contains('\0'),
                        "invalid DOM doctype name"
                    );
                    bounded_string(public_id)?;
                    bounded_string(system_id)?;
                }
                _ => {}
            }
            ancestors.push(index);
        }
        for (node, children) in self.nodes.iter().zip(child_counts) {
            if matches!(node, DomNode::Element { namespace, name, .. } if namespace == HTML_NAMESPACE && name == "template")
            {
                ensure!(children == 1, "missing template boundary");
            }
        }
        Ok(DomSnapshotStats {
            node_count: self.nodes.len() as u32,
            max_depth,
            coverage,
        })
    }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut output = DOM_CANONICAL_HEADER.as_bytes().to_vec();
        for node in &self.nodes {
            let line = serde_json::to_vec(node)?;
            ensure!(
                line.len() <= MAX_DOM_RECORD_BYTES && output.len() + line.len() < MAX_DOM_BYTES,
                "DOM encoding exceeds byte limit"
            );
            output.extend(line);
            output.push(b'\n');
        }
        Ok(output)
    }
    /// Deterministic inert plaintext: one record per line, explicit depth rather
    /// than unbounded indentation. JSON quoting preserves text whitespace.
    pub fn format_source(&self) -> Result<String> {
        self.validate()?;
        let mut out = String::new();
        for node in &self.nodes {
            use std::fmt::Write;
            write!(out, "[{}] ", node.depth())?;
            match node {
                DomNode::Document => out.push_str("#document"),
                DomNode::Doctype {
                    name,
                    public_id,
                    system_id,
                    ..
                } => write!(
                    out,
                    "!doctype {} {} {}",
                    serde_json::to_string(name)?,
                    serde_json::to_string(public_id)?,
                    serde_json::to_string(system_id)?
                )?,
                DomNode::Element {
                    namespace,
                    name,
                    attributes,
                    prefix,
                    ..
                } => {
                    write!(
                        out,
                        "element {} {}",
                        serde_json::to_string(namespace)?,
                        serde_json::to_string(name)?
                    )?;
                    if let Some(prefix) = prefix {
                        write!(out, " prefix={}", serde_json::to_string(prefix)?)?;
                    }
                    for attr in attributes {
                        write!(
                            out,
                            " {}:{}={}",
                            serde_json::to_string(&attr.namespace)?,
                            serde_json::to_string(&attr.name)?,
                            serde_json::to_string(&attr.value)?
                        )?;
                    }
                }
                DomNode::Text { text, .. } => write!(out, "text {}", serde_json::to_string(text)?)?,
                DomNode::OpenShadow { .. } => out.push_str("#shadow-root (open)"),
                DomNode::TemplateContent { .. } => out.push_str("#template-content"),
            }
            out.push('\n');
            ensure!(
                out.len() <= MAX_DOM_SOURCE_BYTES,
                "formatted DOM source exceeds limit"
            );
        }
        Ok(out)
    }
}

pub fn dom_artifact_id(bytes: &[u8]) -> ArtifactId {
    hex::encode(Sha256::digest(bytes))
        .try_into()
        .expect("SHA-256 is a valid artifact ID")
}

pub const MAX_DOM_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DOM_DIFF_INPUT_LINES: usize = 200_000;
pub const MAX_DOM_DIFF_WORK: usize = 4_000_000;
pub const MAX_DOM_DIFF_OUTPUT_LINES: usize = 20_000;
pub const MAX_DOM_DIFF_OUTPUT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomDiffOptions {
    pub context_lines: usize,
    pub max_work: usize,
    pub max_output_lines: usize,
    pub max_output_bytes: usize,
}
impl Default for DomDiffOptions {
    fn default() -> Self {
        Self {
            context_lines: 3,
            max_work: 2_000_000,
            max_output_lines: MAX_DOM_DIFF_OUTPUT_LINES,
            max_output_bytes: MAX_DOM_DIFF_OUTPUT_BYTES,
        }
    }
}
impl DomDiffOptions {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.context_lines <= 100
                && self.max_work <= MAX_DOM_DIFF_WORK
                && self.max_output_lines <= MAX_DOM_DIFF_OUTPUT_LINES
                && self.max_output_bytes <= MAX_DOM_DIFF_OUTPUT_BYTES,
            "DOM diff options exceed limits"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomDiffRowKind {
    Context,
    Added,
    Removed,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomDiffRow {
    pub kind: DomDiffRowKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
    pub has_newline: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomDiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub rows: Vec<DomDiffRow>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomDiffFallback {
    WorkBudget,
    InputLineLimit,
    OutputLimit,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomTextDiff {
    /// Exact byte equality, independent of display limits or cancellation.
    pub changed: bool,
    pub old_line_count: u32,
    pub new_line_count: u32,
    /// Line counts for the emitted edit script; coarse replacement counts when
    /// work/input limits prevent a minimal edit script (never semantic counts).
    pub added_lines: u32,
    pub removed_lines: u32,
    pub hunks: Vec<DomDiffHunk>,
    pub fallback: Option<DomDiffFallback>,
    pub display_limited: bool,
}

#[derive(Clone, Copy)]
struct Edit<'a> {
    kind: DomDiffRowKind,
    line: &'a str,
}

/// Bounded LCS with common-prefix/suffix trimming. Equal-score ties delete first
/// (stable for repeated lines). Work exhaustion uses a coarse middle replacement.
/// Cancellation stops detail, but never changes the exact equality verdict.
pub fn diff_text(
    before: &str,
    after: &str,
    options: &DomDiffOptions,
    cancelled: impl Fn() -> bool,
) -> Result<DomTextDiff> {
    options.validate()?;
    ensure!(
        before.len() <= MAX_DOM_SOURCE_BYTES && after.len() <= MAX_DOM_SOURCE_BYTES,
        "DOM diff input exceeds byte limit"
    );
    let changed = before != after;
    let old_count = before.split_inclusive('\n').count();
    let new_count = after.split_inclusive('\n').count();
    let mut result = DomTextDiff {
        changed,
        old_line_count: old_count as u32,
        new_line_count: new_count as u32,
        added_lines: 0,
        removed_lines: 0,
        hunks: vec![],
        fallback: None,
        display_limited: false,
    };
    if !changed {
        return Ok(result);
    }
    let is_cancelled = cancelled();
    if is_cancelled || old_count > MAX_DOM_DIFF_INPUT_LINES || new_count > MAX_DOM_DIFF_INPUT_LINES
    {
        result.fallback = Some(if is_cancelled {
            DomDiffFallback::Cancelled
        } else {
            DomDiffFallback::InputLineLimit
        });
        result.added_lines = new_count as u32;
        result.removed_lines = old_count as u32;
        result.display_limited = true;
        return Ok(result);
    }
    let old: Vec<_> = before.split_inclusive('\n').collect();
    let new: Vec<_> = after.split_inclusive('\n').collect();
    // Intern exact lines once: the quadratic edit loop compares integer IDs,
    // not potentially 64 KiB strings. Map iteration order is never observed.
    let mut intern = std::collections::HashMap::<&str, u32>::new();
    let mut next_id = 0u32;
    let mut ids = Vec::with_capacity(old.len() + new.len());
    for (i, line) in old.iter().chain(new.iter()).enumerate() {
        if i % 1024 == 0 && cancelled() {
            result.fallback = Some(DomDiffFallback::Cancelled);
            result.added_lines = new_count as u32;
            result.removed_lines = old_count as u32;
            result.display_limited = true;
            return Ok(result);
        }
        let id = *intern.entry(line).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        ids.push(id);
    }
    let (old_ids, new_ids) = ids.split_at(old.len());
    let mut prefix = 0;
    while prefix < old.len().min(new.len()) && old_ids[prefix] == new_ids[prefix] {
        if prefix % 1024 == 0 && cancelled() {
            result.fallback = Some(DomDiffFallback::Cancelled);
            break;
        }
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old.len().min(new.len()) - prefix
        && old_ids[old.len() - 1 - suffix] == new_ids[new.len() - 1 - suffix]
    {
        if suffix % 1024 == 0 && cancelled() {
            result.fallback = Some(DomDiffFallback::Cancelled);
            break;
        }
        suffix += 1;
    }
    let a = &old[prefix..old.len() - suffix];
    let b = &new[prefix..new.len() - suffix];
    let a_ids = &old_ids[prefix..old.len() - suffix];
    let b_ids = &new_ids[prefix..new.len() - suffix];
    let cells = (a.len() + 1).checked_mul(b.len() + 1);
    let mut edits = Vec::with_capacity(old.len() + new.len());
    edits.extend(old[..prefix].iter().map(|line| Edit {
        kind: DomDiffRowKind::Context,
        line,
    }));
    let can_lcs = cells.is_some_and(|n| n <= options.max_work) && result.fallback.is_none();
    if can_lcs {
        let width = b.len() + 1;
        let mut table = vec![0u32; cells.unwrap_or(0)];
        'rows: for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                if (i * width + j) % 1024 == 0 && cancelled() {
                    result.fallback = Some(DomDiffFallback::Cancelled);
                    break 'rows;
                }
                table[i * width + j] = if a_ids[i] == b_ids[j] {
                    1 + table[(i + 1) * width + j + 1]
                } else {
                    table[(i + 1) * width + j].max(table[i * width + j + 1])
                };
            }
        }
        if result.fallback.is_none() {
            let (mut i, mut j) = (0, 0);
            while i < a.len() || j < b.len() {
                if i < a.len() && j < b.len() && a_ids[i] == b_ids[j] {
                    edits.push(Edit {
                        kind: DomDiffRowKind::Context,
                        line: a[i],
                    });
                    i += 1;
                    j += 1;
                } else if i < a.len()
                    && (j == b.len() || table[(i + 1) * width + j] >= table[i * width + j + 1])
                {
                    edits.push(Edit {
                        kind: DomDiffRowKind::Removed,
                        line: a[i],
                    });
                    i += 1;
                } else {
                    edits.push(Edit {
                        kind: DomDiffRowKind::Added,
                        line: b[j],
                    });
                    j += 1;
                }
            }
        }
    } else if result.fallback.is_none() {
        result.fallback = Some(DomDiffFallback::WorkBudget);
    }
    if result.fallback.is_some() {
        edits.extend(a.iter().map(|line| Edit {
            kind: DomDiffRowKind::Removed,
            line,
        }));
        edits.extend(b.iter().map(|line| Edit {
            kind: DomDiffRowKind::Added,
            line,
        }));
    }
    edits.extend(old[old.len() - suffix..].iter().map(|line| Edit {
        kind: DomDiffRowKind::Context,
        line,
    }));
    result.added_lines = edits
        .iter()
        .filter(|e| e.kind == DomDiffRowKind::Added)
        .count() as u32;
    result.removed_lines = edits
        .iter()
        .filter(|e| e.kind == DomDiffRowKind::Removed)
        .count() as u32;
    if result.fallback == Some(DomDiffFallback::Cancelled) {
        result.display_limited = true;
        return Ok(result);
    }
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (i, edit) in edits.iter().enumerate() {
        if edit.kind == DomDiffRowKind::Context {
            continue;
        }
        let start = i.saturating_sub(options.context_lines);
        let end = (i + options.context_lines + 1).min(edits.len());
        if let Some(previous) = ranges.last_mut().filter(|r| r.1 >= start) {
            previous.1 = previous.1.max(end);
        } else {
            ranges.push((start, end));
        }
    }
    let (mut old_line, mut new_line) = (1u32, 1u32);
    let (mut scan, mut output_rows, mut output_bytes) = (0, 0, 0usize);
    'hunks: for (start, end) in ranges {
        while scan < start {
            if edits[scan].kind != DomDiffRowKind::Added {
                old_line += 1;
            }
            if edits[scan].kind != DomDiffRowKind::Removed {
                new_line += 1;
            }
            scan += 1;
        }
        let mut hunk = DomDiffHunk {
            old_start: old_line,
            new_start: new_line,
            old_lines: 0,
            new_lines: 0,
            rows: vec![],
        };
        for edit in &edits[start..end] {
            if cancelled() {
                result.fallback = Some(DomDiffFallback::Cancelled);
                result.display_limited = true;
                break 'hunks;
            }
            let cost = edit.line.len() + 64;
            if output_rows >= options.max_output_lines
                || output_bytes.saturating_add(cost) > options.max_output_bytes
            {
                result.display_limited = true;
                if result.fallback.is_none() {
                    result.fallback = Some(DomDiffFallback::OutputLimit);
                }
                if !hunk.rows.is_empty() {
                    if hunk.old_lines == 0 {
                        hunk.old_start = hunk.old_start.saturating_sub(1);
                    }
                    if hunk.new_lines == 0 {
                        hunk.new_start = hunk.new_start.saturating_sub(1);
                    }
                    result.hunks.push(hunk);
                }
                break 'hunks;
            }
            let old_number = (edit.kind != DomDiffRowKind::Added).then_some(old_line);
            let new_number = (edit.kind != DomDiffRowKind::Removed).then_some(new_line);
            if old_number.is_some() {
                old_line += 1;
                hunk.old_lines += 1;
            }
            if new_number.is_some() {
                new_line += 1;
                hunk.new_lines += 1;
            }
            hunk.rows.push(DomDiffRow {
                kind: edit.kind,
                old_line: old_number,
                new_line: new_number,
                text: edit.line.strip_suffix('\n').unwrap_or(edit.line).to_owned(),
                has_newline: edit.line.ends_with('\n'),
            });
            output_rows += 1;
            output_bytes += cost;
            scan += 1;
        }
        if hunk.old_lines == 0 {
            hunk.old_start = hunk.old_start.saturating_sub(1);
        }
        if hunk.new_lines == 0 {
            hunk.new_start = hunk.new_start.saturating_sub(1);
        }
        result.hunks.push(hunk);
    }
    Ok(result)
}

/// HTML text-node encoding only; JSON/script/attribute contexts require their own
/// encoder. Reports must never place raw snapshots in executable contexts.
pub fn escape_dom_html_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomSourceIdentity {
    pub run: RunId,
    pub key: TargetKey,
    pub artifact: ArtifactId,
    pub policy: DomSnapshotPolicy,
    pub material_profile: ArtifactId,
    pub rendering_environment: ArtifactId,
    pub stable: bool,
}
impl DomSourceIdentity {
    pub fn validate(&self) -> Result<()> {
        self.policy.validate()?;
        ensure!(
            !self.run.0.is_nil() && !self.key.page.0.is_nil() && !self.key.profile.0.is_nil(),
            "invalid DOM analysis source"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomNotComparableReason {
    BaselineOmission,
    MissingBefore,
    MissingAfter,
    PolicyMismatch,
    MaterialProfileMismatch,
    RenderingEnvironmentMismatch,
    Unstable,
    InvalidEvidence,
    Skipped,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum DomComparisonOutcome {
    NotRequested,
    Unchanged,
    Changed,
    NotComparable { reason: DomNotComparableReason },
    CouldNotCapture { code: DomCaptureError },
}
impl DomComparisonOutcome {
    pub fn needs_review(&self) -> bool {
        matches!(self, Self::Changed)
    }
    pub fn comparable(&self) -> bool {
        matches!(self, Self::Changed | Self::Unchanged)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomAnalysisResult {
    pub key: TargetKey,
    pub before: Option<DomSourceIdentity>,
    pub after: Option<DomSourceIdentity>,
    pub outcome: DomComparisonOutcome,
    pub diff: Option<DomTextDiff>,
}
impl DomAnalysisResult {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.key.page.0.is_nil() && !self.key.profile.0.is_nil(),
            "invalid DOM analysis target"
        );
        for source in self.before.iter().chain(self.after.iter()) {
            source.validate()?;
            ensure!(
                source.key == self.key,
                "DOM analysis source/target mismatch"
            );
        }
        if self.outcome.comparable() {
            let (Some(before), Some(after)) = (&self.before, &self.after) else {
                bail!("DOM comparison requires both sources");
            };
            ensure!(
                before.stable
                    && after.stable
                    && before.policy == after.policy
                    && before.material_profile == after.material_profile
                    && before.rendering_environment == after.rendering_environment,
                "incompatible or unstable DOM comparison"
            );
            ensure!(
                (self.outcome == DomComparisonOutcome::Unchanged)
                    == (before.artifact == after.artifact),
                "DOM exact verdict/source mismatch"
            );
            if let Some(diff) = &self.diff {
                diff.validate()?;
                ensure!(
                    diff.changed == (self.outcome == DomComparisonOutcome::Changed),
                    "DOM diff/verdict mismatch"
                );
            }
        } else {
            ensure!(self.diff.is_none(), "noncomparable DOM evidence has a diff");
            ensure!(
                self.outcome != DomComparisonOutcome::NotRequested
                    || (self.before.is_none() && self.after.is_none()),
                "unrequested DOM result has sources"
            );
            if let DomComparisonOutcome::CouldNotCapture { code } = self.outcome {
                code.validate()?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomAnalysisRevision {
    pub version: u32,
    pub id: DomAnalysisId,
    pub baseline: BaselineId,
    pub run: RunId,
    pub algorithm: String,
    pub results: Vec<DomAnalysisResult>,
}
impl DomAnalysisRevision {
    pub fn source_artifacts(&self) -> BTreeSet<ArtifactId> {
        self.results
            .iter()
            .flat_map(|r| r.before.iter().chain(r.after.iter()))
            .map(|s| s.artifact.clone())
            .collect()
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1
                && self.algorithm == DOM_DIFF_ALGORITHM
                && !self.id.0.is_nil()
                && !self.baseline.0.is_nil()
                && !self.run.0.is_nil(),
            "unsupported DOM analysis identity/algorithm"
        );
        ensure!(
            !self.results.is_empty() && self.results.len() <= MAX_TARGETS,
            "invalid DOM analysis result count"
        );
        let mut keys = BTreeSet::new();
        for result in &self.results {
            result.validate()?;
            ensure!(
                keys.insert(&result.key) && result.after.as_ref().is_none_or(|s| s.run == self.run),
                "duplicate or foreign DOM analysis result"
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DomReviewDecision {
    pub analysis: DomAnalysisId,
    pub key: TargetKey,
    pub before: DomSourceIdentity,
    pub after: DomSourceIdentity,
    pub state: ReviewState,
    pub note: String,
    pub reviewed_at: i64,
}
impl DomReviewDecision {
    pub fn validate_for(&self, revision: &DomAnalysisRevision) -> Result<()> {
        revision.validate()?;
        ensure!(
            self.analysis == revision.id && validate_review_note(&self.note).is_ok(),
            "invalid DOM review identity/note"
        );
        let result = revision
            .results
            .iter()
            .find(|r| r.key == self.key)
            .ok_or_else(|| anyhow::anyhow!("foreign DOM review target"))?;
        ensure!(
            result.outcome.needs_review()
                && result.before.as_ref() == Some(&self.before)
                && result.after.as_ref() == Some(&self.after),
            "DOM review source mismatch or result does not need review"
        );
        Ok(())
    }
}

/// Storage must additionally verify project/draft revisions, CAS artifacts,
/// current-parent ancestry and user consent in one atomic transaction. Binding
/// both parents permits a fresh safe preview after a sibling-kind approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomApprovalSelection {
    pub analysis: DomAnalysisId,
    pub key: TargetKey,
    pub before: DomSourceIdentity,
    pub after: DomSourceIdentity,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomApprovalBinding {
    pub checked_baseline: BaselineId,
    pub current_parent: BaselineId,
    pub project_revision: u64,
    pub selections: Vec<DomApprovalSelection>,
}
impl DomApprovalBinding {
    pub fn validate_sources(
        &self,
        revision: &DomAnalysisRevision,
        current: &BaselineVersion,
    ) -> Result<()> {
        revision.validate()?;
        current.validate()?;
        ensure!(
            self.checked_baseline == revision.baseline
                && self.current_parent == current.id
                && !self.selections.is_empty()
                && self.selections.len() <= MAX_TARGETS,
            "stale DOM approval binding"
        );
        let mut keys = BTreeSet::new();
        for selected in &self.selections {
            ensure!(
                selected.analysis == revision.id && keys.insert(&selected.key),
                "duplicate or foreign DOM approval"
            );
            let result = revision
                .results
                .iter()
                .find(|r| r.key == selected.key)
                .ok_or_else(|| anyhow::anyhow!("unknown DOM approval target"))?;
            ensure!(
                result.outcome.comparable()
                    && result.before.as_ref() == Some(&selected.before)
                    && result.after.as_ref() == Some(&selected.after),
                "ineligible DOM approval sources"
            );
            let reference = current
                .dom_references
                .iter()
                .find(|r| r.key == selected.key)
                .ok_or_else(|| {
                    anyhow::anyhow!("cannot fill a baseline omission by selective approval")
                })?;
            ensure!(
                reference.run == selected.before.run
                    && reference.artifact == selected.before.artifact
                    && reference.policy == selected.before.policy
                    && current.capture_plan.dom_policy == Some(selected.after.policy),
                "selected DOM Before changed; compare again"
            );
        }
        Ok(())
    }
}

/// Both inputs must be complete verified artifacts, not viewer prefixes. Policy,
/// stability and runtime/material gates are independent of byte equality.
pub fn compare_dom_snapshots(
    before: DomSourceIdentity,
    before_bytes: &[u8],
    after: DomSourceIdentity,
    after_bytes: &[u8],
    options: &DomDiffOptions,
    cancelled: impl Fn() -> bool,
) -> Result<DomAnalysisResult> {
    before.validate()?;
    after.validate()?;
    ensure!(before.key == after.key, "DOM comparison target mismatch");
    ensure!(
        dom_artifact_id(before_bytes) == before.artifact
            && dom_artifact_id(after_bytes) == after.artifact,
        "DOM comparison artifact identity mismatch"
    );
    let before_snapshot = DomSnapshot::from_canonical(before_bytes)?;
    let after_snapshot = DomSnapshot::from_canonical(after_bytes)?;
    let reason = if before.policy != after.policy {
        Some(DomNotComparableReason::PolicyMismatch)
    } else if before.material_profile != after.material_profile {
        Some(DomNotComparableReason::MaterialProfileMismatch)
    } else if before.rendering_environment != after.rendering_environment {
        Some(DomNotComparableReason::RenderingEnvironmentMismatch)
    } else if !before.stable || !after.stable {
        Some(DomNotComparableReason::Unstable)
    } else {
        None
    };
    let changed = before_bytes != after_bytes;
    let (outcome, diff) = if let Some(reason) = reason {
        (DomComparisonOutcome::NotComparable { reason }, None)
    } else {
        (
            if changed {
                DomComparisonOutcome::Changed
            } else {
                DomComparisonOutcome::Unchanged
            },
            Some(diff_text(
                &before_snapshot.format_source()?,
                &after_snapshot.format_source()?,
                options,
                cancelled,
            )?),
        )
    };
    let result = DomAnalysisResult {
        key: before.key.clone(),
        before: Some(before),
        after: Some(after),
        outcome,
        diff,
    };
    result.validate()?;
    Ok(result)
}

impl DomTextDiff {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.old_line_count as usize <= MAX_DOM_SOURCE_BYTES
                && self.new_line_count as usize <= MAX_DOM_SOURCE_BYTES
                && self.added_lines <= self.new_line_count
                && self.removed_lines <= self.old_line_count,
            "invalid DOM diff line counts"
        );
        ensure!(
            self.changed
                || (self.added_lines == 0 && self.removed_lines == 0 && self.hunks.is_empty()),
            "unchanged DOM diff contains changes"
        );
        ensure!(
            !self.changed || !self.hunks.is_empty() || self.fallback.is_some(),
            "changed DOM diff is missing detail/limit cause"
        );
        ensure!(
            !self.changed || self.added_lines > 0 || self.removed_lines > 0,
            "changed DOM diff has no changed lines"
        );
        ensure!(
            !self.display_limited || self.fallback.is_some(),
            "DOM display limit has no explanation"
        );
        ensure!(
            self.hunks.len() <= MAX_DOM_DIFF_OUTPUT_LINES,
            "too many DOM diff hunks"
        );
        let (mut count, mut bytes) = (0usize, 0usize);
        for hunk in &self.hunks {
            ensure!(!hunk.rows.is_empty(), "empty DOM diff hunk");
            let old_rows = hunk.rows.iter().filter(|r| r.old_line.is_some()).count() as u32;
            let new_rows = hunk.rows.iter().filter(|r| r.new_line.is_some()).count() as u32;
            ensure!(
                hunk.old_start <= self.old_line_count
                    && hunk.new_start <= self.new_line_count
                    && (old_rows == 0 || hunk.old_start > 0)
                    && (new_rows == 0 || hunk.new_start > 0),
                "DOM diff hunk start out of range"
            );
            ensure!(
                hunk.old_lines == old_rows && hunk.new_lines == new_rows,
                "DOM diff hunk counts mismatch"
            );
            let mut old = if old_rows == 0 {
                hunk.old_start.saturating_add(1)
            } else {
                hunk.old_start
            };
            let mut new = if new_rows == 0 {
                hunk.new_start.saturating_add(1)
            } else {
                hunk.new_start
            };
            for row in &hunk.rows {
                ensure!(
                    !row.text.contains('\n'),
                    "DOM diff row contains unbounded lines"
                );
                ensure!(
                    row.old_line == (row.kind != DomDiffRowKind::Added).then_some(old)
                        && row.new_line == (row.kind != DomDiffRowKind::Removed).then_some(new),
                    "DOM diff line number mismatch"
                );
                if row.old_line.is_some() {
                    ensure!(
                        old > 0 && old <= self.old_line_count,
                        "DOM diff old line out of range"
                    );
                    old += 1;
                }
                if row.new_line.is_some() {
                    ensure!(
                        new > 0 && new <= self.new_line_count,
                        "DOM diff new line out of range"
                    );
                    new += 1;
                }
                count += 1;
                bytes = bytes.saturating_add(row.text.len() + usize::from(row.has_newline) + 64);
                ensure!(
                    count <= MAX_DOM_DIFF_OUTPUT_LINES && bytes <= MAX_DOM_DIFF_OUTPUT_BYTES,
                    "DOM diff output exceeds bounds"
                );
            }
        }
        ensure!(
            count <= MAX_DOM_DIFF_OUTPUT_LINES && bytes <= MAX_DOM_DIFF_OUTPUT_BYTES,
            "DOM diff output exceeds bounds"
        );
        Ok(())
    }
}
