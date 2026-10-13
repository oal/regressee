# Final independent preview delivery verification

Date: **2026-10-13**. **PASS — published preview availability, hosted build gates,
source binding and download integrity. Finite defects: none.** This is not native
execution, clean-install, accessibility, usability or supported-release acceptance.
No binaries were launched, renders/Rust suites repeated, or commits, pushes,
dispatches, tags, release edits/overwrites or cancellations performed here.

## Published identity and actual hosted outcomes

[**v0.1.0-preview.2**](https://github.com/oal/regressee/releases/tag/v0.1.0-preview.2)
is published, **draft=false / prerelease=true**, with exactly five uploaded assets;
publication timestamp is `2026-10-13T02:39:33Z`. Release metadata and all inspected
runs identify **`1911c3b9905d727ac830a420eb6935ceb4d380e9`**.
The annotated tag object's SHA is `bf69b625e626dc0b94692295e4b1d135a1ac45af`;
both local and remote peeled tags resolve to that exact tested source commit.

Fresh independent `gh run view` inspections confirm all runs **completed / success**
and every job succeeded, not just the overall status:

| Trigger / workflow | Run | Successful jobs |
|---|---|---|
| Main Foundation | [38329625059](https://github.com/oal/regressee/actions/runs/38329625059) | Ubuntu 24.04, Windows 2025, macOS 14 |
| Main Phase 0 | [38329625072](https://github.com/oal/regressee/actions/runs/38329625072) | Ubuntu 24.04, Windows 2025, macOS 14 |
| Tagged Foundation | [38333821918](https://github.com/oal/regressee/actions/runs/38333821918) | Ubuntu 24.04, Windows 2025, macOS 14 |
| Tagged Phase 0 | [38333821912](https://github.com/oal/regressee/actions/runs/38333821912) | Ubuntu 24.04, Windows 2025, macOS 14 |
| Tagged preview release | [38333821892](https://github.com/oal/regressee/actions/runs/38333821892) | Linux build, Windows build, publish |

Thus main CI has **6/6** successful jobs; the tag has **9/9**, **15/15** across
these five runs. Foundation's full contracts, strict all-target/all-feature lint
and native build/simulated tests passed on all three hosts. Phase 0's core tests,
private runtime bootstrap, sandboxed startup and pack/provision/smoke/protocol
steps passed; applicable Windows/macOS GPUI compilation passed. Release contract,
native release build, Linux ABI and Windows DLL checks, packaging/upload and
publication passed. Expected non-host-specific conditional steps may be skipped;
they are not counted as qualifying that host. The earlier Windows timeout remains
a historical failure, not relabeled; the new complete matrix establishes repair.

## Independent raw download and archive checks

All five assets were **freshly downloaded independently**, not merely trusted from
the parent receipt. Each fresh file matches GitHub's reported size and SHA-256
digest and is byte-identical to the parent's download. The manifest covers exactly
the four other assets, each passes `sha256sum --check`; GitHub's digest independently
covers `SHA256SUMS` itself. Hashes establish integrity, not publisher authenticity
or signing.

| Asset | Bytes | SHA-256 |
|---|---:|---|
| Linux TAR | 24,443,346 | `97239b6bad97bcc78cbe0eb73d1d934d6bbd41fbb534af01bcb5c65618188cf9` |
| Source TAR | 11,719,357 | `4feaef1f838575b1d2581303eb924ee7f0f06d5237e42127b9d1e9a3fbac7ffb` |
| Windows EXE | 33,300,992 | `12c4fce338ecc9f696e714bc27333dff8c6ba2a1d8c3d88e378be482a7a646bf` |
| Windows ZIP | 19,795,408 | `608a2712e5a3184fba6f0165f8968c01ec65e72a09607b912af3e2416fb80d66` |
| SHA256SUMS | 376 | `5911446eb8383d22f39bc9ed0f3ef3abb497f3500842279182d10b1c325e0075` |

- **Exact source:** all **616 regular files** match `git archive` of the tested
  commit byte-for-byte. Entire member sets, types, modes, sizes and link targets
  also match. No extraction or source execution was needed.
- **Binary identity:** archive `build-info.json` reports the exact source SHA,
  version `0.1.0`, `features=["ui"]`, appropriate Linux GNU / Windows MSVC x64
  target and pinned Rust 1.98.1. Binary hashes match their actual archived bytes.
  Static header inspection confirms Linux ELF64 little-endian x86-64 and Windows
  PE32+ AMD64. The standalone EXE equals the ZIP executable exactly. Linux's
  archived binary hash is
  `4ac6b9c22f1f34677ecf4d36960ea5aa001d924ce864e58952f029f3540511f6`;
  its executable mode is `0755`.
- **Inventory integrity, not legal approval:** every recorded notice-text hash
  matches actual archive bytes: **785** Windows texts / **475** package entries,
  **902** Linux texts / **558** package entries. Notice member sets match the
  inventories exactly. Both declare `normal,build` edges, `ui`, the correct
  target, `release_approved=false` and `native_inventory_complete=false`.
- **No private/runtime sweep:** platform archives have exactly the executable,
  build/inventory metadata, LICENSE/README/notices, and seven explicitly listed
  help documents—**798** Windows entries / **915** Linux entries. There are no
  duplicate/traversal members or extra recursive runtime/data entries. Source
  members match the exact tracked archive and contain no `.runtime`, target,
  artifacts, node_modules, `.git`, SQLite database/sidecar or key/PEM entries.
  This verifies archive scope, not an exhaustive secret-content/security audit.
- **Windows CRLF diagnostic preserved:** Linux preview notes equal the source
  bytes; Windows notes do not. Replacing only CRLF with LF yields exactly the
  source text. Their raw hashes remain distinct: Windows
  `39e366088cea5534c01179ba39b713dcef25cd13f63e443ec5bd5fbfb068f4b4`,
  Linux/source
  `6e85ac8e5765a081e0b52588dcc625f4be0ddabaedbdf2805bf0c2a5f2805712`.
  The parent's initial byte-equality assumption was a verification diagnostic,
  not a product defect; no downloaded content or hash was normalized/rewritten.
- Both parent receipts, [hosted verification](delivery-hosted-verification.json)
  and [release receipt](delivery-release-receipt.json), agree with freshly fetched
  metadata and outcomes. Their counts and CRLF distinction were independently
  reproduced.

## Actual commands and retained evidence

Independent evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-preview2-independent-z_us3dy_`.
It contains fresh `release.json`, five `run-<id>.json` files, all five assets in
`downloads/`, the read-only `verify.py` and completed `verification.json`.
Parent files were read, not modified, at
`/var/home/olav/Data/opencode/tmp/opencode/regressee-preview2-download-prp9isn8`.

Commands completed successfully:

```sh
git rev-parse 'v0.1.0-preview.2^{commit}'
git ls-remote --tags origin 'refs/tags/v0.1.0-preview.2*'
# These exact gh commands were invoked for each ID by the independent harness:
for id in 38329625059 38329625072 38333821918 38333821912 38333821892; do
  gh run view "$id" --json databaseId,name,url,status,conclusion,headSha,headBranch,jobs
done
gh release view v0.1.0-preview.2 --json tagName,targetCommitish,isDraft,isPrerelease,publishedAt,url,assets,body
gh release download v0.1.0-preview.2 --dir /var/home/olav/Data/opencode/tmp/opencode/regressee-preview2-independent-z_us3dy_/downloads
python -B /var/home/olav/Data/opencode/tmp/opencode/regressee-preview2-independent-z_us3dy_/verify.py
# Invoked by the verifier, without launching a product binary:
git archive --format=tar --prefix=regressee-source/ 1911c3b9905d727ac830a420eb6935ceb4d380e9
# In the fresh downloads directory:
sha256sum --check SHA256SUMS
```

## Downloads and remaining acceptance

- [Windows EXE](https://github.com/oal/regressee/releases/download/v0.1.0-preview.2/regressee-windows-x86_64.exe)
- [Windows ZIP with help/notices](https://github.com/oal/regressee/releases/download/v0.1.0-preview.2/regressee-windows-x86_64.zip)
- [Linux TAR](https://github.com/oal/regressee/releases/download/v0.1.0-preview.2/regressee-linux-x86_64.tar.gz)
- [Exact source TAR](https://github.com/oal/regressee/releases/download/v0.1.0-preview.2/regressee-source.tar.gz)
- [SHA256SUMS](https://github.com/oal/regressee/releases/download/v0.1.0-preview.2/SHA256SUMS)

**All seven phases are implemented, feasibly verified, committed and pushed; the
cross-platform CI pipeline is green and the unsigned experimental preview is
available.** Native desktop/default graphics/input, Windows 11/NVDA,
macOS/VoiceOver, clean installation and end-user first-use runtime installation,
host scaling, participant usability, security/installer/signing and complete
native/transitive license acceptance remain **BLOCKED or NOT TESTED** within the
existing platform evidence. Hosted disposable-runner runtime success does not
close those native/clean-install gates. macOS has no preview download and retains
its supervision limitation. Linux needs compatible shared libraries/graphics;
first-use browser setup needs internet. Do not disable OS/browser security.

Parent's remaining action is the final documentation/evidence commit/push and
user-facing links—not artifact replacement or retagging. These final receipts
postdate the released source archive; a later documentation-only commit does not
change the tested binary/source SHA or imply those documents were already in the
616-file release archive. No verification process remains pending.
