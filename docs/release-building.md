# Private build and release preparation

## Downloadable test previews

`.github/workflows/release-preview.yml` builds the desktop app on Ubuntu 24.04 and
Windows Server 2025 for Linux/Windows x86_64. It publishes a GitHub **prerelease**
only after both platforms pass contract tests, native release compilation, and
dependency inspection. These owner-requested testing downloads are not production
release qualification. The existing private-check workflow remains separate.

Once the workflow is committed and pushed to the default branch, open
**Actions → Downloadable Linux and Windows previews → Run workflow**. Choose the
branch containing the source you want to test. This creates an immutable
`preview-<run-number>-<short-commit>` release. Alternatively, push a new `v*` tag
(for example, `v0.1.0-preview.1`) to build and publish under that tag. Both triggers
currently produce experimental prereleases, never a supported stable release.

The repository must allow Actions and the publishing job's `contents: write`
permission. No personal access token, signing key, or external hosting is needed;
only the publishing job receives write access through `GITHUB_TOKEN`. The build
jobs also retain downloadable Actions artifacts for 14 days. A failed platform
build prevents release creation; a failed upload can leave a draft. Existing
releases are not overwritten on reruns: dispatch a new preview, or explicitly
remove a failed draft before rerunning its publishing job.

### Downloads and requirements

- **Windows 11 x64:** download `regressee-desktop-windows-x86_64.exe` and run it as your
  normal user. No Rust, Visual Studio, Node.js, npm, or Playwright installation is
  needed. The MSVC runtime is statically linked; GPUI's release shaders are compiled
  into the executable. The unsigned app may produce a Windows publisher/reputation
  warning. No signing or security-policy bypass is performed by this workflow.
  Download `regressee-desktop-windows-x86_64.zip` as well for help and dependency notices.
- **Linux x64:** extract `regressee-desktop-linux-x86_64.tar.gz`, then run
  `./regressee-desktop-linux-x86_64/regressee-desktop`. Use Ubuntu 24.04 or a compatible glibc-based
  desktop with a working GPU/graphics backend. Shared libraries remain system
  prerequisites; this is a portable directory, **not an AppImage or installer**.
  On Ubuntu 24.04, GPUI's basic runtime prerequisites can be installed with:

  ```sh
  sudo apt-get install libasound2t64 libfontconfig1 libfreetype6 libvulkan1 \
    libwayland-client0 libwayland-cursor0 libxcb1 libxkbcommon0 libxkbcommon-x11-0
  ```

  Browser shared-library and default AppArmor/sandbox compatibility still need
  native qualification; do not use `--no-sandbox` or disable OS protections.
- **Both:** first-run browser installation needs internet access. The app embeds
  its worker and official-runtime catalog and downloads verified Node/Chromium
  components into its private local storage. The development `.runtime` directory
  is never included. App data is stored outside the extracted/download directory.

Each platform archive includes `build-info.json` (source commit, compiler, target,
and binary hash), help, project notices, and cached license texts with a
target-specific normal/build Cargo dependency inventory. Missing license texts
and unreviewed entries remain visible; this is not a completed native/transitive
license audit. `regressee-source.tar.gz` supplies the exact tracked application
sources, including vendored GPUI, at the same commit. The
[whole-app preview notes](preview-release-notes.md) are included in the platform
help directories and appended to the GitHub prerelease notes. `SHA256SUMS` covers
all assets. It detects download corruption, not publisher authenticity:

```sh
sha256sum --ignore-missing --check SHA256SUMS
```

On Windows, compare `(Get-FileHash .\regressee-desktop-windows-x86_64.exe -Algorithm SHA256).Hash`
with its entry in `SHA256SUMS`.

No macOS build, installer, app signing, or automatic updater is added. Hosted CI
build success does not establish Windows 11 desktop usability or clean-install,
accessibility, secure-store, browser-sandbox, or complete license qualification.
See [the support matrix](support-matrix.md) for those outstanding gates.

## Production release preparation (still pending)

Run serially in a dedicated checkout/image with pre-provisioned prerequisites:

```sh
cargo fmt --all -- --check
cargo build --locked --workspace --all-features --all-targets
cargo test --locked --workspace --all-features -- --test-threads=1
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
cargo build --release --locked -p regressee-gui --features ui --bin regressee-desktop
```

Linux: build in Ubuntu 24.04 userspace, inspect every ELF using `readelf -d` and
`readelf --version-info`. Reject GLIBC above 2.39; never patch in a host glibc.
AppImage/FUSE/extract-and-run remains a packaging recipe, not a delivered installer.
Run the exact packaged default-sandbox browser on default AppArmor/userns policy
without admin/sandbox bypasses. The development-host binary fails the ABI floor;
the preview workflow instead builds on Ubuntu 24.04 and enforces GLIBC ≤2.39.

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

## Developer-only tools

Normal builds (`--features ui`), including downloadable previews, ignore
`REGRESSEE_DEV_DATA` and `REGRESSEE_CAPTURE_TRACE_EVIDENCE`. These hooks require
the explicit `developer-tools` Cargo feature; inspection and renderer-capture
features enable it as well. Do not package an `--all-features` build as a preview.
`regressee-desktop --build-info` reports compiled features without initializing graphics
or storage. Preview packaging checks this metadata and rejects developer/test
features or a version mismatch.

For a source-only isolated launch, create a fresh owner-private directory without
symlink ancestors, then run:

```sh
REGRESSEE_DEV_DATA="/absolute/path/to/private-test-directory" \
  cargo run --locked -p regressee-gui --features ui,developer-tools --bin regressee-desktop
```

Do not point developer overrides at existing user data. Normal runtime download
configuration, browser scratch overrides, reduced motion and GPU diagnostics are
separate supported interfaces, not developer storage/tracing hooks.

Automated checks need no special development-host paths or temporary-directory
overrides:

```sh
cargo test --locked --workspace --all-features -- --test-threads=1
```

Deliberately ignored live-browser, native and scale tests require their explicit
fixture inputs and fresh private output directories. Do not run them as part of
an ordinary check. Diagnostic scripts accept caller-selected physical directories;
none requires a particular home directory or development tool.

Renderer tests require both `REGRESSEE_RENDERER_APPROVED_ROOT` and
`REGRESSEE_RENDERER_CAPTURE_DIR` explicitly; there is no machine-specific fallback.
The opt-in authorized-site test requires `REGRESSEE_TEST_RETAINED_STORE` when
`REGRESSEE_TEST_RUNTIME_CACHE` is supplied, so reuse is bound to the caller's
explicit retained store rather than a historical developer directory.

## CLI-only build (implementation skeleton)

```sh
cargo build --release --locked -p regressee-cli --bin regressee
cargo tree --locked -p regressee-cli --edges normal,build
```

This separate target does not depend on GPUI or require native GUI development
libraries, graphics initialization, a display or an unlocked keychain. Only
`--help`, `--version` and data-free `--build-info` exist in Phase 1; there are no
product operations or engine yet. The current preview scripts package the
desktop executable only. CLI-only archives and a desktop package containing both
executables belong to Phase 5, not a completed release claim. Historical previews
used the name `regressee` for the GUI; new builds use `regressee-desktop`.

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
