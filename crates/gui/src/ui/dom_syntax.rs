// SPDX-License-Identifier: MPL-2.0
//! Display-only lexer for `DomSnapshot::format_source`, not HTML or executable code.
//! Ranges refer to the unchanged UTF-8 row. Unknown/over-budget rows stay plain.
use std::ops::Range;

pub(super) const MAX_SPANS: usize = 262_144;
const MAX_SPANS_PER_ROW: usize = 2048;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Kind {
    #[default]
    Plain,
    Metadata,
    Structure,
    Element,
    Attribute,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Span {
    pub bytes: Range<usize>,
    pub kind: Kind,
}

pub(super) fn highlight(text: &str, budget: usize) -> Vec<Span> {
    if budget == 0 {
        return Vec::new();
    }
    let mut lexer = Lexer {
        text: text.as_bytes(),
        at: 0,
        spans: Vec::new(),
        budget: budget.min(MAX_SPANS_PER_ROW),
    };
    if lexer.record().is_some() && lexer.at == text.len() {
        lexer.spans
    } else {
        Vec::new()
    }
}

struct Lexer<'a> {
    text: &'a [u8],
    at: usize,
    spans: Vec<Span>,
    budget: usize,
}

impl Lexer<'_> {
    fn mark(&mut self, start: usize, kind: Kind) -> Option<()> {
        if kind != Kind::Plain {
            if self.spans.len() == self.budget {
                return None;
            }
            self.spans.push(Span {
                bytes: start..self.at,
                kind,
            });
        }
        Some(())
    }

    fn literal(&mut self, text: &str, kind: Kind) -> Option<()> {
        let start = self.at;
        if !self.text[self.at..].starts_with(text.as_bytes()) {
            return None;
        }
        self.at += text.len();
        self.mark(start, kind)
    }

    fn space(&mut self) -> Option<()> {
        self.literal(" ", Kind::Plain)
    }

    // JSON string boundaries, including escaped quotes/backslashes and Unicode
    // escapes. Never decode, normalize, linkify or interpret the string contents.
    fn string(&mut self, kind: Kind) -> Option<()> {
        let start = self.at;
        if self.text.get(self.at) != Some(&b'"') {
            return None;
        }
        self.at += 1;
        loop {
            match *self.text.get(self.at)? {
                b'"' => {
                    self.at += 1;
                    return self.mark(start, kind);
                }
                b'\\' => {
                    self.at += 1;
                    match *self.text.get(self.at)? {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => self.at += 1,
                        b'u' => {
                            self.at += 1;
                            for _ in 0..4 {
                                if !self.text.get(self.at)?.is_ascii_hexdigit() {
                                    return None;
                                }
                                self.at += 1;
                            }
                        }
                        _ => return None,
                    }
                }
                0..=31 => return None,
                _ => self.at += 1,
            }
        }
    }

    fn record(&mut self) -> Option<()> {
        if self.text.first() != Some(&b'[') {
            return None;
        }
        self.at = 1;
        while self.text.get(self.at).is_some_and(u8::is_ascii_digit) {
            self.at += 1;
        }
        if self.at == 1 {
            return None;
        }
        self.literal("]", Kind::Plain)?;
        self.mark(0, Kind::Metadata)?;
        self.space()?;
        if self.text[self.at..].starts_with(b"element ") {
            self.literal("element", Kind::Structure)?;
            self.space()?;
            self.string(Kind::Metadata)?; // namespace
            self.space()?;
            self.string(Kind::Element)?;
            if self.text[self.at..].starts_with(b" prefix=") {
                self.space()?;
                self.literal("prefix", Kind::Attribute)?;
                self.literal("=", Kind::Metadata)?;
                self.string(Kind::Value)?;
            }
            while self.at < self.text.len() {
                self.space()?;
                self.string(Kind::Metadata)?; // attribute namespace
                self.literal(":", Kind::Metadata)?;
                self.string(Kind::Attribute)?;
                self.literal("=", Kind::Metadata)?;
                self.string(Kind::Value)?;
            }
        } else if self.text[self.at..].starts_with(b"text ") {
            self.literal("text", Kind::Structure)?;
            self.space()?;
            self.string(Kind::Plain)?; // page text is the reading focal point
        } else if self.text[self.at..].starts_with(b"!doctype ") {
            self.literal("!doctype", Kind::Structure)?;
            for kind in [Kind::Element, Kind::Value, Kind::Value] {
                self.space()?;
                self.string(kind)?;
            }
        } else {
            let marker = ["#document", "#shadow-root (open)", "#template-content"]
                .into_iter()
                .find(|marker| self.text[self.at..] == *marker.as_bytes())?;
            self.literal(marker, Kind::Structure)?;
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_source_goldens_cover_every_record_and_escaped_unicode() {
        use regressee_core::dom_snapshot::DomSnapshot;
        for canonical in [
            include_bytes!("../../../core/tests/fixtures/dom-v1.ndjson").as_slice(),
            include_bytes!("../../../core/tests/fixtures/dom-v1-html-names.ndjson").as_slice(),
        ] {
            let source = DomSnapshot::from_canonical(canonical)
                .unwrap()
                .format_source()
                .unwrap();
            for row in source.lines() {
                let spans = highlight(row, MAX_SPANS);
                assert!(!spans.is_empty(), "Unrecognized normalized row: {row}");
                for span in &spans {
                    assert!(row.is_char_boundary(span.bytes.start));
                    assert!(row.is_char_boundary(span.bytes.end));
                    assert!(!span.bytes.is_empty());
                }
                assert!(
                    spans
                        .windows(2)
                        .all(|pair| pair[0].bytes.end <= pair[1].bytes.start)
                );
                if row.contains("] text ") {
                    assert_eq!(spans.len(), 2, "Page text stays neutral");
                }
            }
        }
    }

    #[test]
    fn element_attribute_value_and_metadata_have_independent_roles() {
        let row = r#"[3] element "http://www.w3.org/2000/svg" "svg" prefix="s" "":"data-🦀"="say \"hi\" \\ \u00e9""#;
        let spans = highlight(row, MAX_SPANS);
        let role = |text: &str| {
            spans
                .iter()
                .find(|span| &row[span.bytes.clone()] == text)
                .unwrap()
                .kind
        };
        assert_eq!(role("[3]"), Kind::Metadata);
        assert_eq!(role("element"), Kind::Structure);
        assert_eq!(role(r#""http://www.w3.org/2000/svg""#), Kind::Metadata);
        assert_eq!(role(r#""svg""#), Kind::Element);
        assert_eq!(role(r#""data-🦀""#), Kind::Attribute);
        assert_eq!(spans.last().unwrap().kind, Kind::Value);
        assert_eq!(spans.last().unwrap().bytes.end, row.len());
    }

    #[test]
    fn unknown_malformed_and_budget_limited_rows_fall_back_without_rewriting() {
        for row in [
            "",
            "<main class=\"a\">",
            "[] #document",
            "[x] text \"a\"",
            "[2] text \"unfinished",
            "[2] text \"bad\\q\"",
            "[2] text \"bad\\u00xx\"",
            "[2] text \"raw\tcontrol\"",
            "[0] #document trailing",
            "[2] text \"a\" extra",
            "[1] element \"namespace\" \"div\" \"\":\"class\"=\"unterminated",
            "[1] element \"namespace\" \"div\" prefix=oops",
        ] {
            assert!(highlight(row, MAX_SPANS).is_empty(), "{row:?}");
        }
        let row = "[1] element \"namespace\" \"div\"";
        assert!(highlight(row, 0).is_empty());
        assert!(highlight(row, 2).is_empty());
        assert_eq!(highlight(row, 4).len(), 4);
        let huge = format!("{row}{}", " \"\":\"x\"=\"v\"".repeat(MAX_SPANS_PER_ROW));
        assert!(highlight(&huge, MAX_SPANS).is_empty());
    }
}
