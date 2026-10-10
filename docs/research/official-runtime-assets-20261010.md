# Official-source runtime assets — 2026-10-10

## Pins and scope

The application embeds `crates/runtime/assets/official/{linux-x86_64,windows-x86_64,macos-aarch64,macos-x86_64}.json` and `bundled-worker.zip`. These are app-owned **OfficialPinned** source catalogs, not signed runtime packs and not a **QualifiedPublic** claim. No private signing key or generated self-signature is involved. Existing signed-pack authentication remains a separate path.

Exact tuple: Node **24.21.0**, Playwright and playwright-core **1.64.0**, worker protocol identity **5.0.0**, Chromium **156.0.8078.4**, Playwright revision **1248**. The Playwright npm package's `browsers.json` and `lib/coreBundle.js` are the source of the browser version, revision, and executable layout; Google's matching Chrome for Testing JSON independently supplies the exact download URLs. The selected Node release was the official current LTS when researched, but runtime installation never queries `latest`. Catalog support expires on 2027-01-10 UTC and requires a reviewed rebuild.

Each catalog contains archive SHA-256 and exact byte counts, complete final-file SHA-256 and size inventory, normalized ordinary file modes, and contained bundle-link targets. Node contributes **only** `bin/node` (Windows `node.exe`); npm, shell installers, and global executables are not installed. Full Chrome for Testing is downloaded, not just a headless shell, because the product worker selects `channel: 'chromium'` for both screenshot and headed login flows. The worker's `chromiumSandbox: true` remains unchanged. Linux setuid bits from the ZIP are normalized away, with no privileged installation or OS policy changes.

## Sources and verification

- npm metadata: `https://registry.npmjs.org/playwright/1.64.0` and `https://registry.npmjs.org/playwright-core/1.64.0`. Tarballs use the exact `dist.tarball` paths. Pinned SHA-512 SRI is verified **before** parsing package members. The generator additionally verifies npm's registry ECDSA signature when OpenSSL is present, using the pinned public registry key `SHA256:DhQ8wR5APBvFHLF/+Tc+AYvPOdTpcIDqOhxsBHRwC7U`, cross-checked against `https://registry.npmjs.org/-/npm/v1/keys`. This is registry signature verification, **not** independent Microsoft publisher or Sigstore/SLSA attestation verification. Registry-provided signature and attestation metadata is retained in `provenance.json` without overstating it.
- Node: `https://nodejs.org/dist/v24.21.0/SHASUMS256.txt`, its clear-signed `.asc`, and exact `.tar.gz`/Windows `.zip` archives. When GPG is available, checksum signatures are verified in a **new private keyring**, using only explicit active Node release-key fingerprints in the generator, with key bytes fetched from `https://raw.githubusercontent.com/nodejs/release-keys/main/keys/`. No user keyring or network keyserver is touched. The clear-signed payload adds a final blank line; all nonblank checksum records must exactly match the plain file. Archive SHA-256 is then verified before reading Node bytes. Provenance records the actual GPG result and signer; unavailable GPG is explicitly reported as TLS checksum trust only.
- Chrome: `https://googlechromelabs.github.io/chrome-for-testing/156.0.8078.4.json`, cross-checked against Playwright `browsers.json`. Downloads are pinned direct URLs under `https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.4/`. Google does not provide an independent checksum/signature through this API. The generation-time trust here is authenticated official HTTPS bytes, converted into app-owned immutable SHA-256 pins; it is **not** a publisher-signature claim.

All fetches use Python's default verified TLS. Redirects are refused. Archives are never executed; no downloaded Node, Chrome, npm lifecycle script, browser smoke, or system installation is run by this tool. Archive files remain in a mode-0700 directory under `/path/to/private-tests/opencode/regressee-official-assets-*`, not in Git. Only the compressed worker/vendor source payload, catalogs, licenses, and provenance enter the source tree.

## Worker payload and licenses

`scripts/official-runtime.py` retains **all** published files in both npm packages, including API entry points, bundled runtime dependencies, types, tools, data, LICENSE, and NOTICE. It adds only `worker/product/src/**`, that product's explicitly pinned package/lock files, and the project's MPL license. Neither root `worker/` nor `.runtime` is changed. No development test fixtures or user project files are bundled.

