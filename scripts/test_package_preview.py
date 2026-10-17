#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Offline package/source/feature refusal tests; no native SDK or release upload."""
import copy
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from test_source_release import fixture, put
from skill_frontmatter import validate_skill

spec = importlib.util.spec_from_file_location("package_preview", Path(__file__).with_name("package-preview.py"))
preview = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preview)
REVISION = "a" * 40
TARGET = "x86_64-unknown-linux-gnu"


def metadata(name):
    return {"build_info_schema": 1, "executable": name, "version": "0.1.0",
            "engine_protocol": 6, "cli_schema": 1,
            "features": {**preview.FEATURES, "ui": name == "regressee-desktop"}}


def inventory(target, product):
    path = "notices/fixture-1.0/LICENSE"
    data = b"dependency license"
    return {"target": target, "product": product,
            "packages_built": ["regressee-cli"] + (["regressee-gui"] if product == "desktop" else []),
            "features": ["regressee-gui/ui"] if product == "desktop" else [],
            "edges": ["normal", "build"], "release_approved": False, "native_inventory_complete": False,
            "packages": [{"name": "fixture", "version": "1.0", "status": "unreviewed",
                          "texts": [{"path": path, "sha256": hashlib.sha256(data).hexdigest()}]}]}, {path: (data, 0o644)}


class PreviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.parent = Path(self.temp.name).resolve()
        self.root = self.parent / "tree"
        self.root.mkdir()
        fixture(self.root)
        self.output = self.parent / "output"
        for name in ("cli", "gui"):
            put(self.root, f"crates/{name}/Cargo.toml", f'[package]\nname = "regressee-{name}"\nversion = "0.1.0"\n')
            put(self.root, f"crates/{name}/src/main.rs", "fn main() {}\n")
        put(self.root, "crates/app/src/protocol.rs", "pub const VERSION: u32 = 6;\n")
        for target, (_, suffix) in preview.TARGETS.items():
            for name in ("regressee", "regressee-desktop"):
                put(self.root, f"target/{target}/release/{name}{suffix}", (name + " binary fixture").encode())
        put(self.root, ".runtime/private.key", "not for distribution")
        put(self.root, "docs/agent-cli-phase4-review.md", "private internal report sentinel")
        self.infos = {name: metadata(name) for name in ("regressee", "regressee-desktop")}
        self.dirty = ""

    def command(self, root, *args, **kwargs):
        if args[:3] == ("git", "rev-parse", "HEAD"):
            return REVISION
        if args[:2] == ("git", "status"):
            return self.dirty
        if args[:2] == ("git", "show"):
            return "1234567890"
        if args[0] == "rustc":
            return "rustc fixture"
        if args[-1] == "--build-info":
            return json.dumps(self.infos[Path(args[0]).name.removesuffix(".exe")])
        self.fail(f"Unexpected command {args}")

    def build(self, target=TARGET, product="all", **kwargs):
        with patch.object(preview, "command", side_effect=self.command), \
                patch.object(preview, "dependency_notices", side_effect=lambda r, t, p: inventory(t, p)):
            return preview.package(self.root, self.output, target, REVISION, product, **kwargs)

    def files(self, product="cli", target=TARGET):
        platform, suffix = preview.TARGETS[target]
        prefix = f"regressee-{product}-{platform}"
        archive = self.output / (prefix + (".zip" if suffix else ".tar.gz"))
        if suffix:
            with zipfile.ZipFile(archive) as file:
                return {p.removeprefix(prefix + "/"): file.read(p) for p in file.namelist()}
        with tarfile.open(archive) as file:
            self.assertEqual(file.getmember(prefix + "/regressee").mode, 0o755)
            return {p.name.removeprefix(prefix + "/"): file.extractfile(p).read() for p in file.getmembers()}

    def test_cli_and_desktop_archives_contain_correct_executables_notices_skill_checksums(self):
        for target in preview.TARGETS:
            with self.subTest(target=target):
                self.output = self.parent / target
                names = self.build(target)
                self.assertEqual(len(names), 5)
                for product in preview.PRODUCTS:
                    files = self.files(product, target)
                    suffix = preview.TARGETS[target][1]
                    info = json.loads(files["build-info.json"])
                    self.assertEqual(info["product"], product)
                    self.assertEqual(info["source_commit"], REVISION)
                    self.assertFalse(info["dirty"])
                    self.assertFalse(info["runtime_bundled"] or info["signed"] or info["release_approved"])
                    self.assertEqual(set(info["executables"]), {name + suffix for name in preview.PRODUCTS[product]})
                    for name, entry in info["executables"].items():
                        self.assertEqual(entry["sha256"], preview.digest(files[name]))
                        self.assertEqual(entry["build"], metadata(name.removesuffix(".exe")))
                    self.assertEqual("regressee-desktop" + suffix in files, product == "desktop")
                    self.assertTrue(preview.source.SKILL_FILES <= files.keys())
                    self.assertEqual(validate_skill(files["skills/regressee/SKILL.md"])["name"], "regressee")
                    self.assertIn("help/cli.md", files)
                    self.assertIn("notices/fixture-1.0/LICENSE", files)
                    for link in re.findall(r"\]\(([^)]+)\)", files["README.md"].decode()):
                        self.assertIn(link, files)
                    for line in files["SHA256SUMS"].decode().splitlines():
                        sha, name = line.split("  ")
                        self.assertEqual(sha, preview.digest(files[name]))
                    self.assertNotIn(b"not for distribution", b"".join(files.values()))
                with tarfile.open(self.output / "regressee-source.tar.gz") as file:
                    names = {m.name.removeprefix("regressee-source/") for m in file}
                    self.assertTrue(preview.source.SKILL_FILES <= names)
                    self.assertIn("docs/cli.md", names)
                    self.assertNotIn("docs/agent-cli-phase4-review.md", names)
                    self.assertNotIn(".runtime/private.key", names)

    def test_cli_only_does_not_require_desktop_binary(self):
        (self.root / f"target/{TARGET}/release/regressee-desktop").unlink()
        names = self.build(product="cli")
        self.assertEqual(len(names), 4)
        self.assertNotIn("regressee-desktop", self.files())

    def test_actual_release_notes_link_only_to_included_public_documents(self):
        actual = Path(__file__).resolve().parent.parent
        notes = (actual / "docs/preview-release-notes.md").read_bytes()
        put(self.root, "docs/preview-release-notes.md", notes)
        self.build(product="cli")
        files = self.files()
        text = files["help/preview-release-notes.md"].decode()
        self.assertNotIn("docs/research/", text)
        for link in re.findall(r"\]\(([^)]+)\)", text):
            self.assertIn("help/" + link, files)

    def test_invalid_skill_frontmatter_is_refused_before_archive_output(self):
        put(self.root, "skills/regressee/SKILL.md", "---\nname: regressee\ndescription: CLI: authorized\n---\n# Body")
        with self.assertRaises(preview.source.Refusal) as error:
            self.build(product="cli")
        self.assertEqual(str(error.exception), "invalid-portable-skill-frontmatter")
        self.assertFalse(self.output.exists())

    def test_dirty_source_requires_explicit_local_label_and_binds_worktree(self):
        self.dirty = " M README.md"
        with self.assertRaises(ValueError):
            self.build()
        self.assertFalse(self.output.exists())
        self.build(allow_dirty=True)
        self.assertTrue(json.loads(self.files()["build-info.json"])["dirty"])
        manifest = json.loads((self.output / "regressee-source.json").read_bytes())
        self.assertEqual(manifest["source_commit"], REVISION)
        self.assertEqual(manifest["source_inventory_sha256"], preview.digest(preview.source.json_bytes(manifest["files"])))

    def test_source_changes_during_probe_are_rejected_before_output(self):
        original = self.command
        def mutating(root, *args, **kwargs):
            if args[-1] == "--build-info":
                put(self.root, "README.md", "changed across packaging")
            return original(root, *args, **kwargs)
        with patch.object(preview, "command", side_effect=mutating), \
                patch.object(preview, "dependency_notices", side_effect=lambda r, t, p: inventory(t, p)), self.assertRaises(ValueError):
            preview.package(self.root, self.output, TARGET, REVISION)
        self.assertFalse(self.output.exists())

    def test_missing_empty_linked_and_changed_binary_refuse(self):
        path = self.root / f"target/{TARGET}/release/regressee"
        path.unlink()
        with self.assertRaises(FileNotFoundError):
            self.build()
        path.write_bytes(b"")
        with self.assertRaises(ValueError):
            self.build()
        path.unlink()
        if os.name != "nt":
            path.symlink_to(self.root / "README.md")
            with self.assertRaises(ValueError):
                self.build()
            path.unlink()
        path.write_bytes(b"binary")
        original = self.command
        def mutating(root, *args, **kwargs):
            if args[-1] == "--build-info":
                path.write_bytes(b"changed binary")
            return original(root, *args, **kwargs)
        with patch.object(preview, "command", side_effect=mutating), self.assertRaises(ValueError):
            preview.package(self.root, self.output, TARGET, REVISION)
        self.assertFalse(self.output.exists())

    def test_mismatched_protocol_version_product_and_feature_leakage_refuse(self):
        for key, value in (("version", "other"), ("executable", "regressee-desktop"),
                           ("engine_protocol", 5), ("cli_schema", 2)):
            self.infos["regressee"] = {**metadata("regressee"), key: value}
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.build()
        for name in self.infos:
            for feature in preview.FEATURES:
                self.infos = {n: metadata(n) for n in self.infos}
                self.infos[name]["features"][feature] = not self.infos[name]["features"][feature]
                with self.subTest(name=name, feature=feature), self.assertRaises(ValueError):
                    self.build()
        self.infos = {n: metadata(n) for n in self.infos}
        self.infos["regressee"]["features"]["ui"] = 0
        with self.assertRaises(ValueError):
            self.build()
        self.assertFalse(self.output.exists())

    def test_missing_or_malicious_notice_inventory_and_path_containment(self):
        for change in ("target", "features", "missing", "hash", "escape", "orphan", "gpui"):
            inv, texts = inventory(TARGET, "cli")
            if change == "target": inv["target"] = "wrong"
            if change == "features": inv["features"] = ["ui"]
            if change == "missing": texts.clear()
            if change == "hash": inv["packages"][0]["texts"][0]["sha256"] = "0" * 64
            if change == "escape": inv["packages"][0]["texts"][0]["path"] = "../outside"
            if change == "orphan": texts["outside"] = (b"unreviewed", 0o644)
            if change == "gpui": inv["packages"][0]["name"] = "gpui"
            with self.subTest(change=change), self.assertRaises((ValueError, preview.source.Refusal)):
                preview.validate_inventory(inv, texts, TARGET, "cli")

    def test_output_and_missing_public_inputs_do_not_overwrite(self):
        self.output.mkdir()
        marker = self.output / "keep"
        marker.write_bytes(b"keep")
        with self.assertRaises(FileExistsError):
            self.build()
        self.assertEqual(marker.read_bytes(), b"keep")
        self.output = self.root / "crates/new-package"
        with self.assertRaises(ValueError):
            self.build()
        self.output = self.parent / "missing-skill"
        (self.root / "skills/regressee/SKILL.md").unlink()
        with self.assertRaises(preview.source.Refusal):
            self.build()
        self.assertFalse(self.output.exists())

    def test_probe_is_data_free_isolated_and_bounded(self):
        with patch.object(preview, "command", side_effect=self.command) as call:
            preview.probe(self.root, self.root / f"target/{TARGET}/release/regressee", "regressee", "0.1.0")
        self.assertEqual(call.call_args.kwargs["timeout"], 15)
        env = call.call_args.kwargs["env"]
        self.assertNotIn("DISPLAY", env)
        self.assertNotIn("WAYLAND_DISPLAY", env)
        self.assertFalse(Path(env["HOME"]).exists())
        for output in ("not json", "{}"):
            with patch.object(preview, "command", return_value=output), self.assertRaises((ValueError, KeyError)):
                preview.probe(self.root, Path("bad"), "regressee", "0.1.0")
        with patch.object(preview, "command", side_effect=subprocess.TimeoutExpired("bad", 15)), self.assertRaises(subprocess.TimeoutExpired):
            preview.probe(self.root, Path("bad"), "regressee", "0.1.0")
        def writes_home(root, *args, **kwargs):
            (Path(kwargs["env"]["HOME"]) / "unexpected-private-data").write_text("malicious probe")
            return json.dumps(metadata("regressee"))
        with patch.object(preview, "command", side_effect=writes_home), self.assertRaises(ValueError):
            preview.probe(self.root, Path("bad"), "regressee", "0.1.0")

    def test_build_receipt_must_bind_selected_sources_and_exact_binary_bytes(self):
        receipt = {"target": TARGET, "product": "all", "source_inventory_sha256": "0" * 64,
                   "binaries": {}}
        with self.assertRaises(ValueError):
            self.build(build_receipt=receipt)
        receipt["source_inventory_sha256"] = preview.digest(preview.source.json_bytes(preview.source.inventory(preview.selected_sources(self.root))))
        receipt["binaries"] = {name: preview.digest((self.root / f"target/{TARGET}/release/{name}").read_bytes())
                               for name in ("regressee", "regressee-desktop")}
        self.build(build_receipt=receipt)
        self.assertEqual(json.loads(self.files()["build-receipt.json"]), receipt)
        self.assertEqual(json.loads(self.files()["build-info.json"])["source_binding"], "frozen-before-and-after-build")

    def test_dependency_graph_cli_and_desktop_are_product_specific(self):
        (self.root / "Cargo.lock").write_text('[[package]]\nname="example"\nversion="1.0.0"\nchecksum="fixture"\n')
        cache = self.root / "cargo/registry/src/fixture/example-1.0.0"
        cache.mkdir(parents=True)
        (cache / "Cargo.toml").write_text('[package]\nlicense="MIT"\nlicense-file="legal/license.txt"\n')
        (cache / "legal").mkdir()
        (cache / "legal/license.txt").write_text("example license")
        for product in preview.PRODUCTS:
            with patch.dict(os.environ, {"CARGO_HOME": str(self.root / "cargo")}), \
                    patch.object(preview, "command", return_value="regressee-cli v0.1.0\nexample v1.0.0") as call:
                inv, texts = preview.dependency_notices(self.root, TARGET, product)
            self.assertEqual(inv["product"], product)
            self.assertEqual("--features" in call.call_args.args, product == "desktop")
            self.assertEqual(len(texts), 1)
        for graph in ("", "regressee-cli v0.1.0\ngpui v0.2.2"):
            with patch.object(preview, "command", return_value=graph), self.assertRaises(ValueError):
                preview.dependency_notices(self.root, TARGET, "cli")


if __name__ == "__main__":
    unittest.main()
