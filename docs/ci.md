# GitHub Actions checks

These workflows check compilation, tests and the private feasibility harness;
they do not qualify desktop behavior or publish a release.

- `foundation.yml` runs on relevant pushes, pull requests and manual dispatches
  on Ubuntu 24.04, Windows Server 2025 and macOS 14 (Apple Silicon).
- `phase0.yml` uses the same triggers and OS matrix, plus the private pinned
  Node/Playwright runtime pack and sandboxed loopback smoke tests.
- `release-local.yml` is **manual only**. It also checks Intel macOS and builds
  the release binary for inspection. It does not upload or sign artifacts.

Foundation and Phase 0 manual dispatches accept a `runner` choice to check just
one platform. Relevant pushes and pull requests still check the full matrix.
For example, to investigate Windows without repeating Linux/macOS:

```sh
gh workflow run foundation.yml --ref main -f runner=windows-2025
gh workflow run phase0.yml --ref main -f runner=windows-2025
```

All jobs use `.github/actions/setup/action.yml`: Python 3.12, the Rust version
from `rust-toolchain.toml`, formatting/lint components and cached Cargo downloads.
Linux development libraries are installed only on the disposable hosted runner.
Tests use a canonical temporary directory, avoiding macOS's `/var` symlink. The
Windows command wrapper sets its own process token's default owner to its user,
not the elevated runner's Administrators group; existing owners and the product's
strict private ACL checks are unchanged.

The Phase 0 job installs browser shared-library prerequisites using its private,
pinned Playwright CLI. Ubuntu's AppArmor user-namespace restriction is handled by
an executable-scoped profile for only this job's exact pinned Chromium paths.
No host Node/npm, global policy/sysctl disablement or sandbox-disabling flags are
used. This hosted-runner policy is not default-policy desktop qualification.
The synthetic Phase 0 worker inherits stderr only when
`REGRESSEE_PHASE0_WORKER_STDERR=1` (enabled in its workflow) so startup failures
are diagnosable. This does not enable product-worker stderr logging.

## Local checks

```sh
cargo fmt --all -- --check
cargo test --locked --workspace --features regressee-app/test-support
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

The native target is now `cargo build --locked -p regressee-gui --features ui
--bin regressee-desktop`. GUI-only tests use `-p regressee-gui --features ui-tests`;
application/product tests remain in `regressee-app` with `test-support`.
The independent CLI target is `cargo build --locked -p regressee-cli
--bin regressee`; its normal/build dependency closure must remain free of GPUI.

The CLI test target also exercises six simultaneous real engine autostarts,
read-only status without a display/keychain, protocol mismatch refusal, stale
endpoint restart and graceful stop. Application `--features test-support --test
engine` runs the isolated multi-client/ownership/durability/control tests; native
`--features ui-tests --test engine_client` attaches the production GUI facade to
an engine over IPC. Neither substitutes for native Windows/macOS IPC qualification.

`test-support` exposes owned-loopback fixture helpers without pulling in GPUI.
The `product_flow` integration target requires it; `ui-tests` and `--all-features`
also enable it. Plain `cargo test --locked` still runs ordinary tests, but omits
that fixture target. Browser/native/scale tests marked `#[ignore]` retain their
explicit prerequisite and authorization requirements; CI does not silently run
them or claim their qualification.

Local all-feature checks need the documented native build prerequisites. The
Linux linker-name helper in `scripts/build-ui.sh` is for the existing development
host, not a replacement for installing runner prerequisites.
