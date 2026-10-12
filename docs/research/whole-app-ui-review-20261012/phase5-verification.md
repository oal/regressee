# Independent Phase 5 verification — FIX

**2026-10-12 · base `2f09a6b` · S1–S3 substantive exit gate not met.**
The implementation's BLOCKED handoff is correct. Two independently reproduced
functional defects and incomplete async/clip qualification prevent acceptance.
Do not commit/push Phase 5 as verified or start Phase 6.

## Verdict and finite repair assignment

Return this report to the **same sole implementation agent
`ses_ece1e2768ffeN3DjR0W3r25oFx`** for one finite functional follow-up covering
V5-1–V5-3 below, then independent re-verification. No redesign, broad renderer or
polish loop, backend capability reduction, or automatic login/baseline action.
The verifier has no session-messaging tool; the parent must deliver this assignment
to that existing session rather than spawn another implementation agent.

### V5-1 — Validation abandons guided stages (functional defect)

**Owners:** `crates/app/src/main.rs::save_settings` (2882–2894),
`form_recovery.rs`, guided mounted/focus paths.

Invalid login/representative/provider fields route to `SetupStage::Settings`;
invalid discovery fields do likewise. The pinned strip renders only for non-Settings
guided stages. Both existing `access-invalid` originals visibly lose the heading
and all three stages, although the affected input and error are readable.
The external `required_guided_access_validation_preserves_stage` test independently
fails with **actual Settings / required Access** after activating real Save.

**Required repair/tests:** preserve guided versus full-Settings context. Guided
Name/Address validation reveals Website; login fields 10–12 reveal Access;
discovery fields 0,3–6 reveal Pages and disclose advanced fields where needed.
Existing Settings must stay Settings. At both sizes test actual Save and correction
summary activation, expected stage, all three mounted stage controls, focus,
actual input border/error/help within the current scroll clip after reveal,
retained draft, and no save request/site visit on invalid input. Include cross-stage
invalid fields, not only an invalid address on Website. Correcting the field must
restore the appropriate clean/dirty next action without losing advanced options.

### V5-2 — Save-refresh hydration overwrites newer login edits (functional defect)

**Owners:** `main.rs::apply_update` preserved-save branch (703–711),
`refresh_visible` (1210–1215), `queue` (2681–2686), Environment reply (947–995),
`options_ux.rs::EnvironmentDraftSignature`, `setup_ux.rs`.

**Independent reproduction, actual local worker/storage:** load a stored fixture;
enter guided Access; set `environment=None`; change the project name and login URL
to a valid submitted value; activate Save; edit the login URL again before its
Project reply; apply that exact successful reply. The newer URL survives this
reply and no stage continuation occurs. `refresh_visible` then queues initial
EnvironmentStatus with a signature of the **newer draft**. Its successful reply
passes that signature test and hydrates the **older saved URL** over the newer edit.
The external `required_newer_login_draft_survives_save_then_absent_status_hydration`
test fails with `https://example.test/submitted-login` instead of
`https://example.test/newer-login` after the real follow-up read settles.

This is now a demonstrated defect, not merely the implementation's unqualified
race. A signature taken after newer edits cannot establish that saved-field
hydration is safe. Also, `environment_unsaved` returns false when status is absent;
login-only edits in that state need explicit dirty/continuation qualification.

**Required repair/tests:** separate status observation from permission to hydrate
the form. Preserve newer fields/retention/environment choices across save success,
refresh, and initial status reads; status must still become available without a
permanent loading loop. Track saved values/dirty identity sufficiently to keep
newer login-only drafts dirty even without status. Add the exact failing worker
probe above plus fields 11/12 and retention, edits during the subsequent read,
and an ordinary untouched initial hydration positive control. Verify failed save,
stale request, stage/screen switch, project/environment identity switch, and
newer/invalid/composing drafts: no unintended advance or discovery/login visit,
no overwrite of the selected project's draft, and no adoption of another
environment's readiness. Test both Continue and explicit Save & find pages.
Exact authorized success must still save and then discover; ordinary Settings
save must never visit. Preserve baseline identity and saved evidence throughout.

