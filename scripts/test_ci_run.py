#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Offline launcher contracts; native Windows ownership remains a hosted gate."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("ci_run", Path(__file__).with_name("ci-run.py"))
launcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(launcher)


class LauncherTests(unittest.TestCase):
    def test_missing_command_refuses(self):
        with patch.object(sys, "argv", ["ci-run.py"]), self.assertRaises(SystemExit):
            launcher.main()

    def test_preserves_arguments_and_child_failure(self):
        arguments = ["cargo", "test", "--locked", "--", "argument with spaces"]
        with patch.object(sys, "argv", ["ci-run.py", *arguments]), \
                patch.object(launcher.os, "name", "posix"), \
                patch.object(launcher.subprocess, "call", return_value=17) as child, \
                patch.object(launcher, "windows_user_owner") as owner:
            self.assertEqual(launcher.main(), 17)
        child.assert_called_once_with(arguments)
        owner.assert_not_called()

    def test_windows_owner_is_set_before_child(self):
        calls = []
        with patch.object(sys, "argv", ["ci-run.py", "cargo", "test"]), \
                patch.object(launcher.os, "name", "nt"), \
                patch.object(launcher, "windows_user_owner", side_effect=lambda: calls.append("owner")), \
                patch.object(launcher.subprocess, "call", side_effect=lambda _: calls.append("child") or 0):
            self.assertEqual(launcher.main(), 0)
        self.assertEqual(calls, ["owner", "child"])

    def test_windows_owner_failure_never_spawns_child(self):
        with patch.object(sys, "argv", ["ci-run.py", "cargo", "test"]), \
                patch.object(launcher.os, "name", "nt"), \
                patch.object(launcher, "windows_user_owner", side_effect=RuntimeError("refused")), \
                patch.object(launcher.subprocess, "call") as child, \
                self.assertRaises(RuntimeError):
            launcher.main()
        child.assert_not_called()

    def test_token_changes_refuse_outside_hosted_ci(self):
        with patch.dict(launcher.os.environ, {"GITHUB_ACTIONS": "false"}), \
                self.assertRaisesRegex(RuntimeError, "restricted to hosted CI"):
            launcher.windows_user_owner()

    @unittest.skipIf(sys.platform == "win32", "real Windows token change is hosted CI only")
    def test_real_child_exit_status(self):
        result = subprocess.run([
            sys.executable, "-B", str(Path(__file__).with_name("ci-run.py")),
            sys.executable, "-c", "raise SystemExit(23)",
        ], check=False, timeout=10)
        self.assertEqual(result.returncode, 23)


if __name__ == "__main__":
    unittest.main()
