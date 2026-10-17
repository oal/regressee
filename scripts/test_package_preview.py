#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Offline tests of the Linux/Windows preview packages; no native SDK needed."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile


spec = importlib.util.spec_from_file_location("package_preview", Path(__file__).with_name("package-preview.py"))
preview = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preview)
REVISION = "a" * 40
BUILD_INFO = {"version": "0.1.0", "features": {
    "ui": True, "developer-tools": False, "test-support": False,
    "ui-inspection": False, "ui-renderer-capture": False,
}}


class PreviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / "output"
        for target, (_, executable) in preview.TARGETS.items():
            binary = self.root / "target" / target / "release" / executable
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"native binary fixture")
        (self.root / "crates/gui").mkdir(parents=True)
        (self.root / "crates/gui/Cargo.toml").write_text('[package]\nversion = "0.1.0"\n')
        for name in ("LICENSE", "THIRD_PARTY_NOTICES.md", "README.md"):
            (self.root / name).write_text(name)
        (self.root / "docs").mkdir()
        for name in preview.HELP_FILES:
            (self.root / "docs" / name).write_text(name)
        # Never collect the working tree or a local development/browser pack.
        (self.root / ".runtime").mkdir()
        (self.root / ".runtime/private.key").write_text("not for distribution")

    def build(self, target, revision=REVISION, build_info=None):
        inventory = {"target": target, "release_approved": False}
        texts = {"notices/fixture/LICENSE": (b"dependency license", 0o644)}
        metadata = BUILD_INFO if build_info is None else build_info
        with patch.object(preview, "command", side_effect=[REVISION, json.dumps(metadata), "rustc fixture"]), \
                patch.object(preview, "dependency_notices", return_value=(inventory, texts)):
            return preview.package(self.root, self.output, target, revision)

    def check_files(self, files, prefix, executable):
        info = json.loads(files[f"{prefix}/build-info.json"])
        self.assertEqual(info["source_commit"], REVISION)
        self.assertEqual(info["binary_sha256"], hashlib.sha256(b"native binary fixture").hexdigest())
        self.assertFalse(info["runtime_bundled"])
        self.assertFalse(info["signed"])
        self.assertEqual(info["compiled_features"], BUILD_INFO["features"])
        self.assertEqual(files[f"{prefix}/{executable}"], b"native binary fixture")
        self.assertEqual(files[f"{prefix}/notices/fixture/LICENSE"], b"dependency license")
        self.assertIn(f"{prefix}/LICENSE", files)
        for name in preview.HELP_FILES:
            self.assertEqual(files[f"{prefix}/help/{name}"], name.encode())
        self.assertFalse(any("private.key" in name or ".runtime" in name for name in files))
        self.assertFalse(any(b"not for distribution" in data for data in files.values()))

    def test_linux_archive_and_executable_mode(self):
        names = self.build("x86_64-unknown-linux-gnu")
        self.assertEqual(names, ["regressee-desktop-linux-x86_64.tar.gz"])
        with tarfile.open(self.output / names[0]) as archive:
            files = {m.name: archive.extractfile(m).read() for m in archive.getmembers()}
            self.assertEqual(archive.getmember("regressee-desktop-linux-x86_64/regressee-desktop").mode, 0o755)
        self.check_files(files, "regressee-desktop-linux-x86_64", "regressee-desktop")

    def test_windows_standalone_matches_zip_binary(self):
        names = self.build("x86_64-pc-windows-msvc")
        self.assertEqual(names, ["regressee-desktop-windows-x86_64.exe", "regressee-desktop-windows-x86_64.zip"])
        with zipfile.ZipFile(self.output / names[1]) as archive:
            files = {name: archive.read(name) for name in archive.namelist()}
        self.check_files(files, "regressee-desktop-windows-x86_64", "regressee-desktop.exe")
        self.assertEqual((self.output / names[0]).read_bytes(), files["regressee-desktop-windows-x86_64/regressee-desktop.exe"])

    def test_existing_output_is_not_overwritten(self):
        self.output.mkdir()
        marker = self.output / "previous-build"
        marker.write_bytes(b"keep")
        with self.assertRaises(FileExistsError):
            self.build("x86_64-pc-windows-msvc")
        self.assertEqual(marker.read_bytes(), b"keep")

    def test_revision_must_be_full_and_match_checkout(self):
        for revision in ("bad", "b" * 40):
            with self.subTest(revision=revision), self.assertRaises(ValueError):
                self.build("x86_64-unknown-linux-gnu", revision)
        self.assertFalse(self.output.exists())

    def test_empty_binary_is_rejected(self):
        (self.root / "target/x86_64-pc-windows-msvc/release/regressee-desktop.exe").write_bytes(b"")
        with self.assertRaises(ValueError):
            self.build("x86_64-pc-windows-msvc")
        self.assertFalse(self.output.exists())

    def test_developer_test_and_mismatched_builds_are_rejected(self):
        for feature in ("developer-tools", "test-support", "ui-inspection", "ui-renderer-capture"):
            info = {"version": "0.1.0", "features": {**BUILD_INFO["features"], feature: True}}
            with self.subTest(feature=feature), self.assertRaises(ValueError):
                self.build("x86_64-unknown-linux-gnu", build_info=info)
            self.assertFalse(self.output.exists())
        for info in ({}, {**BUILD_INFO, "version": "other"},
                     {**BUILD_INFO, "features": {**BUILD_INFO["features"], "ui": False}}):
            with self.subTest(info=info), self.assertRaises(ValueError):
                self.build("x86_64-unknown-linux-gnu", build_info=info)
        self.assertFalse(self.output.exists())

    def test_native_metadata_probe_is_bounded_and_isolated(self):
        inventory = {"target": "x86_64-unknown-linux-gnu"}
        with patch.object(preview, "command", side_effect=[REVISION, json.dumps(BUILD_INFO), "rustc fixture"]) as command, \
                patch.object(preview, "dependency_notices", return_value=(inventory, {})):
            preview.package(self.root, self.output, "x86_64-unknown-linux-gnu", REVISION)
        probe = command.call_args_list[1]
        self.assertEqual(probe.args[-1], "--build-info")
        self.assertEqual(probe.kwargs["timeout"], 15)
        isolated = Path(probe.kwargs["env"]["REGRESSEE_DEV_DATA"])
        self.assertFalse(isolated.exists(), "private metadata-probe scratch must be removed")
        self.assertNotEqual(isolated, self.root)

    def test_required_help_is_not_silently_omitted(self):
        for name in ("first-run-guide.md", "preview-release-notes.md"):
            with self.subTest(name=name):
                help_file = self.root / "docs" / name
                help_file.unlink()
                with self.assertRaises(FileNotFoundError):
                    self.build("x86_64-unknown-linux-gnu")
                self.assertFalse(self.output.exists())
                help_file.write_text(name)

    def test_target_specific_cached_and_vendored_notices(self):
        (self.root / "Cargo.lock").write_text('''
[[package]]
name = "gpui"
version = "0.2.2"
[[package]]
name = "example"
version = "1.0.0"
checksum = "fixture-checksum"
''')
        vendor = self.root / "vendor/gpui"
        vendor.mkdir(parents=True)
        (vendor / "Cargo.toml").write_text('[package]\nlicense = "Apache-2.0"\n')
        (vendor / "LICENSE-APACHE").write_text("GPUI license")
        cache = self.root / "cargo/registry/src/fixture/example-1.0.0"
        cache.mkdir(parents=True)
        (cache / "Cargo.toml").write_text('[package]\nlicense = "MIT"\nlicense-file = "legal/license.txt"\n')
        (cache / "legal").mkdir()
        (cache / "legal/license.txt").write_text("example license")
        (cache / "NOTICE").write_text("example notice")
        graph = "regressee-gui v0.1.0\ngpui v0.2.2 (/vendor/gpui)\nexample v1.0.0\nexample v1.0.0 (*)"
        with patch.dict(os.environ, {"CARGO_HOME": str(self.root / "cargo")}), \
                patch.object(preview, "command", return_value=graph) as command:
            inventory, texts = preview.dependency_notices(self.root, "x86_64-pc-windows-msvc")
        self.assertIn("x86_64-pc-windows-msvc", command.call_args.args)
        self.assertIn("normal,build", command.call_args.args)
        self.assertEqual(len(inventory["packages"]), 2)
        self.assertFalse(inventory["release_approved"])
        self.assertEqual(texts["notices/gpui-0.2.2/LICENSE-APACHE"][0], b"GPUI license")
        self.assertEqual(texts["notices/example-1.0.0/legal/license.txt"][0], b"example license")
        self.assertEqual(texts["notices/example-1.0.0/NOTICE"][0], b"example notice")

        # Missing texts stay explicit rather than being called reviewed/licensed.
        (cache / "NOTICE").unlink()
        (cache / "legal/license.txt").unlink()
        (cache / "Cargo.toml").write_text('[package]\nlicense = "MIT"\n')
        with patch.dict(os.environ, {"CARGO_HOME": str(self.root / "cargo")}), \
                patch.object(preview, "command", return_value=graph):
            inventory, _ = preview.dependency_notices(self.root, "x86_64-pc-windows-msvc")
        example = next(p for p in inventory["packages"] if p["name"] == "example")
        self.assertEqual(example["status"], "missing-license-text")

    def test_empty_dependency_graph_is_rejected(self):
        (self.root / "Cargo.lock").write_text("package = []\n")
        with patch.object(preview, "command", return_value=""), self.assertRaises(ValueError):
            preview.dependency_notices(self.root, "x86_64-unknown-linux-gnu")


if __name__ == "__main__":
    unittest.main()
