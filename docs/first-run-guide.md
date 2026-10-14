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
    Select a page to open its details and **Include page** / **Exclude page**
    controls. **Previous page** and **Next page** inspect other pages without
    changing inclusion. Pages blocked during discovery explain why; review project
    settings and find pages again rather than overriding website boundaries here.
5. Capture the reviewed pages, inspect coverage and warnings, then explicitly
   **Save baseline**.
    A baseline is the approved reference screenshots. Failed or missing targets
    cannot be called unchanged or silently approved.
    Saved capture screenshots are not automatically a baseline. If the capture
    has gaps, **Inspect capture** shows the screenshots on their own and explains
    why **Save baseline** is unavailable. Use **Filter** → **Gaps** to inspect
    failures, then **Find pages** to fix or exclude pages in a new preview and
    capture again. The earlier screenshots remain in History. Once the capture
    is complete, choose **Save baseline** to establish the comparison reference.
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

### Browser update or repair required

If **Find pages & preview**, login or capture reports **Browser update required**,
the installed private-browser components do not match this app build. Choose
**Open App settings** → **Update browser**, then return to your project and retry.
**Private browser needs repair** instead leads to **Repair browser**. Both actions
require your explicit choice; neither deletes saved screenshots or changes your
baseline. An update alone does not make an older baseline comparable: checks
still require its matching browser components.

Saving project settings is separate from visiting the website. If a save-and-find
operation reports that settings were saved before discovery failed, the saved
title and settings are kept; you do not need to enter them again. Unsaved drafts
are also preserved when opening recovery settings. Missing setup configuration
or an unsupported platform requires a supported build/platform, not repeated
download retries.

Other known failures explain their own next step: website login/access and
request limits lead to Project settings; local low-space failures lead to
History's Storage section when a project is selected. Nothing is installed,
visited, exported or deleted merely by opening a recovery screen.

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

- **URL count differs from screenshot count:** discovery lists included and
  excluded URLs. Only included URLs are scheduled, once per selected screen size.
  Off-site links, redirects, crawl rules, page limits and inspection failures can
  exclude URLs before capture; these are separate from failed screenshot attempts.
  Review the recorded page list for each exclusion reason. Refresh only reads
  saved results—it does not retry capture.
- **Page list was discarded:** **Discard discovered pages…** asks for confirmation
  before removing the unsaved list and include/exclude choices. Close/Escape only
  closes Page options. Recorded pages remain clickable with Previous/Next and
  read-only details. Choose **Use recorded pages** to prepare another capture
  without crawling again, even if no baseline has been saved, or **Find pages**
  to crawl again. Neither action automatically captures or saves a baseline.
- **Destructive actions:** warning dialogs explain what will change before page
  list discard/replacement, login removal, retained-baseline activation, permanent
  history deletion, temporary/recovery-file cleanup and browser repair/update.
  Cancel keeps the existing work. Baseline save/approval and unsaved-edit discard
  also retain their explicit confirmation dialogs. Quitting warns about losing
  discovered pages; Save and quit saves supported edits, not the discovery list.
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

### Retrying a failed browser setup on Windows

Use a newly built preview containing the setup-diagnostics change; older downloads
still show the generic error. Run as your normal user, with internet access. Do
not disable antivirus, TLS verification or the browser sandbox.

If setup fails, **Error details opens automatically**, including when setup was
started by Find pages or login. Keep a screenshot before returning to the
workspace. The details show the failed stage, elapsed seconds, received download
bytes, OS/CPU, app version and the available nested error causes. Official
download failures identify Node/Chromium and the source host; HTTP and filesystem
errors retain their status/OS codes. A browser-test failure distinguishes worker
startup, Chromium's test screenshot and shutdown, but may still provide only a
typed worker failure rather than Chromium's raw stack.

To help diagnose a retry, report the preview release tag, Windows version
(`winver`), the error details, and whether another attempt fails at the same
stage. Preparation points to configuration/storage checks; download to the
network/server; file installation to extraction or filesystem access; browser
startup test to the private worker/browser; saving the verified installation to
final file/pointer publication. These stages identify where to investigate, not
a confirmed cause. Check App settings for availability; use **Repair browser**
only if the installation needs explicit repair. A failed first-use setup can be
retried from the original action without deleting app data.

