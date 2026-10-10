# Phase 5 — exact fixture-calibrated policies (v1)

Algorithm: `aligned-srgb-d65-ciede2000-rows-v1`, schema 1. The independent
`analysis-proposal` corpus and seed-policy JSON remain unchanged historical
research. Phase 5 selects those numeric seeds as product v1 presets after exact
fixture calibration; it does **not** claim proven user accuracy, defect detection,
or measured real-site false-positive/negative rates.

| Product revision | ΔE00 tolerance | Global fraction | Local fraction | Local changed minimum | Connected members minimum |
|---|---:|---:|---:|---:|---:|
| `fixture-calibrated-sensitive-v1` | 1 | 0.0005 (0.05%) | 0.025 (2.5%) | 4 | 8 |
| `fixture-calibrated-balanced-v1` | 2 | 0.002 (0.2%) | 0.05 (5%) | 8 | 16 |
| `fixture-calibrated-relaxed-v1` | 4 | 0.02 (2%) | 0.1 (10%) | 16 | 64 |

All presets: AA off; 32-pixel square local windows, stride 16, minimum 64
eligible pixels; exact four-neighbor connected members (no dilation); any nonzero
height delta triggers review; ignored coverage warning at ≥50%. Significant
opaque pixels use strictly `ΔE00 > tolerance`. Global/local/count/region gates
use inclusive `>=`; at least one significant member is required for a global
trigger. Criteria combine with OR, after comparability gates. A blank advanced
global/local/region field disables that criterion. Advanced immutable revisions
are content-hashed and retain every parameter, including optional AA/height rules.

## Annotation and fixture calibration

`tests/fixtures/analysis-proposal/manifest.json` independently annotates all 13
geometries/raw/mask/component oracles. The retained engine's exact tests assert
all 52 control/seed analyses and all 39 seed interpretations. Product integration
tests additionally run all 39 product policies and compare complete reports with
the corresponding annotated seed semantics (only the policy revision changes).

- Unchanged and +1 gray noise: no significant change under all presets; raw
  noise is still retained (6,144 members for the gray drift).
- Tiny 8×8 icon at `(30,1020)` on a 96×2048 page: exactly 64 / 196,608;
  all presets flag its connected region despite no global trigger. Qualifying
  local windows: 4 / 1 / 0. This protects a small coherent long-page change;
  it is not semantic icon recognition.
- Scattered high-contrast noise: 192 area-1 components; Sensitive/Balanced
  trigger global change, Relaxed does not. This is a deliberate tradeoff, not
  a demonstrated user preference. Coherent blocks trigger all presets.
- Middle insertion: 3,072 aligned changes and three components; added/removed
  background-colored height bands retain 1,536 structural pixels. No warping or
  white-padding equality can erase a missing band.
- Overlapping masks: exact 448-pixel union; 16 / 5,696 changes remain. Near-total
  mask: 96 / 96, with 6,048 / 6,144 ignored and a warning. All-area masking:
  undefined 0/0 and NotComparable, **never a clean result**.
- Unexpected width/DPR/material differences gate ordinary comparison. Rendering
  mismatch is inspection caution and blocks selective approval. Site URL mapping
  alone is neither a material nor rendering mismatch.

## Limits, not promises

Samples are generated arithmetic/color fixtures, plus owned browser-generated
layout/photo/color controls. Real font AA, licensed/consented photographic
diversity, DPR/platform/font variation, maximum 64-million-pixel CPU/RSS and
non-developer accuracy/usability calibration remain unqualified. Optional AA is
only a stationary horizontal blend heuristic and can hide a real blend-color
edit; it is not a proven font-noise classifier. sRGB sample interpretation is
not ICC conversion. Narrow complete PNG/DEFLATE validation, alpha gates, bounds,
coordinate unions and cancellation retain the independently audited contract.

Normal and replacement images cannot be toggled into equivalence after capture.
`neutral-raster-v2` changes source attributes after readiness, preserves measured
light-DOM rectangles only when actually measured/equal, retains vector icons and
does not support canvas/video/frames/shadow DOM/pseudo-elements/complex or
extensionless backgrounds/source-dependent behavior. Replacement-only runs do
not separately save original hidden image content. Readiness/media/source and
geometry warnings remain visible, even if screenshot pixels repeat.
An exact known HTML image URL whose reload starts during source replacement and
ends in Chromium `net::ERR_ABORTED` is counted separately as a canceled source
reload in replacement metadata/UI, not a completed resource. Earlier readiness
failures, HTTP errors, scope denials and other resource failures are not suppressed.
