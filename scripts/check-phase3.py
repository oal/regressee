#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial actual product verification; sampled process RSS is not native UI latency."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--runtime-only', action='store_true')
parser.add_argument('--retained-flow', type=Path)
parser.add_argument('--active-cancel-only', action='store_true')
parser.add_argument('--policy-corrections-only', action='store_true')
parser.add_argument('--resource-corrections-only', action='store_true')
parser.add_argument('--scale-pages', type=int, default=100, choices=range(1, 1001))
args = parser.parse_args()
if args.resource_corrections_only and (args.policy_corrections_only or args.active_cancel_only or args.retained_flow):
    parser.error('resource corrections require a fresh signed pack and no other suite')
if args.policy_corrections_only and (args.active_cancel_only or args.retained_flow):
    parser.error('policy corrections require a fresh signed pack and no other suite')
args.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parent.parent
env = dict(os.environ, LIBRARY_PATH=str(root / '.runtime/link-libs'), RUST_TEST_THREADS='1', CARGO_BUILD_JOBS='4')
commands = []
if not args.runtime_only:
    commands += [('format', ['cargo', 'fmt', '--all', '--', '--check']), ('locked-ui-tests-clippy', ['bash', 'scripts/check-foundation.sh']), ('protocol', [str(root / '.runtime/installed/node/bin/node'), '--test', 'worker/product/test/protocol.test.mjs'])]
test_name = 'actual_phase3_resource_corrections' if args.resource_corrections_only else 'actual_phase3_policy_corrections' if args.policy_corrections_only else 'actual_phase3_active_cancel_and_custom_viewport' if args.active_cancel_only else 'actual_phase3_crawl_capture_resume_and_scale'
commands += [('actual-phase3', ['cargo', 'test', '--locked', '-p', 'regressee-app', '--features', 'test-support', '--test', 'product_flow', test_name, '--', '--ignored', '--nocapture', '--test-threads=1'])]
env['REGRESSEE_PHASE3_EVIDENCE'] = str(args.output.resolve() / 'product-flow')
env['REGRESSEE_PHASE3_SCALE_PAGES'] = str(args.scale_pages)
if args.retained_flow:
    env['REGRESSEE_PHASE3_RETAINED_FLOW'] = str(args.retained_flow.resolve())
reports = []
for name, command in commands:
    start = time.monotonic()
    peaks = {'Rust': 0, 'Node': 0, 'Chromium': 0}
    known = {}
    samples = 0
    with (args.output / f'{name}.log').open('wb') as log:
        process = subprocess.Popen(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
        while process.poll() is None:
            rows = {}
            for entry in Path('/proc').iterdir():
                if not entry.name.isdecimal():
                    continue
                try:
                    stat = (entry / 'stat').read_text()
                    comm = stat[stat.index('(') + 1:stat.rindex(')')]
                    fields = stat[stat.rindex(')') + 2:].split()
                    rows[int(entry.name)] = (int(fields[1]), fields[19], comm, int(fields[21]) * os.sysconf('SC_PAGE_SIZE'))
                except (OSError, ValueError, IndexError):
                    pass
            descendants = {process.pid}
            changed = True
            while changed:
                before = len(descendants)
                descendants.update(pid for pid, row in rows.items() if row[0] in descendants)
                changed = len(descendants) != before
            totals = dict.fromkeys(peaks, 0)
            for pid, row in rows.items():
                if pid in descendants or known.get(pid) == row[1]:
                    known[pid] = row[1]
                    group = 'Node' if row[2] == 'node' else 'Chromium' if any(s in row[2].lower() for s in ('chrome', 'chromium')) else 'Rust'
                    totals[group] += row[3]
            for group in peaks:
                peaks[group] = max(peaks[group], totals[group])
            samples += 1
            time.sleep(0.25)
        code = process.wait()
    reports.append({'name': name, 'command': command, 'exit': code, 'wall_seconds': time.monotonic() - start, 'sampled_peak_group_rss_bytes': peaks, 'samples': samples, 'sampling_interval_ms': 250, 'rss_note': 'Summed observed descendants and known start-time-matched processes; shared memory may be double-counted, short-lived/unobserved processes may be missed. Rust group includes Cargo. Not native input latency.'})
    (args.output / 'commands.json').write_text(json.dumps(reports, indent=2))
    if code:
        raise SystemExit(code)
