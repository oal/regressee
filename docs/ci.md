# GitHub Actions checks

These workflows check compilation, tests and the private feasibility harness;
they do not qualify desktop behavior or publish a release.

- `foundation.yml` runs on relevant pushes, pull requests and manual dispatches
  on Ubuntu 24.04, Windows Server 2025 and macOS 14 (Apple Silicon).
- `phase0.yml` uses the same triggers and OS matrix, plus the private pinned
  Node/Playwright runtime pack and sandboxed loopback smoke tests.
- `release-local.yml` is **manual only**. It also checks Intel macOS and builds
  the release binary for inspection. It does not upload or sign artifacts.

All jobs use `.github/actions/setup/action.yml`: Python 3.12, the Rust version
from `rust-toolchain.toml`, formatting/lint components and cached Cargo downloads.
Linux development libraries are installed only on the disposable hosted runner.
The Phase 0 job installs browser shared-library prerequisites using its private,
pinned Playwright CLI. No host Node/npm or sandbox-disabling flags are used.

## Local checks

```sh
cargo fmt --all -- --check
cargo test --locked --workspace --features regressee-app/test-support
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

`test-support` exposes owned-loopback fixture helpers without pulling in GPUI.
The `product_flow` integration target requires it; `ui-tests` and `--all-features`
also enable it. Plain `cargo test --locked` still runs ordinary tests, but omits
that fixture target. Browser/native/scale tests marked `#[ignore]` retain their
explicit prerequisite and authorization requirements; CI does not silently run
them or claim their qualification.

Local all-feature checks need the documented native build prerequisites. The
Linux linker-name helper in `scripts/build-ui.sh` is for the existing development
host, not a replacement for installing runner prerequisites.
