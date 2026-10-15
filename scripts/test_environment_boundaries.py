#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Offline guards against development-environment leakage; no app/browser launch."""
import os
from pathlib import Path
import stat
import subprocess
import tempfile
import tomllib
import unittest
from unittest.mock import patch
import zipfile

from development_tools import private_directory, require_developer_binary

ROOT = Path(__file__).resolve().parent.parent
HOST_PATHS = (b"/var/home/", b"/home/developer/", b"/path/to/private-tests/")


class EnvironmentBoundaryTests(unittest.TestCase):
    def test_active_first_party_code_has_no_harness_or_personal_paths(self):
        files = [*ROOT.joinpath("crates").rglob("*.rs"),
                 *ROOT.joinpath("scripts").glob("*.py"),
                 *ROOT.joinpath("scripts").glob("*.sh"),
                 *ROOT.joinpath("worker").rglob("*.mjs")]
        for path in files:
            if path == Path(__file__).resolve() or "node_modules" in path.parts:
                continue
            data = path.read_bytes()
            with self.subTest(file=str(path.relative_to(ROOT))):
                self.assertNotIn(b"opencode", data.lower())
                for value in HOST_PATHS:
                    self.assertNotIn(value, data)

    def test_bundled_worker_has_no_personal_paths(self):
        # Third-party integrations may name other tools; their public names are
        # not a personal environment leak and are not rewritten here.
        with zipfile.ZipFile(ROOT / "crates/runtime/assets/official/bundled-worker.zip") as bundle:
            for name in bundle.namelist():
                data = bundle.read(name)
                for value in HOST_PATHS:
                    with self.subTest(file=name, value=value):
                        self.assertNotIn(value, data)

    def test_normal_ui_feature_does_not_enable_developer_tools(self):
        app = tomllib.loads((ROOT / "crates/app/Cargo.toml").read_text())["features"]
        def enabled(name):
            found = {name}
            for dependency in app.get(name, []):
                if dependency in app and dependency not in found:
                    found.update(enabled(dependency))
                else:
                    found.add(dependency)
            return found
        for feature in ("default", "ui", "test-support", "ui-tests"):
            self.assertNotIn("developer-tools", enabled(feature))
            self.assertNotIn("regressee-runtime/developer-tools", enabled(feature))
        for feature in ("ui-inspection", "ui-renderer-capture"):
            self.assertIn("developer-tools", enabled(feature))

    def test_private_directory_is_caller_selected_and_never_permission_repaired(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.assertEqual(private_directory(root), root)
            if os.name == "posix":
                root.chmod(0o755)
                with self.assertRaises(ValueError):
                    private_directory(root)
                self.assertEqual(stat.S_IMODE(root.stat().st_mode), 0o755)

    @unittest.skipUnless(os.name == "posix", "Unix symlink fixture")
    def test_linked_directory_is_not_canonicalized_into_trust(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            link = root / "link"
            link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError):
                private_directory(link)

    def test_native_probe_requires_explicit_developer_build(self):
        for enabled in (False, True):
            result = subprocess.CompletedProcess([], 0,
                stdout='{"features":{"developer-tools":' + str(enabled).lower() + '}}')
            with patch("development_tools.subprocess.run", return_value=result) as run:
                if enabled:
                    require_developer_binary(Path("binary"), Path("isolated-data"))
                else:
                    with self.assertRaises(ValueError):
                        require_developer_binary(Path("binary"), Path("isolated-data"))
                self.assertEqual(run.call_args.args[0], ["binary", "--build-info"])
                self.assertEqual(run.call_args.kwargs["env"]["REGRESSEE_DEV_DATA"], "isolated-data")


if __name__ == "__main__":
    unittest.main()
