# First run and safe review

This is a private preview, not a public release. The default private browser uses
app-pinned official downloads; internet is needed once for setup, with no Node/npm
or terminal configuration. Historical development distributions remain restricted
to authorized literal-loopback fixtures. See [support matrix](support-matrix.md)
for the separate platform/clean-machine qualification limits.

1. **Create project**, enter a recognizable name and your test website
   address(es), then **Save website & continue**. This saves locally without
   visiting the site. Approve all listed websites when using multiple websites.
   Visits themselves can have effects: use a read-only test account, never an
   administrative production account.
2. In **Website access**, choose **No login required** or **Login required**.
   For login, **Save settings** first, then use **Open login browser**, finish
   login/consent yourself, and choose **I'm ready**. An unavailable secure store
   permits explicit **Only for this app session** login; it never silently saves
   plaintext. Production/staging sessions remain separate, and ZIP recipients
   establish their own login. Mapping, language and website-theme options remain
   available without dominating the first access screen.
3. **Continue to pages & sizes**. Choose **Only the starting addresses** or
   **Find linked pages**, website/section boundaries, exclusions and page limit.
   Choose screen sizes, full-page or visible-screen capture, image mode and speed.
   Gentle reduces site load; Fast can run four pages at once. Phone/tablet are
   Chromium emulation, not actual Safari/iOS hardware. Advanced controls remain
   available. **Save settings** before finding pages.
4. Use **Find pages & preview**, then inspect the included/excluded inventory in
   **Pages & baseline** before capturing. A page limit is incomplete discovery,
   not proof you found every page. An existing baseline remains unchanged.
5. Capture the reviewed pages, inspect coverage and warnings, then explicitly
   **Save baseline**.
   A baseline is the approved reference screenshots. Failed or missing targets
   cannot be called unchanged or silently approved.
6. **Review changes** → **Run check** revisits only the fixed baseline
   inventory. Use the coverage summary and searchable queue beside the comparison
   viewer to inspect changed screenshots and failed/non-comparable targets. Differences are evidence
   to review, not automatic bug verdicts. Other computers/fonts can change pixels.
7. Sensitivity and ignored rectangles change analysis only. Select compatible
   screenshots, inspect the approval consequences, then explicitly confirm the
   new baseline version. Other references are inherited, and old versions remain
   until deliberate manual deletion. **History** distinguishes baseline versions
   from the capture/check runs that supplied their evidence.

## Choosing and switching projects

Use **View results** in Projects to select a website and enter its workspace
without opening the editor. **Edit settings** selects that project and opens
**Project settings**. The blue project rail separates **Review changes**, **Pages
& baseline**, **History**, and **Project settings** from global **App settings**.
The header explicitly names the **Selected project** used by Results and History.
Use **Change project** to switch from either screen. If nothing is selected,
Results and History show the saved projects to choose from, not an empty review.
Switching projects does not change saved settings or start a capture; unsaved
edits require confirmation before switching to a different project.

## When something goes wrong

### Linux GPU initialization failure

Start with a plain launch:

```sh
./target/debug/regressee
```

The user reported that this now opens and allows project creation. That is user
evidence, not a diagnosed fix for the earlier startup failure. Do not add GPU
overrides when the plain launch works.

If startup reports `Unable to init GPU context:
Platform(Init(ERROR_INITIALIZATION_FAILED))`, GPUI/Blade failed Vulkan
initialization before displaying the app. That is a native startup blocker;
successful backend tests do not qualify the GUI or identify the driver cause.

The current build has a bounded, data-free backend diagnostic:

```sh
./target/debug/regressee --check-renderer
```

It initializes the same GPUI backend as the app, with filtered GPU logs, before
opening project storage or background jobs. Passing it does **not** verify
swapchain presentation, visible pixels or input. For logs during a normal launch:

```sh
REGRESSEE_GPU_DIAGNOSTICS=1 ./target/debug/regressee
```

Earlier process-only Mesa software-driver attempts also failed for the user with
`NoSupportedDeviceFound` on both Wayland and X11. Those commands are not a known
workaround and are no longer recommended as the next startup step. Neither
VRAM exhaustion nor a particular driver fault has been proved. Keep the original
errors rather than relabeling them as fixed.

