# Synthetic analysis proposal corpus

**Not screenshots of a website. Not calibrated perceptual results.** Original
algorithmic color blocks, MPL-2.0; see `LICENSE` and the repository `LICENSE`.
`corpus.py` is the preferred source for all PNGs. No imported images, fonts,
credentials, third-party packages, network access, or services are involved.

- `images/`: 13 before/after pairs; opaque RGB8, complete non-interlaced PNGs.
- `manifest.json`: dimensions, SHA-256s, literal exact-count oracles, mask
  rectangles, mathematical derivations, and intended evidence annotations.
- `validation.json`: retained independent decoder/count-validator results.
- `seed-policies.json`: **unimplemented** starting proposals, separate from
  exact raw measurements. Not compatible with current Rust policy serialization.
- `../../../docs/research/analysis-corpus.md`: proposal and Phase 5 handoff.

From the repository root, validate without changing any files:

```sh
python3 -B tests/fixtures/analysis-proposal/corpus.py validate
```

To retain another validation report, use a **new** filename inside this directory:

```sh
python3 -B tests/fixtures/analysis-proposal/corpus.py validate \
  --report tests/fixtures/analysis-proposal/independent-validation.json
```

`generate` refuses to overwrite `images/` or `manifest.json`. To reproduce,
copy `corpus.py` and `LICENSE` into a fresh, reviewer-owned directory and run
`python3 -B /path/to/fresh/directory/corpus.py generate`, then `validate` there.
Do not delete or replace this checked-in corpus to regenerate it. Stored DEFLATE
blocks make PNG bytes deterministic independently of compression-library tuning.

The decoder separately verifies complete chunk sequencing, every CRC, bounded
dimensions/inflation, the zlib checksum and EOF, row lengths/filter bytes,
SHA-256, image-file inventory, and exact aligned-coordinate mathematical counts.
Oracles are literals derived from geometry; the measurement loop does not
regenerate expected images. Raw 4-connected components are also checked, but
are **not** perceptual regions. The helper is deliberately limited to this
RGB8/filter-0 corpus, not a general PNG decoder or production comparison engine.

The width-mismatch case contains illustrative coordinate-union counts only;
automatic comparison must be blocked by the profile-integrity gate. An all-masked
case has undefined `0/0`, not a clean result. No masks here span a dimension
change; mask geometry revalidation remains a separate Phase 5 requirement.
