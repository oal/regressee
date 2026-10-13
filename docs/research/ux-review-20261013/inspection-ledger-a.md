# Assessment A — inspection ledger

Date: 2026-10-13. This ledger records the **actual fresh pixels personally inspected**, not merely captured files or a claim that every control was operated. Findings and limitations: [assessment-a.md](assessment-a.md).

Evidence root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j` (private, 0700). Numbers below are stable `number` values in `gallery-manifest.json`. Each row is two original PNGs. Resolve a filename as `<kind>-<width>x<height>-<state>.png` and metadata as the same stem plus `.json`. All 238 app PNGs were inspected in one bounded contact-sheet batch. **F** means an additional full-resolution inspection of that original; all other entries were inspected via contact sheet only. Full-resolution count: 20 (10 per size).

## Exact contact-sheet mapping

Every numbered capture below maps to one of these **personally inspected** sheets. Sheet paths are relative to the evidence root. The private `inspection-ledger-a.json` expands the mapping per PNG and records original/sheet hashes.

| Capture numbers | Inspected sheet | Position rule |
|---|---|---|
| 1–60 | `inspection-sheets/first-1280-01.png` through `first-1280-10.png` | Six images per sheet, source order; sheet = ceil(number/6). |
| 61–88 | `inspection-sheets/base-all-1280-11.png` through `base-all-1280-15.png` | Six per sheet; sheet = ceil(number/6), last has four. |
| 89–172 | `inspection-sheets/base-all-820-01.png` through `base-all-820-14.png` | Six per sheet; sheet = ceil((number−88)/6). |
| 173–176 | `contact-sheets/all-820-15.png` | First four cells; cells five/six are supplemental 188/189. |
| 177–178 | `contact-sheets/all-1280-15.png` | Cells five/six, following repeated base 85–88. |
| 179–184 | `contact-sheets/all-1280-16.png` | Cells one through six. |
| 185–187 | `contact-sheets/all-1280-17.png` | Cells one through three. |
| 188–189 | `contact-sheets/all-820-15.png` | Cells five/six. |
| 190–195 | `contact-sheets/all-820-16.png` | Cells one through six. |
| 196–198 | `contact-sheets/all-820-17.png` | Cells one through three. |
| 199–218 | `contact-sheets/task-1280-01.png` through `task-1280-04.png` | Six per sheet; sheet = ceil((number−198)/6), last has two. |
| 219–238 | `contact-sheets/task-820-01.png` through `task-820-04.png` | Six per sheet; sheet = ceil((number−218)/6), last has two. |

The existence of other generated sheets does not mean they were inspected. Repeated base images in supplemental sheets are not counted twice. Contact sheets preserve each source aspect ratio but reduce detail; full-resolution checks below support the specific fine-layout findings. No source PNG was edited.

## Complete 238-image inventory

### Base matrix — 88 states × 2 desktop sizes = 176 PNGs

| Kind · state | 1280×800 capture number | 820×600 capture number |
|---|---:|---:|
| all · `projects` | 1 | 89 |
| all · `projects-empty` | 2 | 90 |
| all · `projects-loading` | 3 | 91 |
| all · `projects-failed` | 4 | 92 |
| all · `project-chooser` | 5 | 93 |
| all · `new-website` | 6 **F** | 94 |
| all · `website` | 7 | 95 |
| all · `website-validation` | 8 | 96 |
| all · `access-public` | 9 **F** | 97 |
| all · `access-login` | 10 | 98 |
| all · `access-login-actions` | 11 | 99 |
| all · `access-saved-private` | 12 | 100 |
| all · `access-browser-open` | 13 | 101 |
| all · `access-ready` | 14 | 102 |
| all · `access-status-loading` | 15 | 103 |
| all · `access-invalid` | 16 | 104 |
| all · `access-options` | 17 **F** | 105 |
| all · `mapping-preview` | 18 | 106 |
| all · `timezone-popup` | 19 | 107 |
| all · `pages-sizes` | 20 | 108 |
| all · `pages-screen-sizes` | 21 | 109 |
| all · `pages-images` | 22 | 110 |
| all · `pages-speed` | 23 | 111 |
| all · `pages-advanced` | 24 | 112 |
| all · `pages-advanced-tail` | 25 | 113 |
| all · `pages-invalid` | 26 **F** | 114 |
| all · `pages-mobile-revealed` | 27 | 115 |
| all · `project-settings` | 28 | 116 |
| all · `project-settings-bottom` | 29 | 117 |
| all · `pages-baseline` | 30 | 118 |
| all · `discovery` | 31 | 119 |
| all · `discovery-options` | 32 | 120 |
| all · `discovery-no-matches` | 33 | 121 |
| all · `review` | 34 | 122 **F** |
| all · `review-mobile` | 35 | 123 **F** |
| all · `review-mixed-148` | 36 | 124 **F** |
| all · `review-failed-target` | 37 | 125 |
| all · `review-queue-hidden` | 38 | 126 **F** |
| all · `review-difference` | 39 | 127 |
| all · `review-overlay` | 40 | 128 |
| all · `review-heatmap` | 41 | 129 |
| all · `review-details` | 42 | 130 |
| all · `review-diagnostics` | 43 | 131 |
| all · `review-analysis` | 44 **F** | 132 |
| all · `review-analysis-advanced` | 45 | 133 |
| all · `review-ignored-area` | 46 | 134 |
| all · `review-note` | 47 **F** | 135 **F** |
| all · `review-filters` | 48 | 136 |
| all · `review-no-matches` | 49 | 137 |
| all · `review-approval-selection` | 50 | 138 |
| all · `review-approval-confirm` | 51 | 139 |
| all · `review-zoom-menu` | 52 | 140 |
| all · `review-full-width-before` | 53 | 141 **F** |
| all · `review-full-width-after` | 54 | 142 |
| all · `review-mode-chooser` | 55 **F** | 143 |
| all · `review-navigation-open` | 56 | 144 **F** |
| all · `review-approval-many` | 57 | 145 |
| all · `review-approval-last-exact` | 58 | 146 |
| all · `review-approval-last-row` | 59 | 147 |
| all · `review-approval-exact-tail` | 60 | 148 |
| all · `capture-gaps` | 61 | 149 |
| all · `capture-failed` | 62 | 150 |
| all · `capture-ready` | 63 | 151 |
| all · `capture-mobile` | 64 | 152 |
| all · `capture-saved` | 65 | 153 |
| all · `capture-empty` | 66 | 154 |
| all · `capture-loading` | 67 | 155 |
| all · `capture-read-failed` | 68 | 156 |
| all · `capture-read-dismissed` | 69 | 157 |
| all · `history-runs` | 70 | 158 |
| all · `history-bottom` | 71 **F** | 159 |
| all · `history-baselines` | 72 | 160 **F** |
| all · `history-storage` | 73 | 161 |
| all · `history-empty` | 74 | 162 |
| all · `history-no-matches` | 75 | 163 |
| all · `history-recorded-check` | 76 | 164 |
| all · `history-recorded-capture` | 77 | 165 |
| all · `history-provenance` | 78 | 166 **F** |
| all · `export-preview` | 79 | 167 |
| all · `import-preview` | 80 | 168 |
| all · `delete-preview` | 81 | 169 |
| all · `app-settings` | 82 **F** | 170 |
| all · `app-settings-network` | 83 | 171 |
| all · `app-settings-diagnostics` | 84 | 172 **F** |
| all · `diagnostic-preview` | 85 | 173 |
| all · `integrity-preview` | 86 | 174 |
| all · `keyboard-help` | 87 **F** | 175 |
| all · `about` | 88 | 176 |

### Supplemental inspectors — 11 states × 2 sizes = 22 PNGs

| Kind · state | 1280×800 capture number | 820×600 capture number |
|---|---:|---:|
| all · `review-diagnostics-tail` | 177 | 188 |
| all · `review-analysis-field-0` | 178 | 189 |
| all · `review-analysis-field-1` | 179 | 190 |
| all · `review-analysis-field-2` | 180 | 191 |
| all · `review-analysis-field-3` | 181 | 192 |
| all · `review-analysis-field-4` | 182 | 193 |
| all · `review-analysis-field-5` | 183 | 194 |
| all · `review-analysis-field-6` | 184 | 195 |
| all · `review-analysis-field-7` | 185 | 196 |
| all · `review-analysis-field-8` | 186 | 197 |
| all · `review-analysis-switches-tail` | 187 | 198 |

### Task states — 20 states × 2 sizes = 40 PNGs

| Kind · state | 1280×800 capture number | 820×600 capture number |
|---|---:|---:|
| task · `baseline-finishing` | 199 | 219 |
| task · `capture-attention` | 200 | 220 |
| task · `capture-complete` | 201 | 221 |
| task · `capture-gap-details` | 202 | 222 |
| task · `capture-paused` | 203 | 223 |
| task · `capture-pausing` | 204 | 224 |
| task · `capture-progress` | 205 | 225 |
| task · `capture-setup-details` | 206 | 226 |
| task · `discovery-browser-check` | 207 | 227 |
| task · `discovery-crawl-rules` | 208 | 228 |
| task · `discovery-first-page` | 209 | 229 |
| task · `discovery-first-retry` | 210 | 230 |
| task · `discovery-stopping` | 211 | 231 |
| task · `discovery` | 212 | 232 |
| task · `export-late-success` | 213 | 233 |
| task · `failed-task` | 214 | 234 |
| task · `partial-discovery` | 215 | 235 |
| task · `setup-download` | 216 | 236 |
| task · `zip-export-stopping` | 217 | 237 |
| task · `zip-import` | 218 | 238 |

## Renderer proof and exclusions

`renderer-proof.png` was personally inspected at full resolution before the matrix. `proof-reviewed.json` and `proof-review-note.md` record the font/radio/button/color/image calibration judgment. It is not included in the 238 app-state count. Four synthetic website fixture PNGs are also excluded. Production edits, real projects, external website visits, Assessment B and previous critiques/detector reports were excluded.

## Coverage qualifications carried with this inventory

- A captured state proves visible layout/paint, not real OS input, accessibility, screen-reader output, IME, motion or supported-platform readiness.
- Contact-sheet inspection does not establish fine text/contrast quality for every item. Only the marked originals were additionally inspected at full resolution.
- `pages-invalid`: error summary observed; invalid advanced field/reveal not visually established. `pages-mobile-revealed`: no actual reveal claim in the default batch. `pages-advanced-tail`: final custom actions/limitations below the fold not exhausted.
- Access options and History provenance do visibly disclose different content. Keyboard help invokes the real help-rendering path. Preliminary contrary suspicions were rejected after full-resolution/source inspection.
- Checkbox/radio alternatives were visible, not every value change/persisted result tested. Filter/group, permission override, custom profile, reanalysis, mask save/revalidation, network limitations and exact portability-detail outcomes were not all captured.
- Approval-many selection is seeded directly; no demonstrated bulk-select action or committed baseline change. History uses short lists/one baseline, not full retained/composite/last-record variants.
- Task states and website imagery are synthetic. No actual capture/login, network probe, import/export/delete or recovery operation was authorized against real projects.
