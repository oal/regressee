// SPDX-License-Identifier: MPL-2.0
use super::*;
use core::prelude::v1::test;

fn context(n: u32, text: impl Into<Arc<str>>) -> DomDiffRow {
    DomDiffRow {
        kind: DomDiffRowKind::Context,
        old_line: Some(n),
        new_line: Some(n),
        text: text.into(),
    }
}

fn unchanged(count: u32) -> DomDiffDocument {
    DomDiffDocument::new(
        DomDiffMode::Unchanged,
        (1..=count)
            .map(|n| context(n, format!("line {n} café 🦀")))
            .collect(),
        vec![],
        None,
    )
    .unwrap()
}

fn changed() -> DomDiffDocument {
    DomDiffDocument::new(
        DomDiffMode::Changed,
        vec![
            context(1, "<main>"),
            DomDiffRow {
                kind: DomDiffRowKind::Removed,
                old_line: Some(2),
                new_line: None,
                text: "  <p>Before</p>".into(),
            },
            DomDiffRow {
                kind: DomDiffRowKind::Added,
                old_line: None,
                new_line: Some(2),
                text: "  <p>After 🦀</p>".into(),
            },
            context(3, "</main>"),
        ],
        vec![DomDiffHunk { rows: 0..4 }],
        None,
    )
    .unwrap()
}

#[test]
fn model_accepts_independent_sides_and_counts_source_lines_not_segments() {
    let document = changed();
    assert_eq!((document.added, document.removed), (1, 1));
    assert_eq!(document.hunks().len(), 1);
    for mode in [DomDiffMode::BeforeOnly, DomDiffMode::AfterOnly] {
        let row = DomDiffRow {
            kind: DomDiffRowKind::Context,
            old_line: (mode == DomDiffMode::BeforeOnly).then_some(1),
            new_line: (mode == DomDiffMode::AfterOnly).then_some(1),
            text: "long source ".repeat(40).into(),
        };
        let capture = DomDiffDocument::new(mode, vec![row], vec![], None).unwrap();
        assert!(capture.segments.len() > 1);
        assert_eq!((capture.added, capture.removed), (0, 0));
        assert!(capture.blocks.iter().all(|b| !matches!(b, Block::Fold(_))));
    }
}

#[test]
fn model_rejects_inconsistent_kind_numbers_hunks_and_newlines() {
    let invalid = [
        DomDiffRow {
            kind: DomDiffRowKind::Added,
            old_line: Some(1),
            new_line: Some(1),
            text: "bad".into(),
        },
        context(0, "zero"),
        context(1, "two\nlines"),
        context(1, "two\rlines"),
    ];
    for row in invalid {
        assert!(
            DomDiffDocument::new(
                DomDiffMode::Changed,
                vec![row],
                vec![DomDiffHunk { rows: 0..1 }],
                None
            )
            .is_err()
        );
    }
    assert!(
        DomDiffDocument::new(
            DomDiffMode::Unchanged,
            vec![context(1, "a"), context(1, "b")],
            vec![],
            None
        )
        .is_err()
    );
    let rows = changed().rows;
    for hunks in [
        vec![],
        vec![DomDiffHunk { rows: 0..1 }],
        vec![DomDiffHunk { rows: 0..5 }],
        vec![DomDiffHunk { rows: 0..3 }, DomDiffHunk { rows: 2..4 }],
    ] {
        assert!(DomDiffDocument::new(DomDiffMode::Changed, rows.clone(), hunks, None).is_err());
    }
    assert!(DomDiffDocument::new(DomDiffMode::Unchanged, rows, vec![], None).is_err());
}

