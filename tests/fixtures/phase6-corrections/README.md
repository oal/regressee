# Exact Phase 6 verifier specimens

Bounded copies of two synthetic, non-executable archive specimens from
`/tmp/opencode/regressee-p6-recovery-f013b87b4eca/hostile/`.
Do not regenerate or normalize these bytes. No history tree was copied.

- `long-inflate.zip`: SHA-256
  `b4f0f2404da5652c164662a19ce3d887c36637a0f76f255a50af08d20c732e0e`.
  Correctly hashed/CRC'd RGBA600×300 original inflates 301 rows.
- `unknown-coverage-field.zip`: SHA-256
  `1c14a76b93b5a7e8a41575a68018f33bf00c9310ba898a490654972f334de03e`.
  Unknown `inventory.coverage.Limited.script`; the string is not executed.

Storage regressions require specific image/schema causes, all-table and exact
physical-file preservation, and no partial CAS/staging/active publication. A
valid fixture is made in private test scratch by removing only the unknown field
from the second manifest; both actual Chromium originals remain unchanged.
