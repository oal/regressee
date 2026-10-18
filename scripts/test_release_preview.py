#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Production feature/source-binding wrapper tests; builds are mocked, not qualified."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release_preview", Path(__file__).with_name("release-preview.py"))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)
TARGET = "x86_64-unknown-linux-gnu"


class ReleasePreviewTests(unittest.TestCase):
    def test_source_policy_gate_supplies_its_required_report_argument(self):
        workflow = (Path(__file__).resolve().parent.parent / ".github/workflows/release-preview.yml").read_text()
        commands = [line.strip().removeprefix("run: ") for line in workflow.splitlines()
                    if "run: python -B scripts/test_source_release.py" in line]
        self.assertEqual(commands, [
            'python -B scripts/test_source_release.py --report "$RUNNER_TEMP/source-release-tests.json"'
        ])

    def test_each_product_builds_only_production_packages_and_binds_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary_dir = root / "target" / TARGET / "release"
            binary_dir.mkdir(parents=True)
            for name in ("regressee", "regressee-desktop"):
                (binary_dir / name).write_bytes(name.encode())
            for product in ("cli", "desktop", "all"):
                with patch.dict(release.os.environ, {"CARGO_TARGET_DIR": str(root / "target")}), \
                        patch.object(release.preview, "command", side_effect=["a" * 40, "", "rustc fixture"]), \
                        patch.object(release.preview, "selected_sources", return_value={"source.rs": (b"frozen", 0o644)}), \
                        patch.object(release.subprocess, "run") as build, \
                        patch.object(release.preview, "package", return_value=["archive"]) as package:
                    release.build(root, root / "new", TARGET, product, offline=True)
                commands = [call.args[0] for call in build.call_args_list]
                for args in commands:
                    self.assertIn("--release", args)
                    self.assertIn("--locked", args)
                    self.assertIn("--offline", args)
                    self.assertNotIn("--all-features", args)
                    self.assertNotIn("test-support", " ".join(args))
                self.assertNotIn("--features", commands[0])
                self.assertNotIn("regressee-gui", commands[0])
                self.assertEqual(len(commands), 1 if product == "cli" else 2)
                if product != "cli":
                    self.assertIn("ui", commands[1])
                    self.assertNotIn("regressee-cli", commands[1])
                receipt = package.call_args.args[-1]
                self.assertEqual(receipt["commands"], commands)
                self.assertEqual(set(receipt["binaries"]), {"regressee"} if product == "cli" else {"regressee", "regressee-desktop"})
                self.assertFalse(receipt["clean_machine_qualified"] or receipt["signed_attestation"])

    def test_dirty_or_changed_sources_refuse_before_packaging(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            with patch.object(release.preview, "command", side_effect=["a" * 40, " M source.rs"]), \
                    patch.object(release.subprocess, "run") as build, self.assertRaises(ValueError):
                release.build(root, root / "new", TARGET)
            build.assert_not_called()
            with patch.object(release.preview, "command", side_effect=["a" * 40, ""]), \
                    patch.object(release.preview, "selected_sources", side_effect=[{"s": (b"before", 0o644)}, {"s": (b"after", 0o644)}]), \
                    patch.object(release.subprocess, "run"), patch.object(release.preview, "package") as package, self.assertRaises(ValueError):
                release.build(root, root / "new", TARGET)
            package.assert_not_called()


if __name__ == "__main__":
    unittest.main()
