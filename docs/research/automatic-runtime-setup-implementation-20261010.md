# Automatic runtime workflow — bounded implementation handoff

**2026-10-10: source changes delivered; compilation/tests NOT RUN.**
Only `crates/app/src/product.rs`, `crates/app/src/jobs.rs`,
`crates/runtime/src/distribution.rs` and this report changed in this implementation
task. Tests are inline in the owned app files. No builds/formatters/tests, downloads,
app/native/worker/browser launches, public-site visits or subagents were run.
Parent owns UI/main/core/Cargo/status changes and serial verification.

## Delivered behavior

- First-use prerequisites now run inside the **same Jobs request, background
  owner and cancellation slot** as new discovery/new-baseline capture. No second
  setup request, UI-thread I/O or automatic repair queue.
- `OpenLogin`/`ProbeEnvironment` provision automatically only when the project has
  **no active baseline**; otherwise they keep the existing resolver behavior so
  login/probe for a pinned comparison cannot install a new default.
- Legacy `Capture { check:false }` and
  `CaptureInventory { check:false, resume:None }` provision on first use. Explicit
  new discovery/replacement is allowed even when a historical baseline exists.
  Comparisons, **every resume**, existing-inventory selection, image previews,
  confirm/close login and read-only status never invoke automatic setup.
- Projects are loaded/validated before setup. Existing continuation functions keep
  their revision/environment/runtime-identity/scope checks. Direct low-level
  `Product::discover`/auth/capture calls retain their old resolve-only behavior;
  the UI Jobs workflow supplies the automatic prerequisite. Direct callers can
  explicitly call `ensure_runtime` for unpinned new work.
- Automatic installation requires valid configured trust, an eligible adapter
  (Linux/Windows x86-64), safe physical cache ancestors and a genuinely absent
  `current` entry. An existing malformed/dangling pointer, missing generation,
  damaged inventory, expired support/signature or path refusal **never** falls
  back to download. Existing ready generations are reverified, not redownloaded.
- Cancellation is checked before the prerequisite, after setup/verification and
  immediately before the original owner action. Installer phases also check it
  before extraction/verification/smoke. Cancellation after atomic publication
  may retain the valid new generation, but cannot continue to website work.
- Missing trust is explained as **this build being incomplete**, not a user
  requirement to install Node/npm or configure developer tooling. There are no
  new defaults, hosts, keys, pins, scope promotions or release-catalog entries.

## Exact parent UI API

`regressee_app::product` exports:

```rust
pub enum RuntimeState {
    UnavailableDistribution, UnsupportedPlatform, NotInstalled, Ready, RepairRequired,
}
pub struct RuntimeReadiness {
    pub status: RuntimeState,
    pub setup_available: bool,
    pub message: &'static str, // UI-safe, no debug paths/URLs/credentials
    pub identity: Option<RuntimeIdentity>,
    pub scope: Option<Scope>,
}
```

- **New reply:** `Reply::RuntimeReadiness(RuntimeReadiness)` is the terminal reply
  for both `Work::RuntimeStatus` and successful `Work::SetupRuntime`. Handle this
  variant in parent main/UI matches and store the struct, not environment presence.
  Invalid Product configuration at actor startup also returns this typed
  unavailable state for RuntimeStatus. Storage-unavailable behavior is unchanged.
- `setup_available` means validated configured trust + eligible adapter, **not**
  already installed, permission to use a development pack on public sites, or
  external release/platform qualification. Inspect `status`/`scope` separately.
- `Product::readiness(&AtomicBool) -> Result<RuntimeReadiness>` is blocking,
  read-only, with no network/download/cache creation; call only off the UI thread.
- `Product::ensure_runtime(Arc<AtomicBool>, &mut dyn FnMut(InstallProgress))
  -> Result<Installed>` is for **unpinned new work only**.
- `Product::setup_with_progress(Arc<AtomicBool>, &mut dyn FnMut(InstallProgress))
  -> Result<String>` is explicit setup/repair. Existing `Product::status`,
  `Product::setup` byte-callback signatures and `Reply::Runtime(String)` remain
  available for compatibility, although Jobs now uses typed terminal readiness.
- Error context keeps the underlying `anyhow` diagnostic chain while ordinary
  setup/path/verification UI errors are safe explanations. For private diagnostics,
  existing `Product::status` still returns resolver errors; do not render full
  chained debug/path/network errors as normal first-use UI copy.

`regressee_runtime::distribution` exports new:

```rust
pub enum InstallProgress {
    Downloading { bytes: u64, total: Option<u64> },
    Verifying, Extracting, TestingBrowser, Ready,
}
```

`setup_network_with_progress` and `install_with_progress` add this callback;
existing setup/setup_network/install APIs retain their signatures as wrappers.
`current_absent(&Path) -> Result<bool>` checks physical ancestors and distinguishes
absence from resolver failure (including dangling symlinks).

Jobs retains `Reply::Progress { message, completed:u32, total:Option<u32> }`:
download counters are **actual bytes** under the unchanged 2 GiB cap, with a common
overflow-safe scale for larger future values. Unknown/zero totals and all non-byte
phases are indeterminate (`total:None`), not fabricated 0/1 percentages. Messages
label only actual stages. Progress remains best-effort over the existing bounded
channel; terminal delivery remains reliable under existing actor rules. Clear
setup-phase progress when the original operation emits its own page/target counts.

## Test source added (not executed)

- `product::automatic_runtime_tests`: missing/invalid trust, read-only status,
  eligible platform matrix, Mac refusal (Mac-only), configured first installation
  and existing-generation reuse, tampered files, invalid/dangling pointers,
  missing generation, linked cache, signed expired support, cancellation before/
  during/after installation and after an installer returns success.
- Tests use **signed inert inventories**, private fixture keys and an injected
  private installer callback. They exercise real signature/hash/path/atomic
  installer validation, with explicitly inert smoke acceptance. They never execute
  Node/browser and are not public runtime or screenshot-smoke qualification.
- Jobs tests cover comparison/resume/no-baseline auth routing, byte/phase counters,
  typed no-trust status, same-request missing-trust failure and queued cancellation.
  No automatic network download integration is claimed from these unit tests.

Parent should first adapt the new reply variant, then format/compile and run these
bounded tests **serially**. No full guardian/soak/native/browser rerun is required
by this handoff. No PASS is asserted here.

## Still open

No usable public default distribution was fabricated. Approved authenticated
current artifacts, real owner trust/signatures, authorized hosting or the separately
designed signed official-component provisioner remain external/code dependencies.
macOS still cannot setup/capture because exact-owned-tree supervision is absent;
its original supervisor guard is untouched. Windows/Linux native/package/sandbox,
all-three-OS release and existing native/UI/GPU qualifications remain OPEN.
TLS/no-redirect policy, sandbox, signatures, path/link validation, process ownership,
baseline identity/evidence and DevelopmentLoopback capture guards are unchanged.
