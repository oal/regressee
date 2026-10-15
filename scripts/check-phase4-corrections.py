#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial bounded P4 correction checks; own descendants only, no host cleanup."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--output', required=True, type=Path)
args = p.parse_args()
root = Path(__file__).resolve().parent.parent
args.output.mkdir(parents=True, exist_ok=False)
if os.name == 'posix' and Path('/proc').exists():
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
        raise RuntimeError('owned child subreaper setup failed')
protected = [root/'crates/app/src/analysis_engine.rs', *sorted((root/'docs/research').glob('*analysis*')), root/'docs/phases/04-verification.md']
hashes = {str(f.relative_to(root)): hashlib.sha256(f.read_bytes()).hexdigest() for f in protected if f.is_file()}
(args.output/'protected-entry.json').write_text(json.dumps(hashes, indent=2))
env = dict(os.environ, LIBRARY_PATH=str(root/'.runtime/link-libs'), RUST_TEST_THREADS='1', CARGO_BUILD_JOBS='4')
commands = [
    ('focused-binding', ['cargo', 'test', '--locked', '-p', 'regressee-app', '--features', 'ui-tests', '--lib', 'auth::tests::', '--', '--nocapture', '--test-threads=1']),
    ('locked-node-browser', ['python3', 'scripts/check-phase4.py', '--output', str(args.output.resolve()/'handoff')]),
]
records = []
exit_code = 0
for name, command in commands:
    start = time.monotonic()
    with (args.output/f'{name}.log').open('wb') as log:
        result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=1800)
    records.append(dict(name=name, command=command, exit=result.returncode, wall_seconds=time.monotonic()-start))
    (args.output/'commands.json').write_text(json.dumps(records, indent=2))
    if result.returncode:
        exit_code = result.returncode
        break
reaped = []
deadline = time.monotonic()+10
while time.monotonic() < deadline:
    try:
        pid, status = os.waitpid(-1, os.WNOHANG)
    except ChildProcessError:
        break
    if pid:
        reaped.append(dict(pid=pid, status=status))
    else:
        time.sleep(.05)
children = []
for proc in Path('/proc').glob('[0-9]*'):
    try:
        status = (proc/'status').read_text()
        if int(next(l for l in status.splitlines() if l.startswith('PPid:')).split()[1]) == os.getpid():
            children.append(int(proc.name))
    except (FileNotFoundError, ProcessLookupError, PermissionError, StopIteration):
        pass
changed = [f for f, h in hashes.items() if hashlib.sha256((root/f).read_bytes()).hexdigest() != h]
(args.output/'ownership.json').write_text(json.dumps(dict(reaped_own_descendants=reaped, remaining_own_children=children, protected_changes=changed, qualification='implementer local checks; no native/human/keychain/independent verdict'), indent=2))
print(json.dumps(dict(output=str(args.output), commands=records, remaining_own_children=children, protected_changes=changed)))
raise SystemExit(exit_code or (1 if children or changed else 0))