Screen/identity races are **qualification holes**, not additional reproduced
defects: the signature includes them but no dedicated test currently proves the
preserved-save branch's subsequent record/read handling safe.

### V5-3 — Window containment is mislabeled control visibility (qualification gap)

**Owners:** `ui_review_matrix.rs` Phase 5 assertions (900–968),
`phase5_setup_tests.rs`, setup form/focus reveal.

At 820×600 `pages-sizes`, Desktop and Tablet are readable, the selected Mobile
dimensions are in the plan, but the Mobile checkbox is clipped by the footer.
The renderer nevertheless emits `fully_visible:true` for all three checkboxes
because it compares them with the whole window, not `screen_scroll.bounds()`.
This does not prove clickability/readability inside the painted scroll owner.
The advanced field test also reuses a clip captured before changing stages instead
of fetching the current owner's bounds for each asserted frame.

**Required repair/tests:** use actual current clipping owners for scrolling
controls, painted field borders, attached errors/help and mapping rows. Pinned
heading/footer controls correctly use their own owner/window, not the form clip.
At both sizes exercise real mounted/enabled Tab/focus-reveal and dispatch paths
for Desktop/Tablet/Mobile and every advanced setup capability, including custom
add/replace/remove, mapping/locale/timezone/theme and session maintenance.
Assert Mobile is reachable, revealed wholly within the form clip, and updates the
current plan without an implicit save/visit. Do not claim initial visibility if
scrolling is necessary. **Scrolling itself is not a defect requiring a redesign:**
sizes lead the form and selected dimensions are already discoverable. Preserve
blocked evidence/receipts; label the old window-only assertions truthfully and
record repaired functional geometry separately. No new renderer cycle is needed
to execute these controller/layout requirements.

## Substantive gate accounting

| Gate | Independent result |
|---|---|
| Public/private choice and immediate safe action in first minimum viewport | PASS for inspected clean/public, dirty/private, saved/private, browser-open and ready synthetic states; private status/actions remain attached in footer |
| Three stages, current/completed/unavailable state, persistent wayfinding | PASS on ordinary guided states; **FIX on validation**, including discovery validation by source |
| Clean Continue without redundant save; dirty authoritative continuation | Existing tests PASS; exact/newer-name/stale-request/save-failure cases covered; **FIX for hydration and uncovered identity races** |
| Explicit save/visit boundaries | PASS in existing scoped tests: Save alone does not open login; Save & find pages names and authorizes discovery; no combined login continuation introduced; real login not qualified |
| Sizes first and compact current capture plan | PASS for selected dimensions, extent, image policy and emulation wording at both sizes; **reachability/clip qualification incomplete**, not an initial-all-controls pass |
| Mapping before scope prose | PASS by source and both-size contact sheets; narrow original clearly leads with source/capture pair and attached Close; remaining rows intentionally scroll |
| Included-target estimate and discovery limitation/next action near Capture | PASS: inspected `68 × 2 = 136`, real fixture inclusion count and Page limit reached cause; rediscovery advice adjacent; options retain disk/time estimates as estimates |
| Advanced capability preservation and ordinary explicit Settings | Source inventory retained: sources/consent/bounds/skips/robots/speed/depth/parallelism/retries/link/tracking/fragments, profiles/custom/extent/images/limitations, separate environments/access/retention/login/providers/mapping/probe/locale/timezone/theme/maintenance. **Full traversal qualification incomplete**; no capability removal found |
| Validation identifies input while guided flow remains | Input reveal PASS for sampled originals/existing tests; guided-flow requirement **FIX** |

Bitmap/photo and session-only copy preserve no-plaintext fallback, screenshot
privacy, unsupported replacement cases and frozen-capture limitations. Worker
operations remain off the UI thread. Core/storage/schema/capture/approval owners
are unchanged by this Phase 5 diff; package regression is not universal safety proof.