These are local on-screen details, not the structured diagnostic export described
below. Request URLs are omitted, but local paths/Windows account names can appear
in OS errors. Review screenshots before sharing. Nothing is uploaded, and the
privacy-filtered diagnostic export still excludes free-form errors and paths.

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

Search in the discovery preview and review queue filters as you type; there is
no separate Search submission. Clear restores the list. If your inspected
screenshot falls outside the filter, it remains visible with a notice so typing
does not discard ignored-area edits or replace evidence underneath you.

History separates Runs, Baselines and Storage, with direct selection and live date/type filtering of
loaded records. Its search scope is labelled; another history page can contain
additional matches. Storage totals are measured separately and are never shown
as zero just because measurement has not completed. Refresh history reloads the
current page; Refresh storage totals requests a separate census. Saved network settings load
automatically in App settings; saving edits is still explicit.

Review decisions and Save note are available without opening Analysis. Decisions
apply only to the displayed analysis; they never approve a baseline. Use Select
for baseline update → Review approval to preview reference changes separately.
Next unreviewed waits for your drafts to be saved. Hide page queue gives the
screenshots more room. Drag or scroll in an image to move around; Ctrl+scroll
(also Cmd+scroll on macOS) zooms around the pointer. Shift+scroll moves sideways.
The paired images move together. The **Zoom** menu groups **Fit width**, **Actual
size (100%)**, and percentage presets. The desktop control shows the fit mode and
resulting percentage; small windows show a compact Zoom label and percentage,
with the selected mode in the menu. Manually zooming switches to a fixed
percentage. Fit width resets the view and follows window resizing. With the
image focused, +/− zoom, arrows and
Page Up/Down move, 0 fits width, and 1 shows actual size. No navigation buttons
are shown for a missing screenshot.

Opening projects, reading screenshots and loading History show task-specific
loading messages, not empty-result messages. Short local reads use a compact
activity indicator instead of a dashed progress track. Capture feedback remains
visible, and operations with known totals show measured progress.
An empty screenshot/history state appears only after the corresponding read
finishes. Read failures remain distinct from an empty project and offer recovery.

Details shows the page, screen size, readable UTC run time, capture status and
recovery advice. Exact IDs and source records remain under Technical diagnostics.
Details and Analysis open a bounded flyout over the viewer, with Close/Escape;
they never replace the queue or change the underlying image size or position.
Only comparisons have Analysis; capture-only views do not show editing tools.

For keyboard ignored-area editing, open Analysis → Add by coordinates, edit the
whole-pixel coordinates, Apply area edit, then Save ignored areas. Invalid or
all-masked geometry cannot be saved. Original screenshots remain unchanged.
Draw ignored areas temporarily folds away the flyout so you can draw on either
original. Finishing a rectangle returns to the inspector; Done drawing or Escape
also leaves drawing mode. Area edits remain drafts until explicitly saved.
The queue, ordinary review actions and retained notes stay available.

In History, Open check results uses the recorded reference and saved analysis;
Open screenshots displays the selected capture. Both are inspection-only and
leave your active baseline unchanged. Return to latest evidence to do new work.
Quit/window close offers Cancel, Save and quit, or Discard and quit when drafts
exist. A failed save or unapplied area edit keeps the window open.

Keyboard: Tab/Shift-Tab, Enter/Space for focused buttons, Ctrl/Cmd+F find in the
review/discovery queue or History, Ctrl/Cmd+N create, Ctrl/Cmd+S explicit save,
Ctrl/Cmd+1…5 sections, Escape dismiss, Ctrl/Cmd+Q quit. F1 or App settings →
Keyboard shortcuts displays help. Go to next correction reveals an invalid
field without submitting your settings.
Native/high-DPI and screen-reader acceptance remain unqualified; focus-model tests
are not a screen-reader pass. Text fields support Shift+arrows/Home/End, mouse
selection and double-click selection; word handling is a simple fallback, not
locale-aware segmentation. The interface has no decorative motion.