The independent UI sweep also tried one private X11/software-rendered window:
it mapped, but its owned-client capture was entirely black. No input was sent
and no native visual pass was claimed. This separate test route is not the
user's working desktop and does not establish why either route behaves that way.

GPUI chooses Wayland when `WAYLAND_DISPLAY`
is nonempty, then X11 when `DISPLAY` is nonempty; there is no `ZED_DISPLAY`
backend-selection variable in the pinned version. `ZED_DEVICE_ID` accepts a
four-digit hexadecimal PCI device ID for hardware selection, not an adapter index.
Do not repeat identical failing launches. Keep the exact error from each attempted
configuration. A GPU/backtrace log is not the sanitized diagnostic export.

### Safe preview checks

Once a window opens, first test Create project → Save settings → View results,
keyboard focus and Settings' diagnostic preview/save cancellation. No website
navigation or browser installation is needed for these actions. To keep these
checks separate from your real data, set `REGRESSEE_DEV_DATA` to a fresh physical
private directory before launch; it must not have symlink ancestors.

The automated non-native suite can be run without opening a desktop window:

```sh
TMPDIR="$(realpath /path/to/private-tests/opencode)" \
  LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}" \
  cargo test --locked --workspace --all-features -- --test-threads=1
```

That TMPDIR is this development host's approved disk-backed scratch, not a portable
installation requirement. Deliberately ignored real-browser/scale tests do not
run with this command. Those require their explicit signed fixture inputs; see
`docs/phases/07-guardian-verification.md`. The ordinary suite is not human/native
rendering or public-site acceptance.

### Capture and storage problems

- **Needs login:** reopen Project setup's browser/login controls and confirm the
  correct environment. Resume requires the same frozen settings; changed releases
  or origins need a new run/login rather than reused credentials.
- **Could not capture:** inspect the selected target's error/warnings. Resolve
  access/network/readiness, then resume unfinished work or start fresh. No evidence
  of a failed page is a clean check.
- **Low disk:** pause, inspect History and the deletion impact preview, free space
  yourself, then resume. Active/shared references are protected. Runtime repair
  and interrupted-write cleanup are separate from retained screenshot history.
- **Browser setup unavailable:** Linux/Windows x86_64 now have an embedded default;
  an unsupported OS/architecture, expired catalog or invalid explicit development
  override can still block setup. macOS remains unsupported by this preview's
  process supervisor. You should not install Node/npm yourself. For a failed
  download, check internet access, free space and Settings' browser diagnostics,
  then retry. Damaged installations require explicit **Repair browser**. Repair
  never deletes baselines, disables TLS or turns off Chromium's sandbox.

Full-page capture has bounded height/readiness/image budgets; virtualized lists,
hidden controls, frames, canvas/video and complex image replacement are not
universally supported. Numerical presets are fixture-calibrated, not human accuracy.
See [capture/auth limitations](auth-and-capture.md).

## Sharing and diagnostics

A baseline ZIP contains original screenshots and addresses, including private
queries. Ignored rectangles **do not redact pixels**. Inspect its privacy preview
and share only with an authorized recipient. ZIPs exclude sessions and secret keys.

Settings → **Preview diagnostic contents** shows the exact structured JSON before
**I reviewed this — export locally…**. Cancel/Escape or cancelling the save picker
saves nothing. A diagnostic includes only numeric versions, typed runtime state,
OS/CPU family and bounded counts; no screenshots, URLs/queries, project names,
session references, secrets, local paths, private logs or free-form errors.
Nothing is uploaded. Review and share the local file only if you choose.

Keyboard: Tab/Shift-Tab, Enter/Space for focused buttons, Ctrl/Cmd+N create,
Ctrl/Cmd+S explicit save, Ctrl/Cmd+1…5 sections, Escape dismiss, Ctrl/Cmd+Q quit.
Native/high-DPI and screen-reader acceptance remain unqualified; focus-model tests
are not a screen-reader pass. Text fields support Shift+arrows/Home/End, mouse
selection and double-click selection; word handling is a simple fallback, not
locale-aware segmentation. The interface has no decorative motion.
