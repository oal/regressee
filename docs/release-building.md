# Private build and release preparation

Run serially in a dedicated checkout/image with pre-provisioned prerequisites:

```sh
cargo fmt --all -- --check
cargo build --locked --workspace --all-features --all-targets
cargo test --locked --workspace --all-features -- --test-threads=1
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
cargo build --release --locked -p regressee-app --features ui --bin regressee
```

Linux: build in Ubuntu 24.04 userspace, inspect every ELF using `readelf -d` and
`readelf --version-info`. Reject GLIBC above 2.39; never patch in a host glibc.
AppImage/FUSE/extract-and-run remains a packaging recipe, not a delivered installer.
Run the exact packaged default-sandbox browser on default AppArmor/userns policy
without admin/sandbox bypasses. The current development binary fails the ABI floor.

Windows: x86_64-pc-windows-msvc on a pre-provisioned SDK. Native conditional tests
must exercise Job assignment, nested Chromium jobs, owner crash, cancellation,
locked-file publication and quota/ACL/reparse refusal as a standard user. Inspect
`dumpbin /dependents`; retain verified upstream Authenticode bytes and have the
owner sign the app/per-user installer. Server CI is not Windows 11 desktop evidence.

macOS: separate aarch64-apple-darwin and x86_64-apple-darwin builds, deployment
target 14.0, inspect `lipo -info`, `otool -L/-l`. Runtime manifest v3 authenticates
relative contained `.app`/framework link targets and exact file modes. Links are
created last; escapes/cycles/dangling/case/leaf collisions refuse. Never flatten
frameworks or strip quarantine. Product supervision is still a blocking code gap.
Owner/native `codesign --verify --deep --strict`, Gatekeeper, Developer ID,
notarization/stapling and approved JIT entitlements are separate required work;
no automatic signing or vendor submission is authorized.

## Source and artifact binding

`scripts/source_release.py` is the integrated independently checked Linux-only
source helper. It does not execute sources, emits deterministic source archive,
inventory/hashes and explicitly blocked release/legal gates in a fresh private
directory **outside** the source tree. New help/build documents are reviewed
allowlist entries. Unknown/unreviewed dependencies are never auto-approved.

```sh
python3 -B scripts/test_source_release.py --report <new-private-report>
python3 -B scripts/source_release.py --help
python3 scripts/release-candidate.py --help
python3 scripts/runtime-pack.py --help
```

Freeze the source inventory before building. Compare the selected-source inventory
again after build and package; refuse binding if any selected file changes. Bind
binary, source archive/inventory, license inventory and runtime identities/hashes
in the candidate manifest. A repeat incremental build hash is not a clean-machine
bit-reproducibility proof. Timestamped signing/notary bytes are distinct from
reproducible unsigned payloads. Private candidate manifests do not confer trust.

The default official-component catalogs and bundled source payload are generated
by `python3 scripts/official-runtime.py`, then reviewed and committed as application
inputs. Exact provenance, licenses, archive pins and complete inventory must bind
to the packaged app; unreviewed regeneration is not a release gate. This does not
publish or sign an app. See decision 0011 and the official-runtime asset report.

For an optional signed runtime v3 override, `runtime-pack.py` prepares an unsigned manifest from a reviewed
staging tree without following links. The owner signs its exact bytes with an
external Ed25519 key (64-byte detached signature) and supplies only the public key
to the configured product distribution. No private production key is committed.
App/installer platform signatures and upstream payload provenance remain separate.

Before publication: complete actual target-specific license text/source/native
library obligations (including SQLite, Node internals, browser codecs/FFmpeg and
packaging tools), validate source retrieval, run clean installs/update/uninstall
without evidence removal, native/hardware/accessibility and three real human
sessions. Do not dispatch CI, upload, sign with real keys or configure services
without owner authorization.
