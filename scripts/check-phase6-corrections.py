#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""One serial bounded command with retained evidence and owned-child settlement.

Use distinct output directories. This is local evidence, not qualification.
Linux only; no process-name/global cleanup, runtime download, or suite retries.
"""
import argparse
import ctypes
import fcntl
import json
import os
from pathlib import Path
import selectors
import signal
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--output', required=True, type=Path)
p.add_argument('--timeout', required=True, type=int)
p.add_argument('command', nargs=argparse.REMAINDER)
a = p.parse_args()
command = a.command[1:] if a.command[:1] == ['--'] else a.command
if not command or a.timeout <= 0:
    p.error('a positive deadline and command are required')
root = Path(__file__).resolve().parent.parent
a.output.mkdir(parents=True, exist_ok=False, mode=0o700)
out = a.output.resolve()
libc = ctypes.CDLL(None)
if libc.prctl(36, 1, 0, 0, 0) != 0:
    raise RuntimeError('owned Linux subreaper unavailable')

def ready(fd, seconds):
    with selectors.DefaultSelector() as s:
        s.register(fd, selectors.EVENT_READ)
        return bool(s.select(seconds))

def children():
    return [int(v) for v in Path(f'/proc/self/task/{os.getpid()}/children').read_text().split()]

def reap():
    records = []
    while True:
        try:
            pid, status = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            break
        if not pid:
            break
        records.append([pid, status])
    return records

env = dict(os.environ, TMPDIR='/tmp/opencode', RUST_TEST_THREADS='1',
           CARGO_BUILD_JOBS='4', LIBRARY_PATH=str(root / '.runtime/link-libs'))
start = time.monotonic()
with Path('/tmp/opencode/regressee-phase6-corrections.lock').open('a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    with (out / 'command.log').open('xb') as log:
        child = subprocess.Popen(command, cwd=root, env=env, stdout=log,
                                 stderr=subprocess.STDOUT, start_new_session=True)
        fd = os.pidfd_open(child.pid)
        (out / 'start.json').write_text(json.dumps(dict(pid=child.pid, command=command,
            stat=Path(f'/proc/{child.pid}/stat').read_text()), indent=2))
        expired = not ready(fd, a.timeout)
        if expired:
            signal.pidfd_send_signal(fd, signal.SIGTERM)
            if not ready(fd, 5):
                signal.pidfd_send_signal(fd, signal.SIGKILL)
        code = child.wait()
        os.close(fd)
    reaped = reap()
    forced = []
    # Adoption proves ownership even if Node/Chromium left their process group.
    # Bound each exact unreaped child by a pidfd, never by names or historical PIDs.
    until = time.monotonic() + 30
    while children() and time.monotonic() < until:
        for pid in children():
            try:
                fd = os.pidfd_open(pid)
            except ProcessLookupError:
                continue
            if not ready(fd, 0):
                forced.append(dict(pid=pid, command=Path(f'/proc/{pid}/cmdline').read_bytes()
                                   .replace(b'\0', b' ').decode(errors='replace')))
                signal.pidfd_send_signal(fd, signal.SIGTERM)
                if not ready(fd, 2):
                    signal.pidfd_send_signal(fd, signal.SIGKILL)
                    ready(fd, 2)
            os.close(fd)
        reaped.extend(reap())
    record = dict(command=command, exit=code, deadline_seconds=a.timeout, timeout=expired,
                  wall_seconds=time.monotonic() - start, reaped=reaped,
                  forced_children=forced, remaining_children=children(),
                  qualification='local bounded implementer evidence; independent reverification pending')
    (out / 'result.json').write_text(json.dumps(record, indent=2))
    print(json.dumps(record), flush=True)
    if record['remaining_children']:
        raise RuntimeError('owned children remain; do not hand off as settled')
    raise SystemExit(code if code > 0 else 1 if code or expired or forced else 0)
