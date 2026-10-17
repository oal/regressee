#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial Phase 6 local checks; no independent/native/public qualification."""
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
env = dict(os.environ, RUST_TEST_THREADS='1',
           CARGO_BUILD_JOBS='4', LIBRARY_PATH=str(root / '.runtime/link-libs'))
for phase in (5, 6):
    env.update({f'REGRESSEE_PHASE{phase}_EVIDENCE': str(output / f'phase{phase}-product'),
                f'REGRESSEE_PHASE{phase}_DISTRIBUTION': str(output / 'signed-safety/product-flow/distribution.json'),
                f'REGRESSEE_PHASE{phase}_RUNTIME_CACHE': str(output / 'signed-safety/product-flow/app/cache/product-runtime')})
commands = [
    ('locked-ui-build-test-clippy-fmt', ['bash', 'scripts/check-foundation.sh']),
    ('signed-auth-crawl-resource-safety', ['python3', 'scripts/check-phase4.py', '--output', str(output / 'signed-safety'), '--runtime-only']),
    ('signed-phase5-product', ['cargo', 'test', '--locked', '-p', 'regressee-app', '--features', 'test-support', '--test', 'product_flow', 'actual_phase5_product_analysis_composite_and_replacement', '--', '--exact', '--ignored', '--nocapture', '--test-threads=1']),
    ('signed-phase6-recipient', ['cargo', 'test', '--locked', '-p', 'regressee-app', '--features', 'test-support', '--test', 'product_flow', 'actual_phase6_composite_zip_fresh_recipient_login_and_fixed_mapped_check', '--', '--exact', '--ignored', '--nocapture', '--test-threads=1']),
    ('readonly-independent-corpus', ['python3', '-B', 'tests/fixtures/analysis-proposal/corpus.py', 'validate']),
    ('final-format', ['cargo', 'fmt', '--all', '--', '--check']),
]
records = []
for name, command in commands:
    start = time.monotonic()
    with (output / f'{name}.log').open('xb') as log:
        try:
            result = subprocess.run(command, cwd=root, env=env, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=2400)
            code = result.returncode
        except subprocess.TimeoutExpired:
            code = 124
    records.append(dict(name=name, command=command, exit=code,
                        wall_seconds=time.monotonic() - start,
                        qualification='implementer local checks, not independent/native/human/public qualification'))
    (output / 'commands.json').write_text(json.dumps(records, indent=2))
    if code:
        raise SystemExit(code)
