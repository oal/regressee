#!/usr/bin/env python3
"""Locked/offline feasible feature matrix; no install, browser or network work.

Run from the repository root. Logs/results are preserved in a new private root.
Warnings are retained; clippy does not silently impose a new -D warnings policy.
Ignored OS/browser/runtime tests remain ignored, never relabelled PASS.
"""
import json
import hashlib
import os
from pathlib import Path
import subprocess
import signal
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PARENT = Path("/var/home/olav/Data/opencode/tmp/opencode")


def main():
    output = Path(tempfile.mkdtemp(prefix="regressee-phase7-checks-", dir=PARENT))
    sources = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
               for p in sorted((ROOT / "crates").rglob("*.rs"))}
    (output / "source-start.json").write_text(json.dumps(sources, indent=2))
    env = os.environ.copy()
    for key in list(env):
        if key.startswith("REGRESSEE_"):
            env.pop(key)
    env.update(LIBRARY_PATH=str(ROOT / ".runtime/link-libs"), TMPDIR=str(PARENT))
    commands = [
        ("workspace-all", ["cargo", "test", "--locked", "--offline", "--workspace", "--all-features"]),
        ("workspace-default", ["cargo", "test", "--locked", "--offline", "--workspace", "--no-default-features"]),
        ("production-ui", ["cargo", "check", "--locked", "--offline", "-p", "regressee-app", "--features", "ui", "--bin", "regressee"]),
        ("test-ui-only", ["cargo", "test", "--locked", "--offline", "-p", "regressee-app", "--features", "ui", "--bin", "regressee"]),
        ("test-ui-tests", ["cargo", "test", "--locked", "--offline", "-p", "regressee-app", "--features", "ui-tests", "--bin", "regressee"]),
        ("inspection-test-build", ["cargo", "test", "--locked", "--offline", "-p", "regressee-app", "--features", "ui-inspection", "--bin", "regressee", "--no-run"]),
        ("clippy-all", ["cargo", "clippy", "--locked", "--offline", "--workspace", "--all-features", "--all-targets"]),
        ("fmt", ["cargo", "fmt", "--all", "--", "--check"]),
        ("whitespace", ["git", "diff", "--check"]),
    ]
    results = []
    for name, command in commands:
        start = time.monotonic()
        with (output / (name + ".log")).open("wb") as log:
            try:
                process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                code = process.wait(timeout=900)
            except subprocess.TimeoutExpired:
                # Cargo's test child must not survive a qualification deadline.
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                code = "TIMEOUT"
        results.append({"name": name, "command": command, "exit": code, "seconds": time.monotonic() - start})
        (output / "results.json").write_text(json.dumps({"environment": {k:env[k] for k in ["LIBRARY_PATH","TMPDIR"]}, "results": results}, indent=2))
        print(name, code, flush=True)
    print(output, flush=True)
    raise SystemExit(any(r["exit"] != 0 for r in results))


if __name__ == "__main__":
    main()