#[test]
fn model_empty_and_coarse_changed_results_do_not_invent_equality() {
    let empty = DomDiffDocument::new(DomDiffMode::Unchanged, vec![], vec![], None).unwrap();
    assert!(empty.segments.is_empty());
    assert!(DomDiffDocument::new(DomDiffMode::Changed, vec![], vec![], None).is_err());
    let coarse = DomDiffDocument::new(
        DomDiffMode::Changed,
        vec![],
        vec![],
        Some("Detailed diff exceeded its work budget; complete originals remain available.".into()),
    )
    .unwrap();
    assert_eq!(coarse.mode(), DomDiffMode::Changed);
    assert!(coarse.notice.is_some());
}

#[test]
fn model_context_folding_retains_all_source_and_three_lines_on_each_edge() {
    let document = unchanged(100);
    assert_eq!(document.rows().len(), 100);
    assert!(
        matches!(&document.blocks[..], [Block::Lines(a), Block::Fold(b), Block::Lines(c)] if *a == (0..3) && *b == (3..97) && *c == (97..100))
    );
    assert_eq!(document.row_segments.len(), 100);
    assert_eq!(document.segments.len(), 100);
}

#[test]
fn model_wrapping_is_utf8_exact_bounded_and_preserves_empty_lines() {
    for text in [
        String::new(),
        "🦀é界a".repeat(200),
        "x".repeat(80),
        "x".repeat(81),
    ] {
        let ranges = model::wrap_ranges(&text);
        assert!(!ranges.is_empty());
        assert_eq!(
            ranges.iter().map(|r| &text[r.clone()]).collect::<String>(),
            text
        );
        assert!(
            ranges
                .iter()
                .all(|r| text[r.clone()].chars().count() <= WRAP_COLUMNS)
        );
    }
    let text = "🦀".repeat(80);
    let document = DomDiffDocument::new(
        DomDiffMode::Unchanged,
        vec![context(1, text.clone())],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(
        document.segment_at(SourcePosition {
            row: 0,
            byte: text.len()
        }),
        0
    );
    let high = DomDiffDocument::new(
        DomDiffMode::Unchanged,
        vec![context(u32::MAX, "last original line")],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(high.number_digits, 10);
}

#[test]
fn model_literal_search_finds_hidden_adversarial_source_and_cancels() {
    let mut rows = (1..=100).map(|n| context(n, "plain")).collect::<Vec<_>>();
    rows[49].text = "</script><script>alert('&');</script> 🦀".into();
    let document = DomDiffDocument::new(DomDiffMode::Unchanged, rows, vec![], None).unwrap();
    let results = document.search("<script>", &AtomicBool::new(false));
    assert_eq!(
        results.matches,
        vec![model::SearchMatch {
            row: 49,
            bytes: 9..17
        }]
    );
    assert!(
        document
            .search("<SCRIPT>", &AtomicBool::new(false))
            .matches
            .is_empty()
    );
    assert!(
        document
            .search("plain", &AtomicBool::new(true))
            .matches
            .is_empty()
    );
    assert!(
        document
            .search("", &AtomicBool::new(false))
            .matches
            .is_empty()
    );
}

#[test]
fn model_search_limit_is_explicit_and_not_a_comparison_verdict() {
    let document = DomDiffDocument::new(
        DomDiffMode::Unchanged,
        vec![context(1, "a".repeat(model::MAX_MATCHES + 1))],
        vec![],
        None,
    )
    .unwrap();
    let results = document.search("a", &AtomicBool::new(false));
    assert_eq!(results.matches.len(), model::MAX_MATCHES);
    assert!(results.limited);
    assert_eq!(document.mode(), DomDiffMode::Unchanged);
}

#[test]
fn model_copy_uses_original_unicode_source_without_numbers_prefixes_or_wraps() {
    let document = unchanged(100);
    let selection = Selection {
        anchor: SourcePosition {
            row: 98,
            byte: "line 99 ".len(),
        },
        caret: SourcePosition {
            row: 0,
            byte: "line 1 ".len(),
        },
    };
    let copied = document.selected_text(selection);
    assert!(copied.starts_with("café 🦀\nline 2"));
    assert!(copied.ends_with("line 99 "));
    assert_eq!(copied.lines().count(), 99);
    let document = changed();
    let copied = document.selected_text(Selection {
        anchor: SourcePosition { row: 1, byte: 0 },
        caret: SourcePosition {
            row: 2,
            byte: document.rows[2].text.len(),
        },
    });
    assert_eq!(copied, "  <p>Before</p>\n  <p>After 🦀</p>");
}

#[test]
fn model_viewer_budget_rejects_without_truncation() {
    let result = DomDiffDocument::new(
        DomDiffMode::Unchanged,
        vec![context(1, "x".repeat(16 * 1024 * 1024 + 1))],
        vec![],
        None,
    );
    let error = result.unwrap_err();
    assert!(error.message.contains("display limit"));
    assert!(error.recovery.contains("has not been altered"));
    assert!(
        DomDiffDocument::new(
            DomDiffMode::Unchanged,
            vec![],
            vec![],
            Some("x".repeat(8193).into())
        )
        .is_err()
    );
}

#[test]
fn word_selection_is_scalar_safe_at_empty_and_unicode_edges() {
    let text = "one 🦀 café";
    assert_eq!(&text[word_range(text, text.len())], "café");
    assert_eq!(&text[word_range(text, 4)], "🦀");
    assert_eq!(word_range("", 0), 0..0);
}

fn syntax_document() -> DomDiffDocument {
    DomDiffDocument::new(
        DomDiffMode::AfterOnly,
        vec![DomDiffRow {
            kind: DomDiffRowKind::Context,
            old_line: None,
            new_line: Some(1),
            text: format!(
                r#"[1] element "namespace" "div" "":"class"="{}" "":"title"="café 🦀""#,
                "a🦀".repeat(100)
            )
            .into(),
        }],
        vec![],
        None,
    )
    .unwrap()
}

#[test]
fn model_syntax_survives_wrapping_with_exact_bytes_and_neutral_page_text() {
    let document = syntax_document();
    let row = &document.rows[0].text;
    let mut reconstructed = String::new();
    for (_, bytes) in &document.segments {
        let spans = document.styled_segment(
            0,
            bytes.clone(),
            Selection::default(),
            &SearchResults::default(),
            None,
        );
        assert_eq!(
            spans.iter().map(|span| span.bytes.len()).sum::<usize>(),
            bytes.len()
        );
        for span in spans {
            reconstructed.push_str(&row[span.bytes.clone()]);
            if bytes.start > row.find("class").unwrap() && bytes.end < row.find("title").unwrap() {
                assert_eq!(span.kind, syntax::Kind::Value);
            }
        }
    }
    assert_eq!(reconstructed, row.as_ref());
    assert_eq!(
        document.selected_text(Selection {
            anchor: SourcePosition::default(),
            caret: SourcePosition {
                row: 0,
                byte: row.len()
            },
        }),
        row.as_ref()
    );
    let plain = unchanged(1);
    assert!(plain.syntax[0].is_empty());
}

#[test]
fn model_search_selection_precedence_clips_to_original_wrapped_utf8_ranges() {
    let document = syntax_document();
    let results = document.search("a🦀", &AtomicBool::new(false));
    assert_eq!(results.matches.len(), 100);
    let found = &results.matches[0];
    let search_selection = Selection {
        anchor: SourcePosition {
            row: 0,
            byte: found.bytes.start,
        },
        caret: SourcePosition {
            row: 0,
            byte: found.bytes.end,
        },
    };
    let spans = document.styled_segment(
        0,
        0..document.rows[0].text.len(),
        search_selection,
        &results,
        Some(0),
    );
    assert_eq!(
        spans
            .iter()
            .find(|span| span.bytes.contains(&found.bytes.start))
            .unwrap()
            .emphasis,
        Emphasis::CurrentMatch
    );
    assert_eq!(
        spans
            .iter()
            .find(|span| span.bytes.contains(&results.matches[1].bytes.start))
            .unwrap()
            .emphasis,
        Emphasis::Match
    );
    let selection = Selection {
        anchor: search_selection.anchor,
        caret: SourcePosition {
            row: 0,
            byte: results.matches[3].bytes.end,
        },
    };
    for (_, bytes) in &document.segments {
        for span in document.styled_segment(0, bytes.clone(), selection, &results, Some(0)) {
            if (selection.anchor.byte..selection.caret.byte).contains(&span.bytes.start) {
                assert_eq!(span.emphasis, Emphasis::Selection);
            }
            assert!(document.rows[0].text.is_char_boundary(span.bytes.start));
            assert!(document.rows[0].text.is_char_boundary(span.bytes.end));
        }
    }
}

#[test]
fn model_highlighting_budget_is_independent_of_evidence_and_verdict() {
    let rows = (1..=70_000)
        .map(|n| context(n, r#"[1] element "namespace" "div""#))
        .collect();
    let document = DomDiffDocument::new(DomDiffMode::Unchanged, rows, vec![], None).unwrap();
    assert_eq!(document.rows.len(), 70_000);
    assert_eq!(
        document.syntax.iter().map(Vec::len).sum::<usize>(),
        syntax::MAX_SPANS
    );
    assert!(document.syntax.last().unwrap().is_empty());
    assert_eq!(document.mode(), DomDiffMode::Unchanged);
    assert_eq!(
        document.rows.last().unwrap().text.as_ref(),
        r#"[1] element "namespace" "div""#
    );
}

#[cfg(feature = "ui-tests")]
mod native {
    use super::*;
    use gpui::{Modifiers, TestAppContext, VisualTestContext};

    struct Host {
        diff: Entity<DomDiff>,
        width: f32,
        height: f32,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(self.width))
                .h(px(self.height))
                .child(self.diff.clone())
        }
    }
    fn setup(
        cx: &mut TestAppContext,
        document: DomDiffDocument,
        width: f32,
    ) -> (Entity<DomDiff>, &mut VisualTestContext) {
        cx.update(super::super::super::text_input::bind);
        let (host, cx) = cx.add_window_view(|_, cx| {
            let diff = cx.new(DomDiff::new);
            diff.update(cx, |diff, cx| {
                let token = diff.begin_load("fixture", cx);
                assert!(diff.finish_load(&token, Ok(Arc::new(document)), cx));
            });
            Host {
                diff,
                width,
                height: 520.,
            }
        });
        let diff = cx.update(|_, cx| host.read(cx).diff.clone());
        cx.run_until_parked();
        (diff, cx)
    }

    #[gpui::test]
    fn syntax_search_selection_and_theme_changes_preserve_native_text_geometry(
        cx: &mut TestAppContext,
    ) {
        let (diff, cx) = setup(cx, syntax_document(), 420.);
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                let original = this.document().unwrap().clone();
                let text = original.rows[0].text.clone();
                let bounds = this.viewport.unwrap();
                let old_dark = is_dark();
                let mut first_geometry = None;
                for dark in [false, true] {
                    set_dark(dark);
                    this.results = original.search("a🦀", &AtomicBool::new(false));
                    this.visit_match(0);
                    let frame = this.prepare_frame(bounds, window, cx);
                    let mut geometry = Vec::new();
                    for frame in frame {
                        if let FrameContent::Source { segment, line, .. } = frame.content {
                            let bytes = &original.segments[segment].1;
                            assert_eq!(line.text.as_ref(), &text[bytes.clone()]);
                            let baseline = window.text_system().shape_line(
                                line.text.clone(),
                                px(SOURCE_SIZE),
                                &[TextRun {
                                    len: bytes.len(),
                                    font: Font {
                                        family: MONO.into(),
                                        ..window.text_style().font()
                                    },
                                    color: rgb(palette().ink).into(),
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                }],
                                None,
                            );
                            assert_eq!(line.width, baseline.width);
                            for (byte, _) in line.text.char_indices() {
                                assert_eq!(line.x_for_index(byte), baseline.x_for_index(byte));
                            }
                            geometry.push((segment, line.width));
                        }
                    }
                    if let Some(first) = &first_geometry {
                        assert_eq!(&geometry, first);
                    } else {
                        first_geometry = Some(geometry);
                    }
                    assert_eq!(this.document().unwrap().rows[0].text, text);
                    assert_eq!(
                        source_colors(syntax::Kind::Attribute, Emphasis::CurrentMatch),
                        (palette().on_accent, Some(palette().accent_hover))
                    );
                    assert_eq!(
                        source_colors(syntax::Kind::Value, Emphasis::Selection),
                        (palette().ink, Some(dom_palette().selection))
                    );
                }
                set_dark(old_dark);
                window.focus(&this.source_focus);
            });
        });
        cx.simulate_keystrokes("ctrl-a ctrl-c");
        cx.update(|_, cx| {
            let this = diff.read(cx);
            assert_eq!(
                cx.read_from_clipboard().unwrap().text().unwrap(),
                this.document().unwrap().rows[0].text.as_ref()
            );
        });
    }

    #[gpui::test]
    fn short_windows_keep_source_rows_and_reveal_focused_tools(cx: &mut TestAppContext) {
        cx.update(super::super::super::text_input::bind);
        let (host, cx) = cx.add_window_view(|_, cx| {
            let diff = cx.new(DomDiff::new);
            diff.update(cx, |diff, cx| {
                let token = diff.begin_load("short-window", cx);
                assert!(diff.finish_load(&token, Ok(Arc::new(changed())), cx));
            });
            Host {
                diff,
                width: 552.,
                height: 184.,
            }
        });
        cx.simulate_resize(size(px(820.), px(600.)));
        let diff = cx.update(|_, cx| host.read(cx).diff.clone());
        cx.run_until_parked();
        diff.read_with(cx, |source, _| {
            assert!(source.viewport.unwrap().size.height >= px(64.));
            assert!(!source.painted.is_empty());
        });
        cx.update(|window, cx| {
            diff.update(cx, |source, _| {
                window.focus(&source.handle(Control::NextHunk));
            })
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let tools = cx.debug_bounds("dom-diff-tools").unwrap();
        let next = cx.debug_bounds("action-dom-diff-next-change").unwrap();
        assert_eq!(next.size.height, px(CONTROL_HEIGHT));
        assert!(
            next.top() >= tools.top() && next.bottom() <= tools.bottom(),
            "{next:?} outside {tools:?}"
        );
        diff.read_with(cx, |source, _| {
            assert!(source.viewport.unwrap().size.height >= px(64.))
        });
    }

    #[gpui::test]
    fn stale_loads_clear_old_source_selection_and_do_not_publish(cx: &mut TestAppContext) {
        let (diff, cx) = setup(cx, changed(), 800.);
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                window.focus(&this.source_focus);
                this.selection = Selection {
                    anchor: SourcePosition::default(),
                    caret: SourcePosition { row: 1, byte: 3 },
                };
                let old = this.begin_load("old", cx);
                let current = this.begin_load("current", cx);
                assert!(this.painted.is_empty());
                assert!(this.selection.is_empty());
                assert!(!this.finish_load(&old, Ok(Arc::new(changed())), cx));
                assert!(matches!(this.state, LoadState::Loading));
                assert!(this.finish_load(
                    &current,
                    Err(DomDiffError::new("Missing source", "Reload from History.")),
                    cx
                ));
                assert!(!this.finish_load(&current, Ok(Arc::new(changed())), cx));
            });
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        // GPUI's diagnostic bounds map retains prior selectors across frames.
        // Current source/model/paint state, not that map, establishes absence.
        diff.read_with(cx, |source, _| {
            assert!(source.document().is_none());
            assert!(source.painted.is_empty());
            assert!(source.items.is_empty());
        });
        assert!(cx.debug_bounds("dom-diff-state").is_some());
    }

    #[gpui::test]
    fn long_error_body_has_a_keyboard_reachable_bounded_scroll_owner(cx: &mut TestAppContext) {
        let (diff, cx) = setup(cx, changed(), 420.);
        cx.update(|_, cx| {
            diff.update(cx, |this, cx| {
                let token = this.begin_load("failed read", cx);
                this.finish_load(
                    &token,
                    Err(DomDiffError::new(
                        "The recorded source could not be read. ".repeat(100),
                        "Reload the result from History.",
                    )),
                    cx,
                );
                assert_eq!(this.focus_handles(cx).len(), 1);
            })
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            diff.update(cx, |this, _| {
                assert!(this.state_scroll.max_offset().height > px(0.));
                window.focus(&this.state_focus);
            })
        });
        cx.simulate_keystrokes("end");
        cx.update(|_, cx| {
            let this = diff.read(cx);
            assert_eq!(
                this.state_scroll.offset().y,
                -this.state_scroll.max_offset().height
            );
        });
    }

    #[gpui::test]
    fn viewport_rows_are_bounded_and_context_expands_without_changing_evidence(
        cx: &mut TestAppContext,
    ) {
        for width in [800., 420.] {
            let (diff, cx) = setup(cx, unchanged(20_000), width);
            cx.update(|_, cx| {
                diff.update(cx, |this, cx| {
                    assert!(this.items.len() < 10);
                    assert!(this.painted.len() <= 8);
                    this.toggle_context(cx);
                    assert_eq!(this.items.len(), 20_000);
                    assert_eq!(this.document().unwrap().rows.len(), 20_000);
                })
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let this = diff.read(cx);
                let bounds = this.viewport.unwrap();
                assert!(
                    this.painted.len()
                        <= (f32::from(bounds.size.height) / SOURCE_HEIGHT).ceil() as usize + 2
                );
                assert!(this.overflow(bounds).height > px(0.));
                assert_eq!(
                    this.overflow(bounds).width > px(0.),
                    this.content_width > bounds.size.width
                );
            });
        }
    }

    #[gpui::test]
    fn multiple_hunks_wrap_in_both_directions_and_long_source_stays_reachable(
        cx: &mut TestAppContext,
    ) {
        let mut rows = changed().rows;
        rows.extend([
            DomDiffRow {
                kind: DomDiffRowKind::Removed,
                old_line: Some(4),
                new_line: None,
                text: "old tail".into(),
            },
            DomDiffRow {
                kind: DomDiffRowKind::Added,
                old_line: None,
                new_line: Some(4),
                text: "🦀".repeat(500).into(),
            },
            context(5, "end"),
        ]);
        let document = DomDiffDocument::new(
            DomDiffMode::Changed,
            rows,
            vec![DomDiffHunk { rows: 0..4 }, DomDiffHunk { rows: 4..7 }],
            None,
        )
        .unwrap();
        let (diff, cx) = setup(cx, document, 420.);
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                window.focus(&this.handle(Control::NextHunk));
                this.navigate_hunk(true, cx);
                assert_eq!(this.current_hunk, Some(0));
                this.navigate_hunk(true, cx);
                assert_eq!(this.current_hunk, Some(1));
                this.navigate_hunk(true, cx);
                assert_eq!(this.current_hunk, Some(0));
                this.navigate_hunk(false, cx);
                assert_eq!(this.current_hunk, Some(1));
                assert!(this.handle(Control::NextHunk).is_focused(window));
                let end = SourcePosition {
                    row: 5,
                    byte: "🦀".repeat(500).len(),
                };
                this.move_caret(end, true, cx);
            })
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let this = diff.read(cx);
            let document = this.document().unwrap();
            let last = document.segment_at(this.selection.caret);
            assert!(this.painted.iter().any(|line| line.segment == last));
            assert!(this.overflow(this.viewport.unwrap()).width > px(0.));
            assert_eq!(document.rows[5].text.chars().count(), 500);
            assert!(document.row_segments[5].len() > 1);
        });
    }

    #[gpui::test]
    fn readonly_keyboard_selection_copy_and_live_gate(cx: &mut TestAppContext) {
        let (diff, cx) = setup(cx, changed(), 800.);
        let gate = interaction::InputGate::default();
        cx.update(|window, cx| {
            gate.bind(window, cx);
            window.focus(&diff.read(cx).source_focus);
        });
        cx.simulate_keystrokes("ctrl-a ctrl-c");
        cx.update(|_, cx| {
            assert_eq!(
                cx.read_from_clipboard().unwrap().text().unwrap(),
                "<main>\n  <p>Before</p>\n  <p>After 🦀</p>\n</main>"
            );
        });
        cx.simulate_keystrokes("backspace delete ctrl-x ctrl-v");
        cx.simulate_input("must not edit source");
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                assert_eq!(this.document().unwrap().rows[0].text.as_ref(), "<main>");
                let before = this.selection;
                gate.set_allowed(false);
                this.move_caret_if_allowed_for_test(window, cx);
                this.copy_selection(window, cx);
                assert_eq!(this.selection, before);
            })
        });
    }

    #[gpui::test]
    fn search_reveals_folded_matches_and_navigation_does_not_steal_focus(cx: &mut TestAppContext) {
        let (diff, cx) = setup(cx, unchanged(100), 800.);
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                this.focus_search(window, cx);
                this.query = "line 50 ".into();
                this.start_search(cx);
            })
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let this = diff.read(cx);
            assert!(!this.searching);
            assert_eq!(this.results.matches.len(), 1);
            assert_eq!(this.selection.caret.row, 49);
            assert!(!this.expanded.is_empty());
            assert!(this.search_focus.is_focused(window));
        });
        let (diff, cx) = setup(cx, changed(), 800.);
        cx.update(|window, cx| {
            diff.update(cx, |this, cx| {
                window.focus(&this.handle(Control::NextHunk));
                this.activate(Control::NextHunk, window, cx);
                assert_eq!(this.current_hunk, Some(0));
                assert!(this.handle(Control::NextHunk).is_focused(window));
                this.set_enabled(false, cx);
                assert!(this.focus_handles(cx).is_empty());
            })
        });
    }

    #[gpui::test]
    fn pointer_selection_copies_unicode_and_releases_outside(cx: &mut TestAppContext) {
        let (diff, cx) = setup(cx, changed(), 800.);
        let (start, end) = cx.update(|_, cx| {
            let this = diff.read(cx);
            let line = this
                .painted
                .iter()
                .find(|line| this.document().unwrap().segments[line.segment].0 == 2)
                .unwrap();
            let byte = "  <p>After ".len();
            (
                point(
                    line.bounds.left() + line.line.x_for_index(byte),
                    line.bounds.center().y,
                ),
                point(
                    line.bounds.left() + line.line.x_for_index(byte + "🦀".len()),
                    line.bounds.center().y,
                ),
            )
        });
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(point(px(-20.), end.y), MouseButton::Left, Modifiers::none());
        cx.simulate_keystrokes("ctrl-c");
        cx.update(|_, cx| {
            assert!(!diff.read(cx).selecting);
            assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), "🦀");
        });
    }

    impl DomDiff {
        fn move_caret_if_allowed_for_test(&mut self, window: &Window, cx: &mut Context<Self>) {
            self.source_key(
                &KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("shift-right").unwrap(),
                    is_held: false,
                },
                window,
                cx,
            );
        }
    }
}
