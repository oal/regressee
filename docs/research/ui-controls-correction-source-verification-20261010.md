# Controls/timezone correction — independent source verification

**2026-10-10 — PASS WITH SOURCE-ONLY SCOPE.** No material P1/P2 fault found in
the reviewed corrections. This reviewer did not implement controls, timezone,
main integration or core focus changes. Earlier runtime work is not independently
reverified here.

One bounded read-only pass: current `ui/{controls,timezone_select,performance,
text_input}.rs`, relevant `main.rs` integration/regression regions,
`crates/core/src/shell.rs`, bundled timezone catalog and its provenance note.
Only this new report was written. No source edits, builds, formatters, tests,
native/worker/browser launches, subagents, user stores/authentication or screenshots
were accessed. No parent test totals are adopted as this reviewer's results.

## Freeze binding

Read-only SHA-256 comparisons before and after the pass matched **all 73 source/
manifest entries** in
`/path/to/private-tests/opencode/regressee-ui-source-freeze-20261010.json`.
The freeze file itself retained SHA-256
`e35447500b1d74cc9b6d441423d5536110f610171120b4b07fa6450e7a4516bc`.
The recorded binary hash was read, not independently checked or executed here.
This report is outside that frozen source inventory.

## Source contracts checked

| Concern | Finding and exact source |
| --- | --- |
| Authoritative timezone | Main creates/observes the selector (`main.rs:236,291`); Save writes `.selected` to every draft profile (`1808–1812`). Neither search text nor `crawl_fields[16]` supplies the persisted value. |
| Load/new/discard | Stored/imported project replies replace the draft then call `set_crawl_fields` (`391–420`); new-project reset calls it (`1733–1746`); discard restores stored profiles then calls it (`5014–5016`). That helper feeds stored profile timezone through `set_value` (`1943–1946`). Ordinary navigation does not replace the draft. |
| Dirty/environment save | Dirty comparison reads selector selection (`4879–4924`), separate from the search filter. General unsaved includes environment edits (`4926–4945`). Save incorporates timezone into the same project sent with local login/environment settings via `Work::SaveEnvironment` (`1787–1919`), rather than dispatching a second profile save. Existing baseline references are preserved in the copied project. Login/probe guard rejects dirty saved-environment inputs (`2239–2243`). This is UI dispatch verification, not a new storage transaction test. |
| Current focus handle | `handle(CrawlField(16))` returns the selector's current trigger/search handle before the ordinary crawl-field branch (`1555–1584`). Mounted and enabled checks agree on saved-project Setup fields 10–16 (`3423,3493–3507`); render separately applies that eligibility to the selector (`5120–5125`). The old input entity at index 16 is unmounted and is not the logical Tab handle. |
| Visible environment order | Render orders access choices → fields 10–12 → login retention choices → fields 13–14 → mapping actions 37/38 → locale 15 → timezone 16 → light/dark → login actions (`2571–2598,2620–2628`). Core candidates reproduce that order (`shell.rs:242–254`). Main filters candidates using the same mounted/enabled predicates for pointer activation, keyboard dispatch and Tab (`1597–1624,3165–3194`). |
| Popup focus/dispatch | Selector uses separate near trigger/search key contexts; main binds it after TextInput, before Shell (`5383–5390`). Enter chooses rather than Shell Activate; Space opens only the trigger, leaving search text editing intact. Tab belongs to Shell and uses the dynamic selector handle. Escape closes an open popup and restores the trigger; closed Escape propagates. Outside/blur dismissal does not steal a newly focused target (`timezone_select.rs:194–220,249–289,343–359,442–449,517–542`). Disabled state closes and removes stops without editing selection. |
| Catalog/compatibility | Catalog is embedded with `include_str!`, not runtime host-zoneinfo/network lookup (`timezone_select.rs:23–35`). Read-only count found 437 distinct asset identifiers. Provenance note records installed tzdata 2026c, source hash, public-domain declaration and Zone-only extraction plus UTC; this reviewer did not reauthenticate upstream tzdata. Unknown/legacy values are preserved exactly by `new`/`set_value`, marked “Not in list” with bounded sanitized presentation; only catalog rows can newly commit (`46–67,165–174,282–289,613–620`). Browser ICU acceptance remains explicitly separate from catalog membership. |
| Compact action geometry | Shared actions set intrinsic/non-navigation alignment and bounded width, with a flex label wrapper rather than a collapsing block child (`controls.rs:95–127`). Navigation retains column stretch; selection rows and editable fields retain full-width geometry (`129–136`). No interaction state replaces text properties: focus adds border/shadow; hover/active change background only (`56–90`). TextInput focus similarly changes only border/background (`text_input.rs:561–595`). |
| Feedback/operation controls | Dismiss is a child of the feedback surface, beside a 160px-bounded scrollable message area (`controls.rs:328–375`; `main.rs:5167–5177`). Busy progress and guarded pause/cancel remain outside that message scroller (`5138–5165`); content has its own flex/min-height scroll region and Setup footer (`5224–5265`). Feedback availability is shared between candidates, mounting and activation (`shell.rs:174`; `main.rs:3205,3390`). |
| Data-free timing | Opt-in requires `REGRESSEE_UI_DIAGNOSTICS=1`. Marker fields contain static screen/phase labels and elapsed durations only (`performance.rs:9–34`; `main.rs:1702–1704,5284–5303`): no project names, URLs, login data or diagnostic payloads. CPU construction/layout/paint labels are not GPU presentation measurements; the module explicitly states that limitation. |
| Inspection CLI boundary | Linux inspection module/start are compile-feature gated (`main.rs:33–34,5451–5454`). `--inspect-ui` refuses a featureless/non-Linux build and absent explicit data/inspection roots before Jobs/store startup (`5345–5357`). With inspection requested, the data-path branch cannot silently select ordinary discovered user data (`5376–5380`). Actual private-controller ownership, directory validation and PNG/native results belong to the separate native inspection agent, not this source pass. |

## Regression-source assessment and limits

Reviewed authored regressions cover actual action/label bounds at 1120×800 and
820×600, attached bounded feedback, backend setup capability, mounted focus-handle
membership, saved locale/timezone/environment dispatch, selector competing Shell
Enter bindings, search/no-match behavior, pointer notifications, popup clipping,
Tab/blur/Escape and unknown/disabled values. These are meaningful source assertions,
**not tests executed by this reviewer**. Main's mounted-handle regression checks
membership/uniqueness, not complete visible order; the changed environment order
was additionally compared directly above and has explicit core adjacency assertions
(`shell.rs:351–375`).

No rendered readability, native pointer/keyboard/IME/accessibility, usable minimum-
window dropdown or human workflow qualification follows from these findings.
The user-reported roughly one-second Projects → Setup delay remains an active
native investigation: this pass supplies no measured diagnosis, fix claim or
speculative performance change. Parent/native reports and actual frame evidence
must remain distinct from this source-only acceptance.
