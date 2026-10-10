# Phase 4 — authentication, environments, and capture reliability

## Outcome

The app handles the intended development/release workflow for logged-in admin
panels and staging/internal sites, with honest stability/comparability signals.

Prerequisite: Phase 3 gate passed. Read `PLAN.md`, Sections 6–10 and 13.

Local implementation delivered; **independent verification and the human completion
gate remain pending**. Checked items below mean implemented/tested in the stated
local scope, not cross-platform/native/release qualification. See `04-status.md`.

## Work

- [x] Implement environment-specific **Open browser / I'm ready** login and
  consent setup, representative-page confirmation, and secure state persistence.
- [x] Support the locally qualified cookie/localStorage/IndexedDB/sessionStorage cases.
  Document device-bound/enterprise/session replay limitations.
- [x] Protect encryption keys with OS credential stores; make missing secure
  storage actionable and offer ephemeral sessions instead of plaintext fallback.
- [x] Reuse isolated contexts with saved state, detect expiry/access failures,
  and offer re-login without diffing the login screen as application content.
- [x] Keep production and staging sessions separate; redact all diagnostics.
- [x] Implement same-environment and explicit origin/base-prefix mappings;
  preview/probe mapped baseline URLs before launching the full check.
- [x] Implement navigation-scope checks for external redirects and suspicious
  action/logout links. Recommend read-only accounts and explain GET-side effects.
- [x] Complete deterministic locale/timezone/theme/motion/hover handling,
  bounded font/image readiness, lazy-load scrolling, and stability checks.
- [x] Handle infinite/tall pages with locally qualified limits and explicit warnings;
  document virtualized/nested-scroll coverage rather than silently cropping.
- [x] Persist rendering-environment fingerprints and enforce material profile
  compatibility; add OS/environment caution mode.
- [ ] Qualify VPN/system-network access and configured proxy/trusted-CA paths
  on supported environments. Never disable TLS verification as a repair step.
- [x] Test stale sessions, unusual login redirects, delayed fonts/images,
  unstable counters, rate limits, and production/staging path differences.

## Deliverables

- Interactive auth/session management and environment mapping flows.
- Stabilization metadata and compatibility UI.
- Cross-platform secret-storage and auth integration tests.
- Published capture/auth/network limitations in user-facing help.

Local signed auth tests, explicit test-provider encryption/restart tests and
user-facing help are delivered. Real Linux secure-store persistence is **not
qualified**: the bounded availability check reports unavailable on this host.
Windows/macOS and native/human interaction tests are **NOT RUN**. Proxy download
fixture passes; Chromium proxy/custom-CA and VPN/enterprise qualification stay open.

## Completion gate

A non-developer logs in normally, captures authenticated fixture pages, and
compares matching paths on another environment with a separately established
session. Expiry produces **Log in again**, not ordinary visual diffs. Staged
mapping previews are correct for prefix/query/hash fixtures. Secrets remain
encrypted/redacted and never enter project export data. Unstable and incomplete
captures are visible. Imported cross-OS baselines cannot masquerade as fully
comparable release checks.

**Gate NOT PASSED:** no non-developer/native session was exercised here. Simulated
GPUI and headless signed fixtures do not substitute for that gate.

## Next

Phase 5 delivers the full noise-management, post-capture analysis, image-mode,
and baseline-approval experience.
