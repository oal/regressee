# Third-party notices — private candidate and historical fixture bench

Application sources are MPL-2.0 (`LICENSE`), except the explicitly attributed
Apache-2.0 adaptation in `crates/app/src/ui/text_input.rs` of GPUI's input example
(shared by the maintained product and separate fixture bench).
GPUI 0.2.2: Zed contributors, Apache-2.0; retained license in
`third-party/GPUI-LICENSE-APACHE`. GPUI dependencies have their own licenses;
the complete Cargo.lock inventory is not yet a release-license audit.

Node 22.20.0: MIT and embedded third-party notices (private pack's `node/LICENSE`).
Playwright/playwright-core 1.56.1: Microsoft, Apache-2.0, retained in the pack.
Chromium 141.0.7390.37 build 1194: BSD and bundled third-party licenses, retained
with the browser distribution. FFmpeg build 1011: retain supplied notices.
No external screenshot/photo corpus is used: fixtures and generated PNGs are
synthetic project-owned test material. Rasters are generated at test time, not
shipping brand assets. Do not publish the development runtime pack as a release.

## Maintained foundation additions

| Direct component | Exact pin | License / notice source |
| --- | --- | --- |
| rusqlite | 0.37.0 | MIT; upstream package LICENSE |
| libsqlite3-sys (bundled) | 0.35.0 | MIT wrapper; bundled SQLite 3.50.2 is public domain |
| directories | 6.0.0 | MIT OR Apache-2.0; upstream LICENSE files |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0; upstream LICENSE files |
| uuid | 1.27.0 | MIT OR Apache-2.0; upstream LICENSE files |
| url | 2.5.8 | MIT OR Apache-2.0; upstream LICENSE files |
| tempfile | 3.25.0 | MIT OR Apache-2.0; upstream LICENSE files |
| zeroize | 1.9.1 | MIT OR Apache-2.0; upstream LICENSE files |
| serde / serde_json | 1.0.228 / 1.0.145 | MIT OR Apache-2.0; upstream LICENSE files |
| sha2 / hex | 0.10.9 / 0.4.3 | MIT OR Apache-2.0; upstream LICENSE files |
| anyhow | 1.0.100 | MIT OR Apache-2.0; upstream LICENSE files |

Cargo.lock pins the full resolved graph, including optional GPUI test-support
dependencies. `worker/product` keeps a browser-free protocol npm lock; its actual
capture adapter uses Playwright supplied by a verified signed private runtime
pack, whose development-only component notices are listed above. No shipping
brand raster assets are introduced.
No font is bundled: platform-installed UI type is requested. Source distributions
must retain the Apache input attribution and license. Before binary distribution,
generate the complete target-specific dependency/license inventory and retain all
required texts, including bundled platform/graphics dependencies. This foundation
notice tracking is **not** a completed release-license audit.

## Product vertical-slice direct additions / reuse

Upstream Cargo package license declarations checked locally; required release
texts and the complete target-specific transitive inventory still need audit.

| Component | Exact pin | License |
| --- | --- | --- |
| reqwest | 0.12.24 | MIT OR Apache-2.0 |
| tokio | 1.50.0 | MIT |
| ed25519-dalek | 2.2.0 | BSD-3-Clause |
| zip | 2.4.2 | MIT |
| png | 0.17.16 | MIT OR Apache-2.0 |
| image | 0.25.10 | MIT OR Apache-2.0 |
| libc | 0.2.177 | MIT OR Apache-2.0 |

The actual product integration pack is signed with an ephemeral development key,
not the old bench private key and not release trust. It reuses verified historical
runtime bytes for **owned loopback fixtures only**; public capture remains refused
until a current supported trusted runtime is qualified. Do not publish this test
pack as a release.

## Phase 4 session protection

| Direct component | Exact pin | License / notice source |
| --- | --- | --- |
| aes-gcm | 0.10.3 | RustCrypto Developers; Apache-2.0 OR MIT; upstream LICENSE-APACHE / LICENSE-MIT |
| keyring | 3.6.3 | keyring-rs contributors; MIT OR Apache-2.0; upstream LICENSE-MIT / LICENSE-APACHE |

These pins and their license declarations were checked against the local Cargo
registry packages. AES-GCM, its transitive primitives and the target-specific
Secret Service/Keychain/Credential Manager dependencies must be included in the
full binary-release notice inventory. This is not a release-license audit or
cross-platform secure-store qualification. No real account, key or storage-state
fixture is bundled in application sources.

## Phase 7 private candidate

`windows-sys` **0.61.2**, Microsoft/windows-rs contributors, MIT OR Apache-2.0,
is now a direct target-conditional dependency for native Windows APIs. Its exact
cached package license texts are available; API compilation is not license approval.
No external dependency version was upgraded in Cargo.lock by this hardening;
only core/runtime direct edges to the already locked libc/windows-sys were added.

`scripts/release-candidate.py` resolves the **actual Linux x86-64 normal dependency
graph with app feature `ui`**, retains available local LICENSE/NOTICE/COPYING bytes
and their hashes beside the private app candidate, and explicitly marks all
entries unreviewed and missing declarations/texts blocked. This is not a complete
native/static-library SBOM, browser license audit or legal clearance. Native SQLite,
graphics/system dependencies, Node internals/browser codecs/FFmpeg and any final
packaging tools require artifact-specific source/notice review before release.
No runtime/browser/font or package tool is bundled into the private app tarball.

## Bundled timezone catalog

`crates/app/assets/timezones.txt` contains 436 Zone identifiers and UTC derived
from the locally installed IANA tzdata 2026c compact zic input, whose header
declares the data public domain. No timezone rules, host configuration or user
timezone are bundled. Source/derivation/hash are recorded in
`docs/research/timezone-select-20261010.md`; upstream authority:
<https://www.iana.org/time-zones>. This identifier list does not qualify browser
ICU compatibility or release-license review of other dependencies.
