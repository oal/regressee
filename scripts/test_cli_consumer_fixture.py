#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Owned loopback fixture tests; do not invoke a browser or actual application."""
import importlib.util
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from urllib.error import URLError
from urllib.request import Request, urlopen
from unittest.mock import patch, Mock

spec = importlib.util.spec_from_file_location("consumer", Path(__file__).with_name("cli-consumer-fixture.py"))
consumer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(consumer)


class ConsumerFixtureTests(unittest.TestCase):
    def test_stale_stopped_engine_is_absent_but_other_failures_are_not(self):
        for message, expected in (("Connection refused (os error 111)", True),
                                  ("The system cannot find the file specified. (os error 2)", True),
                                  ("version_mismatch: installed engine protocol differs", False),
                                  ("Permission denied (os error 13)", False)):
            result = subprocess.CompletedProcess([], 6, stdout=json.dumps({"code": "operation_failed", "data": {"message": message}}))
            self.assertEqual(consumer.absent_engine(result), expected)
        self.assertFalse(consumer.absent_engine(subprocess.CompletedProcess([], 4, stdout='{"code":"busy"}')))

    def test_actual_owned_site_mode_changes_and_authenticated_shutdown(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            consumer.write(root / "mode.json", {"mode": "baseline"})
            process = subprocess.Popen([sys.executable, str(Path(consumer.__file__).resolve()), "serve", "--root", str(root)],
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 5
                while not (root / "site.json").exists():
                    self.assertIsNone(process.poll())
                    self.assertLess(time.monotonic(), deadline)
                    time.sleep(0.02)
                site = json.loads((root / "site.json").read_text())
                with urlopen(site["url"] + "details", timeout=3) as response:
                    html = response.read()
                    self.assertIn(b"Stable baseline panel", html)
                    self.assertIn(b'id="long-page-evidence"', html)
                    self.assertIn(b"min-height:6000px", html)
                    self.assertIn(b"Below-fold baseline marker", html)
                    self.assertIn(b"top:4800px", html)
                    self.assertEqual(html.count(b'<a href='), 3)
                (root / "mode.json").write_text('{"mode":"changed"}')
                with urlopen(site["url"] + "details", timeout=3) as response:
                    data = response.read()
                    self.assertIn(b"Intentional revised layout", data)
                    self.assertIn(b'href="/new"', data)
                    self.assertIn(b"Below-fold revised marker", data)
                    self.assertEqual(data.count(b'<a href='), 4)
                (root / "mode.json").write_text('{"mode":"failed"}')
                with self.assertRaises((OSError, URLError)):
                    urlopen(site["url"] + "fail", timeout=3)
                with self.assertRaises(URLError) as denied:
                    urlopen(Request(site["url"] + "__stop", method="POST"), timeout=3)
                denied.exception.close()
                with urlopen(Request(site["url"] + "__stop", method="POST", headers={"X-Fixture-Token": site["token"]}), timeout=3):
                    pass
                self.assertEqual(process.wait(timeout=5), 0)
            finally:
                if process.poll() is None:
                    process.terminate()  # Only this test's newly spawned server.
                    process.wait(timeout=5)

    def test_setup_refuses_bad_or_uninventoried_package_without_starting_server(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory).resolve()
            package = parent / "package"
            package.mkdir()
            consumer.write(package / "build-info.json", {"executables": {"regressee": {"build": {"engine_protocol": 5}}}})
            with self.assertRaises(ValueError):
                consumer.setup(parent / "new", package)
            self.assertFalse((parent / "new").exists())

    def test_fresh_task_explicitly_reexercises_repaired_contracts(self):
        # No product/Chromium subprocess: validate generation from a checksum-bound
        # minimal synthetic package and a simulated owned server admission only.
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory).resolve()
            package = parent / "package"
            package.mkdir()
            cli = "regressee.exe" if consumer.os.name == "nt" else "regressee"
            data = b"SYNTHETIC CLI NOT EXECUTED"
            (package / cli).write_bytes(data)
            consumer.write(package / "build-info.json", {"executables": {
                cli: {"sha256": hashlib.sha256(data).hexdigest(), "build": {"engine_protocol": 6}}}})
            (package / "SHA256SUMS").write_text("".join(
                hashlib.sha256((package / name).read_bytes()).hexdigest() + "  " + name + "\n"
                for name in (cli, "build-info.json")))
            root = parent / "fresh-v2"

            def fake_server(*args, **kwargs):
                consumer.write(root / "site.json", {"url": "http://127.0.0.1:12345/", "pid": 0})
                return Mock(poll=lambda: None)

            with patch.object(consumer.subprocess, "Popen", side_effect=fake_server), patch("builtins.print"):
                consumer.setup(root, package)
            task = (root / "TASK.md").read_text()
            meta = json.loads((root / "exercise.json").read_bytes())
            self.assertEqual(meta["fixture_version"], 2)
            self.assertEqual(meta["long_page"]["minimum_css_height"], 6000)
            for required in ("standard YAML", "name/description", "omitted internal research",
                             "below-fold", "around y=5000", "--save failed-result.json",
                             "CompletedWithErrors", "jobs show/wait --save", "LOCAL ONLY",
                             "derived difference", "rendered layout unqualified",
                             "do not accept Home or the failed target"):
                self.assertIn(required, task)
            self.assertFalse((root / "cache/product-runtime").exists())


if __name__ == "__main__":
    unittest.main()
