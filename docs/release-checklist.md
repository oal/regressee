# Release checklist

Operational checklist for preparing and publishing a Regressee release. Use it
together with:

- [Build, package and preview instructions](release-building.md) — products,
  packaging and the production gates that are still open.
- [GitHub Actions checks](ci.md) — what CI verifies and what stays manual.
- [Support matrix](support-matrix.md) — the qualification limits every public
  statement must respect.

**Current policy:** only unsigned, experimental **test previews** are published,
as GitHub prereleases tagged `vX.Y.Z-preview.N`. A stable production release
still has open qualification gates and requires explicit owner authorization.
Never describe a preview as a supported release.

## At a glance

1. Pick the next preview number; bump crate versions only when the series
   changes.
2. Rewrite `docs/preview-release-notes.md` for the new preview.
3. Run the pre-tag gates; commit and push to `main`.
4. Push the `v…-preview.N` tag, or dispatch the release workflow with
   `release_tag`; it publishes only after both platforms pass.
5. Verify the published prerelease and its checksums.
6. Update the `site/` download links; the Pages workflow redeploys the page.

## 0. Before you start

- [ ] A human asked for this release. Agents do not publish releases on their
  own initiative, and never sign with real keys, upload elsewhere or configure
  services without owner authorization.
- [ ] Pick the next preview number (`preview.10` → `preview.11`). Never reuse,
  move or delete a published tag or release; a failed release gets the next
  number (recovery precedent:
  `docs/research/preview7-release-recovery-20261018.md`).
- [ ] The release commit is on `main` and `git status` is clean; the tag will
  point at that exact commit.

## 1. Bump versions when the series changes

Preview numbering does **not** change crate versions: `0.1.0` stays `0.1.0`
across `v0.1.0-preview.N`. Only when the version series itself changes:

- [ ] Update `version` in
  `crates/{core,storage,runtime,app,gui,cli}/Cargo.toml`. `crates/spike` is a
  feasibility crate and stays out of product releases. Worker, engine-protocol
  and CLI-schema versions are separate internal boundaries — change them only
  when those contracts actually change.
- [ ] Refresh and commit `Cargo.lock` in the same commit; locked builds refuse
  a stale lockfile.
- [ ] Packaging checks that the CLI and desktop agree on a version and that
  each executable's build-info matches its crate, so confirm with a locked
  build before tagging.

## 2. Update the release notes

- [ ] Rewrite `docs/preview-release-notes.md`: the new `preview.N` heading and
  title, what changed, and the honest limits. The release workflow appends this
  file to the GitHub release notes and the packages copy it into their help
  directories, so it must describe the tagged commit truthfully.
- [ ] Adjust statements that reference the version (for example "the
  application crate version remains `0.1.0`"), and confirm README and
  support-matrix claims still hold for this commit.

## 3. Run the pre-tag gates

Run serially in a dedicated checkout with pre-provisioned prerequisites. These
fail fast locally; the workflow re-runs its own copies of the tests on both
platforms and adds packaging and ABI/dependency inspection.

```sh
cargo fmt --all -- --check
cargo build --locked --workspace --all-features --all-targets
cargo test --locked --workspace --all-features -- --test-threads=1
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
```

```sh
python3 -B scripts/test_ci_run.py
python3 -B scripts/test_package_preview.py
python3 -B scripts/test_release_preview.py
python3 -B scripts/test_cli_consumer_fixture.py
python3 -B scripts/test_skill_frontmatter.py
python3 -B scripts/test_environment_boundaries.py
# Linux only, with a fresh private report path outside the repository:
python3 -B scripts/test_source_release.py --report <new-private-report>
```

- [ ] Optional local package rehearsal (the output directory must be new; the
  helper refuses dirty sources unless `--allow-dirty` is explicit):

```sh
python3 -B scripts/release-preview.py --target x86_64-unknown-linux-gnu --product all --output artifacts/local-preview
```

- [ ] All gates pass. Known failures are recorded truthfully, never silenced,
  and the release notes name them.

## 4. Commit and freeze

- [ ] Commit every release input (version, notes, docs) and push to `main`.
- [ ] Let the push CI (`foundation.yml`, `phase0.yml`) run green on the release
  commit, or rerun the full local gates if you deliberately skip that.
- [ ] Confirm `git status` is clean. Do not edit the repository after tagging;
  the workflow checks out the tagged commit (`github.sha`) exactly.

## 5. Publish through the release workflow

The only release pipeline is **Actions → Downloadable Linux and Windows
previews** (`.github/workflows/release-preview.yml`). `release-local.yml` is a
private platform check that never publishes, and `scripts/release-candidate.py`
is historical private tooling. Choose one trigger:

- **Push a tag** — builds and publishes immediately:

  ```sh
  git tag -a v0.1.0-preview.N -m "Regressee preview N"
  git push origin v0.1.0-preview.N
  ```

- **Manual dispatch** — the tag is created only after both platforms pass; the
  request must match `vX.Y.Z-preview.N` and must not already exist:

  Actions → Downloadable Linux and Windows previews → Run workflow, or:

  ```sh
  gh workflow run release-preview.yml --ref main -f release_tag=v0.1.0-preview.N
  ```

- [ ] Watch both platform build jobs (Ubuntu 24.04 and Windows Server 2025):
  contract tests, production packaging, CLI dependency-closure check, Linux
  GLIBC ≤ 2.39 check and Windows DLL inspection.
- [ ] Only the publish job creates the GitHub **prerelease**, after both builds
  succeed. A failed platform blocks the release; a failed upload can leave a
  draft. Never overwrite a published release — fix and use the next preview
  number.

## 6. Verify the published release

- [ ] `gh release view <tag>` (or the web UI) lists both
  `regressee-desktop-*` and `regressee-cli-*` platform archives plus
  `regressee-source.tar.gz`, `regressee-source.json` and `SHA256SUMS`.
- [ ] The notes contain the preview notes and the source-commit line, and the
  release is a prerelease (not marked latest).
- [ ] Spot-check a download against `SHA256SUMS`:

  ```sh
  sha256sum --ignore-missing --check SHA256SUMS
  ```

## 7. Update the website

Only after the release exists, so no published link 404s:

- [ ] `site/app.js` — the `RELEASES` download base URL → the new tag.
- [ ] `site/index.html` — the desktop download buttons, CLI links, displayed
  archive sizes and the "Version …" note → the new tag.
- [ ] Push to `main`; the **Landing page** workflow redeploys GitHub Pages when
  `site/**` changes (or run it manually). Confirm the run succeeds and click
  through the deployed download links.

## 8. Close out

- [ ] Record the released tag and workflow run where recovery work is tracked
  (`docs/research/…` when repairs were involved).
- [ ] Rewrite `docs/preview-release-notes.md` for the next preview only when
  preparing it; the current text already lives in the published release.
- [ ] Do not retag, do not delete released assets, and do not claim stable or
  production readiness.