The payload uses sorted paths, a canonical 2000-01-01 ZIP timestamp, ordinary 0644/0755 modes, and fixed deflate compression. It is deterministic for the same source bytes and generator/compressor implementation. A SHA-256 of the entire ZIP is in every catalog. Inventory rejects duplicate/case-colliding paths, nonportable or traversing names, leaf/directory collisions, special entries, and browser links outside their outer `.app`/`.framework` boundary, dangling links, cycles, and excessive resolution depth. Mac signed file bytes and framework aliases are preserved, not flattened. Ordinary executable modes are retained; privilege bits are never installed.

`crates/runtime/assets/official/LICENSES/` carries MPL-2.0, both Apache-2.0 package licenses/notices, and Node's full license including bundled third-party notices. Chrome binary bytes and bundled resources remain in the downloaded official distribution; Chromium's upstream licensing and credits apply to that distribution. No Node or browser binary archive is committed.

## Generated results

The complete four-platform generation succeeded. Node checksum signatures were actually GPG-verified under primary fingerprint `5BE8A3F6C8A5C01D106C0AD820B1A390B168D356`; both npm registry signatures and SHA-512 SRI checks were actually verified. Repeating generation from the same authenticated archives produced the same source payload hash.

`bundled-worker.zip`: **4,177,469 bytes**, **184 regular entries**, SHA-256 `0646531342f9a4c0c00253df68e4702dd9bc366fd487e8a299f2a492b3dd9fe9`.

| Catalog | Files | Links | Download bytes | Installed file bytes |
| --- | ---: | ---: | ---: | ---: |
| Linux x86_64 | 486 | 0 | 258,433,266 | 566,268,773 |
| Windows x86_64 | 490 | 0 | 244,691,932 | 569,957,372 |
| macOS arm64 | 529 | 5 | 247,741,846 | 523,665,903 |
| macOS x86_64 | 529 | 5 | 259,536,079 | 537,892,183 |

All catalog JSON files are below 250 KiB. Each archive's explicit directory entries have cataloged descendants, so the strict extractor does not need to create unlisted empty directories. Actual launch/containment qualification has **not** been run in this task.

## Integration caveats (not qualification evidence)

- Linux x86_64 is the immediate Fedora/Wayland target. Windows x86_64 has a catalog; both still require the runtime integrator's real managed-worker smoke before activation.
- macOS arm64 and x86_64 catalogs are preparation only. The existing process supervisor currently refuses macOS; catalog presence must **not** enable it before equivalent containment, launch, and qualification are implemented. Catalog generation does not claim Gatekeeper/notarization qualification.
- Playwright 1.64.0 writes `browsers/chromium-1248/DEPENDENCIES_VALIDATED` after successful host-dependency validation. This is mutable upstream bookkeeping, **not** a signed catalog member. A strict runtime inventory must deliberately handle that marker (for example, removing only this known ordinary marker after each worker stops and before re-verifying). Do not bundle a fabricated recent validation marker or disable validation to conceal the inventory mismatch.
- Inspection of this exact package shows Chromium host-dependency scans targeting legacy `chrome-linux` / `chrome-win` directories, while executable layout is `chrome-linux64` / `chrome-win64`. Those scan directories are absent in the actual CfT distribution; consequently Chromium's current Windows launch path does not invoke `winldd`. `winldd` is not bundled based on this inspected behavior, not an assumption. Future package updates must revisit it. FFmpeg is needed only for video recording, which this worker does not request. A real browser smoke remains necessary to detect missing OS libraries; catalog generation is not a host-dependency test.

## Regeneration

```sh
python3 scripts/official-runtime.py
```

Requires Python 3.11+, approximately 3 GiB free space, verified HTTPS access to the above official sources, and optionally GPG/OpenSSL for the recorded signature checks. Archives remain private after completion or failure. `--work-dir` can reuse an approved private download directory after a transient failure; all immutable pins are rechecked. `--platform` restricts a development regeneration; full release regeneration must include all four catalogs so provenance covers the complete set. The generated provenance supplies exact archive lengths/hashes, catalog hashes, bundled-payload hash, npm metadata, and actual signature-verification outcomes. Build/test/smoke execution belongs to the parent integration session, not this asset-generation task.
