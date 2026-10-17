# First run and safe review

Regressee is a public beta. The default app-managed browser uses
app-pinned official downloads; internet is needed once for setup, with no Node/npm
or terminal configuration. Historical development distributions remain restricted
to authorized literal-loopback fixtures. See [support matrix](support-matrix.md)
for the separate platform/clean-machine qualification limits.

1. **Create project**, enter a recognizable name and your test website
   address(es), then **Save project & continue**. This saves settings locally without
   visiting the site. Approve all listed websites when using multiple websites.
   Visits themselves can have effects: use a read-only test account, never an
   administrative production account.
2. In **Website access**, choose **No login required** or **Login required**.
   For login, **Save settings** first, then use **Open login browser**, finish
   login/consent yourself, and choose **Confirm login** to check that it works in a
   separate session. An unavailable secure store
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
4. Use **Find pages**, then inspect the included/excluded page list in
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
    Saved capture screenshots are not automatically a baseline. If a finished
    capture has some failures, choose **Save baseline** to save the successful
    screenshots. A confirmation names how many screenshots will be saved and
    skipped. Skipped page/screen-size pairs are not included in future checks;
    their failure records remain in History. **Review failures** filters the
    results without changing the screenshot being inspected. To include missing
    screenshots later, use **Pages & baseline** → **Use recorded pages** (or
    **Find pages** for a new list), capture again, and save a replacement baseline.
    No-success, unfinished and preview-only captures cannot be saved as baselines.
6. **Review changes** → **Run check** revisits only the fixed baseline
   inventory. Use the coverage summary and searchable queue beside the comparison
   viewer to inspect changed screenshots and failed/non-comparable targets. Differences are evidence
   to review, not automatic bug verdicts. Other computers/fonts can change pixels.
7. Sensitivity and ignored rectangles change analysis only. Select compatible
   screenshots, inspect the approval consequences, then explicitly confirm the
   new baseline version. Other references are inherited, and old versions remain
   until deliberate manual deletion. **History** distinguishes baseline versions
   from the capture/check runs that supplied their evidence.

## Comparing screenshots with the split slider

In the screenshot viewer, choose **View → Split slider**. Before is on the left
and After is on the right; drag the vertical divider to reveal either original
at the same zoom and position. Drag elsewhere on the image, or scroll, to pan;
Ctrl/Command + scroll zooms at the pointer. **Compare** remains the default view,
and all existing viewing modes remain available.

For keyboard control, Tab from the image to the divider. Left/Right moves it by
1%; Shift + Left/Right moves it by 10%. Home shows all After; End shows all Before.
The slider needs both screenshots. If one is unavailable, the viewer uses
separate panes instead. Changing the view never saves or updates a baseline.

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

If **Find pages**, login or capture reports **Browser update required**,
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

### Linux browser setup mentions `/tmp/opencode`

Earlier previews mistakenly preferred a development tool's temporary directory.
If setup reports `worker scratch ancestor has unsafe ownership/permissions:
/tmp/opencode`, use a build with that preference removed. Do not change that
directory's permissions or disable browser security.

For an affected build, close Regressee and launch the extracted Linux executable
with an explicit Regressee-owned namespace, then retry browser setup in App settings:

```sh
REGRESSEE_WORKER_SCRATCH_PARENT="/tmp/rg2-$(id -u)" ./regressee-desktop
```

The app creates the final directory with mode 0700 if missing and validates its
ownership and ancestors. This override applies only to that launch and does not
change project or screenshot storage. Windows uses a different scratch mechanism;
similar setup failures there need their own error details.

### Linux GPU initialization failure

Start with a plain launch:

```sh
./regressee-desktop
```

Do not add GPU overrides when the plain launch works.

If startup reports `Unable to init GPU context:
Platform(Init(ERROR_INITIALIZATION_FAILED))`, GPUI/Blade failed Vulkan
initialization before displaying the app. Check that your system has a working
graphics driver and meets the [platform requirements](support-matrix.md).

The current build has a bounded, data-free backend diagnostic:

```sh
./regressee-desktop --check-renderer
```

It initializes the same GPUI backend as the app, with filtered GPU logs, before
opening project storage or background jobs. Passing it does **not** verify
swapchain presentation, visible pixels or input. For logs during a normal launch:

```sh
REGRESSEE_GPU_DIAGNOSTICS=1 ./regressee-desktop
```

GPUI chooses Wayland when `WAYLAND_DISPLAY`
is nonempty, then X11 when `DISPLAY` is nonempty; there is no `ZED_DISPLAY`
backend-selection variable in the pinned version. `ZED_DEVICE_ID` accepts a
four-digit hexadecimal PCI device ID for hardware selection, not an adapter index.
Do not repeat identical failing launches. Keep the exact error from each attempted
configuration. A GPU/backtrace log is not the sanitized diagnostic export.

### Safe preview checks

Once a window opens, first test Create project → Save settings → View results,
keyboard focus and Settings' diagnostic preview/save cancellation. No website
navigation or browser installation is needed for these actions. Use a new test
project rather than changing an existing baseline.

Source-build isolation and automated test instructions are in the
[build guide](release-building.md#developer-only-tools). Downloaded previews do not
enable developer storage overrides or private capture tracing.

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

Comparisons open with changed results only. **Show all** restores the other
results without changing screenshots or review drafts; search still applies.
The summary always includes missing/not-compared results. **No changed results**
does not mean every capture succeeded. Capture-only runs show all results.

History separates Runs, Baselines and Storage, with direct selection and live date/type filtering of
loaded records. Its search scope is labelled; another history page can contain
additional matches. Storage totals are measured separately and are never shown
as zero just because measurement has not completed. Refresh history reloads the
current page; Refresh storage totals requests a separate census. Saved network settings load
automatically in App settings; saving edits is still explicit.

**App settings → Appearance** controls Regressee's light/dark theme. **System**
is the default and follows desktop appearance changes automatically. Choose
**Light** or **Dark** to override it; the change applies immediately and is
remembered on this computer. Choose **System** again to resume desktop following.
This never recolors saved screenshots or changes the website-theme setting used
for captures.

No significant change means the visual check needs no manual review, using the
saved sensitivity and ignored areas; it does not necessarily mean identical
pixels. These results have no disposition buttons and Next unreviewed skips them.
Loading cautions remain separate from the pixel verdict. Notes are optional.
Review decisions for changed results and Save note are available without opening
Analysis. Decisions apply only to the displayed analysis; they never approve a baseline.
To use an After screenshot as the reference for future checks, choose **Select After
screenshot** in **Update baseline**. Select any other page and screen size you want
to include, then choose **Review update…**. Nothing changes until you choose
**Save baseline update** in the confirmation. Other baseline screenshots stay
unchanged, and earlier versions remain in History.
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