## Independent execution and integrity

Read the **full plan, blueprint, implementation report and receipt**, actual tracked
diff (including DESIGN/checkpoint changes), both new source files, and affected
save/status/form/options/focus/capability owners. No product source edit.

- Locked offline core/storage/app all-features packages: **303 passed, 0 failed,
  40 ignored**, exit 0. Log: `/var/home/olav/Data/opencode/tmp/opencode/phase5-verifier-packages.log`.
- Separate serial `phase5_setup_tests`: **4 passed, 0 failed, 0 ignored**, exit 0.
  Log: `/var/home/olav/Data/opencode/tmp/opencode/phase5-verifier-targeted.log`.
- External required-behavior probes: **0 passed, 2 failed**, exit 101, with the two
  relevant assertions above. Final log:
  `/var/home/olav/Data/opencode/tmp/opencode/phase5-verifier-probe-final2.log`.
- Probe workspace: `/var/home/olav/Data/opencode/tmp/opencode/phase5-verifier-probe-4yc78ll3`;
  `crates/app/src/independent_phase5_probe.rs` SHA-256
  `90b21416b69db173c9e5a4aa3aaf4ead88a0a07c665f0fbef01d902e2e2e3e25`.
  App sources byte-match current source except the added test-module declaration;
  original product files were never edited. Uses isolated temporary fixture data
  and existing local dependencies/assets; no real browser/runtime visit.
- Initial external probe attempt lacked the fixture-root symlink; a concurrent
  shared Cargo target later selected an original binary and ran **zero** probes.
  Neither is counted as qualification. The corrected, forced-recompiled final run
  executes both named tests and reproduces both required-behavior failures.
- `python3 scripts/verify-phase5-receipt.py`: exit 0, **12 immutable archives,
  182 original-root PNGs, 38 confirmation pairs, 65 source bindings**. All 65
  source hashes independently rechecked after probes, unchanged.
- `cargo fmt --all -- --check` and `git diff --check`: exit 0.

Commands used standard handoff environment `LIBRARY_PATH=.runtime/link-libs`,
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`, locked/offline Cargo.
External probes use `--manifest-path <probe-root>/Cargo.toml`, absolute library
path and existing `CARGO_TARGET_DIR`, filtering `independent_phase5_probe` with
`--test-threads=1`. Existing targeted filter is `phase5_setup_tests`.

## Bounded existing-pixel read ledger and limits

One read-only verifier batch, **eight confirmation contact sheets and 14 originals**;
no new renderer, recapture, gallery rebuild or polish cycle. Root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012/confirmation`.
Sheets: `contact-sheets/all-{1280,820}-{01,02,03,04}.png`.

Originals read (`all-` prefix, `.png` suffix):

- `820x600-access-{public,login,saved-private,browser-open,ready,invalid}`;
- `1280x800-access-invalid`;
- `820x600-pages-sizes`, `1280x800-pages-sizes`;
- `820x600-pages-advanced-tail`, `1280x800-project-settings-bottom`;
- `820x600-mapping-preview`, `820x600-discovery`, `820x600-discovery-options`.

Not every one of the 38 originals was independently read at full resolution.
Narrow status-loading is pixel-identical to public and does not establish private
status-loading feedback; the fixture deliberately removes public environment
status. Full advanced tails and identity/hydration routes need the named functional
tests, not inference from label counts or image nonidentity.

These are synthetic **Linux GPUI/Blade offscreen pixels**, not native-window/input
acceptance. Real OS pointer/keyboard, IME/clipboard, screen readers, scaling/high
contrast/motion, Windows/macOS, runtime install/authentication, PM usability,
large-site throughput and release readiness remain **not tested**. Existing
trusted-browser-distribution and vendor warning limits remain. No score increase,
commit/push, product repair, new delegate, or Phase 6 was performed.
