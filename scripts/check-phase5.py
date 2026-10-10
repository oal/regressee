#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial local checks. Uses only fresh signed literal-loopback fixtures."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
args.output.mkdir(parents=True, exist_ok=False)
output = args.output.resolve()
env = dict(os.environ, TMPDIR='/tmp/opencode', RUST_TEST_THREADS='1',
           CARGO_BUILD_JOBS='4', LIBRARY_PATH=str(root / '.runtime/link-libs'))
env.update(REGRESSEE_PHASE5_EVIDENCE=str(output / 'phase5-product'),
           REGRESSEE_PHASE5_DISTRIBUTION=str(output / 'signed-safety/product-flow/distribution.json'),
           REGRESSEE_PHASE5_RUNTIME_CACHE=str(output / 'signed-safety/product-flow/app/cache/product-runtime'))
commands = [
    ('locked-ui-build-test-clippy-fmt', ['bash', 'scripts/check-foundation.sh']),
    ('signed-auth-crawl-resource-safety', ['python3', 'scripts/check-phase4.py', '--output', str(output / 'signed-safety'), '--runtime-only']),
    ('signed-phase5-product', ['cargo', 'test', '--locked', '-p', 'regressee-app', '--features', 'ui-tests', '--test', 'product_flow', 'actual_phase5_product_analysis_composite_and_replacement', '--', '--exact', '--ignored', '--nocapture', '--test-threads=1']),
    ('readonly-independent-corpus', ['python3', '-B', 'tests/fixtures/analysis-proposal/corpus.py', 'validate']),
    ('final-format', ['cargo', 'fmt', '--all', '--', '--check']),
]
records = []
for name, command in commands:
    start = time.monotonic()
    with (output / f'{name}.log').open('wb') as log:
        result = subprocess.run(command, cwd=root, env=env, stdout=log,
                                stderr=subprocess.STDOUT, timeout=2400)
    records.append(dict(name=name, command=command, exit=result.returncode,
                        wall_seconds=time.monotonic()-start,
                        qualification='implementer local checks, not independent/native/human/public qualification'))
    (output / 'commands.json').write_text(json.dumps(records, indent=2))
    if result.returncode:
        raise SystemExit(result.returncode)
