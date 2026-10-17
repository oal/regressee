#!/usr/bin/env python3
"""Independent synthetic-tree tests only; no project suites or source execution."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import stat
import struct
import tarfile
import tempfile
import unittest
from unittest import mock

import source_release as helper


BASE = Path(__file__).parent
EPOCH = 1234567890
SENTINEL = b"PRIVATE-SENTINEL-8e36703e-never-archive-or-manifest"


def put(root, path, data):
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data.encode() if isinstance(data, str) else data)


def fixture(root):
    """No real accounts/keys/data: minimal independently authored source tree."""
    put(root, "Cargo.toml", '[workspace]\nmembers = ["crates/app"]\nresolver = "2"\n[workspace.package]\nlicense = "MPL-2.0"\n')
    put(root, "Cargo.lock", 'version = 4\n[[package]]\nname = "synthetic-app"\nversion = "0.1.0"\n[[package]]\nname = "declared-upstream"\nversion = "1.2.3"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "' + "a" * 64 + '"\n')
    put(root, "rust-toolchain.toml", '[toolchain]\nchannel = "1.98.1"\n')
    put(root, "LICENSE", 'Mozilla Public License Version 2.0\nSYNTHETIC TEST STUB, NOT A LICENSE COPY\n')
    put(root, "THIRD_PARTY_NOTICES.md", '# Notices\nApache input adaptation retains its separate license.\n')
    put(root, "README.md", '# Synthetic source fixture\nBuild instructions, no release qualification.\n')
    put(root, "crates/app/Cargo.toml", '[package]\nname = "synthetic-app"\nversion = "0.1.0"\nlicense.workspace = true\n')
    put(root, "crates/app/src/lib.rs", 'const QUERY: &str = include_str!("../migrations/001.sql");\n')
    put(root, "crates/app/src/ui/text_input.rs", '// Adapted from GPUI example (Apache-2.0; Zed contributors).\n// SPDX-License-Identifier: Apache-2.0\n')
    put(root, "crates/app/migrations/001.sql", 'CREATE TABLE synthetic (id INTEGER);\n')
    put(root, "crates/app/tests/source_test.rs", '#[test] fn fixture_only() { assert!(true); }\n')
    package = {"name": "synthetic-worker", "version": "0.1.0", "license": "MPL-2.0", "dependencies": {"tiny-dependency": "1.0.0"}}
    lock = {"name": package["name"], "version": package["version"], "lockfileVersion": 3,
            "packages": {"": package, "node_modules/tiny-dependency": {"version": "1.0.0", "license": "MIT",
                         "resolved": "https://registry.npmjs.org/tiny-dependency/-/tiny-dependency-1.0.0.tgz", "integrity": "sha512-YWJjZA=="}}}
    for prefix in ("worker", "worker/product"):
        put(root, prefix + "/package.json", json.dumps(package))
        put(root, prefix + "/package-lock.json", json.dumps(lock))
        put(root, prefix + "/src/index.mjs", 'export const protocol = "synthetic";\n')
        put(root, prefix + "/test/protocol.test.mjs", '// SYNTHETIC worker test source, not executed here.\n')
    put(root, "worker/product/test/fixtures/session-requests.ndjson", '{"synthetic":true}\n')
    put(root, "scripts/build-ui.sh", '#!/bin/sh\n# Build recipe, never executed by this test.\n')
    put(root, "scripts/bootstrap.py", '# Synthetic build/pack recipe; never executed.\n')
    put(root, "tests/fixtures/analysis-proposal/corpus.py", '# Preferred source for synthetic PNG fixture\n')
    put(root, "tests/fixtures/analysis-proposal/images/before.png", b"\x89PNG\r\n\x1a\nsynthetic-not-decoded-here")
    put(root, "tests/fixtures/analysis-proposal/manifest.json", '{"fixture":"synthetic"}\n')
    put(root, "tests/fixtures/analysis-proposal/LICENSE", 'Synthetic fixture attribution, retained verbatim\n')
    put(root, "third-party/GPUI-LICENSE-APACHE", 'Apache License\nSYNTHETIC TEST STUB\n')
    put(root, ".github/workflows/source.yml", 'name: synthetic-build-recipe\n')
    for name in helper.USER_DOCS:
        put(root, "docs/" + name, '# Synthetic user-facing documentation\n')
    for name in helper.SKILL_FILES:
        put(root, name, '---\n{"name":"regressee","description":"Synthetic skill fixture"}\n---\n# Synthetic portable skill\n' if name.endswith("SKILL.md") else '# Synthetic portable skill reference\n')


class SyntheticSourceTests(unittest.TestCase):
    def test_readme_comparison_image_and_provenance_resolve_in_source_archive(self):
        import re
        actual = BASE.parent
        readme = (actual / "README.md").read_bytes()
        images = re.findall(rb"!\[[^\]]*\]\((assets/screenshots/[^)]+)\)", readme)
        self.assertEqual(images, [b"assets/screenshots/regressee-comparison.png"])
        image_path = images[0].decode()
        provenance_path = "assets/screenshots/README.md"
        self.assertIn(f"]({provenance_path})".encode(), readme)
        image = (actual / image_path).read_bytes()
        self.assertTrue(image.startswith(b"\x89PNG\r\n\x1a\n"))
        self.assertEqual(struct.unpack_from(">II", image, 16), (1440, 900))
        self.assertIn(b"Origin: Actual Regressee native comparison workspace", image)
        provenance = (actual / provenance_path).read_bytes()
        self.assertIn(b"offscreen", provenance)
        self.assertIn(b"loopback", provenance)
        self.assertNotIn(b"/var/" + b"home/", provenance)
        for path, data in [("README.md", readme), (image_path, image), (provenance_path, provenance)]:
            put(self.root, path, data)
        self.build()
        with tarfile.open(self.output / "source.tar.gz") as archive:
            for path in [image_path, provenance_path]:
                self.assertEqual(archive.extractfile("regressee-source/" + path).read(),
                                 (actual / path).read_bytes())

    def test_phase7_reviewed_binary_fixture_inputs_are_exact_and_required(self):
        actual = Path(__file__).resolve().parent.parent
        for name in helper.FIXTURE_ARCHIVES:
            put(self.root,name,(actual/name).read_bytes())
        self.build()
        paths={r['path'] for r in self.manifest()['source_files']}
        self.assertTrue(set(helper.FIXTURE_ARCHIVES) <= paths)
        name=next(iter(helper.FIXTURE_ARCHIVES))
        put(self.root,name,b'changed synthetic ZIP')
        self.refused('reviewed-fixture-archive-changed',self.parent/'changed-candidate')
    def test_phase7_arbitrary_archive_is_not_a_source_allowlist_extension(self):
        put(self.root,'tests/fixtures/unreviewed.zip',b'arbitrary ZIP')
        self.refused('unclassified-source-tree-file')
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="synthetic-", dir=BASE)
        self.parent = Path(self.temp.name)
        self.root = self.parent / "tree"
        self.root.mkdir(mode=0o700)
        fixture(self.root)
        self.output = self.parent / "candidate"

    def tearDown(self):
        self.temp.cleanup()  # Only this test's newly created isolated synthetic tree.

    def build(self, output=None):
        return helper.build(str(self.root), str(output or self.output), EPOCH)

    def manifest(self, output=None):
        return json.loads(((output or self.output) / "manifest.json").read_bytes())

    def refused(self, reason, output=None):
        target = output or self.output
        with self.assertRaises(helper.Refusal) as caught:
            self.build(target)
        self.assertEqual(str(caught.exception), reason)
        if target != self.output or not self.output.exists():
            self.assertFalse(target.exists())
        self.assertFalse(any(p.name.startswith(".source-stage-") for p in self.parent.iterdir()))

    def test_repeat_exact_bytes_despite_mtime_permissions_creation_order(self):
        first = self.build()
        for path in self.root.rglob("*"):
            if path.is_file():
                os.utime(path, (987654321, 987654321))
                os.chmod(path, 0o755 if path.suffix != ".sh" else 0o600)
        # Recreate files in a different directory entry order.
        for path in sorted(self.root.rglob("*"), reverse=True):
            if path.is_file():
                value = path.read_bytes()
                path.unlink()
                path.write_bytes(value)
        second_path = self.parent / "repeat"
        second = self.build(second_path)
        self.assertEqual(first, second)
        for name in ("source.tar.gz", "manifest.json", "SHA256SUMS"):
            self.assertEqual((self.output / name).read_bytes(), (second_path / name).read_bytes())

    def test_archive_inventory_and_hashes_independently_match_exact_source(self):
        result = self.build()
        manifest = self.manifest()
        rows = manifest["source_files"]
        paths = [row["path"] for row in rows]
        self.assertEqual(paths, sorted(paths))
        self.assertEqual(len(paths), len(set(paths)))
        encoded_rows = (json.dumps(rows, sort_keys=True, ensure_ascii=True, separators=(",", ":")) + "\n").encode()
        independent_identity = hashlib.sha256(encoded_rows).hexdigest()
        self.assertEqual(result["source_sha256"], independent_identity)
        self.assertEqual(manifest["source_identity"]["sha256"], independent_identity)
        expected = {p.relative_to(self.root).as_posix(): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        self.assertEqual(set(paths), set(expected))
        with tarfile.open(self.output / "source.tar.gz", "r:gz") as archive:
            members = archive.getmembers()
            self.assertEqual([m.name for m in members], ["regressee-source/" + p for p in paths])
            for member, row in zip(members, rows):
                content = archive.extractfile(member).read()
                self.assertEqual(content, expected[row["path"]])
                self.assertEqual(hashlib.sha256(content).hexdigest(), row["sha256"])
                self.assertEqual(len(content), row["size"])
                self.assertEqual(member.mode, int(row["mode"], 8))
                self.assertEqual((member.uid, member.gid, member.uname, member.gname, member.mtime), (0, 0, "", "", EPOCH))
                self.assertTrue(member.isfile())
        zipped = (self.output / "source.tar.gz").read_bytes()
        self.assertEqual(zipped[:4], b"\x1f\x8b\x08\x00")  # No filename/comment/header flags.
        self.assertEqual(struct.unpack("<I", zipped[4:8])[0], EPOCH)
        self.assertEqual(manifest["archive"]["sha256"], hashlib.sha256(zipped).hexdigest())
        checksum_lines = (self.output / "SHA256SUMS").read_text().splitlines()
        for line in checksum_lines:
            value, name = line.split("  ")
            self.assertEqual(value, hashlib.sha256((self.output / name).read_bytes()).hexdigest())

    def test_required_source_locks_notices_docs_and_fixtures_present(self):
        self.build()
        paths = {r["path"] for r in self.manifest()["source_files"]}
        expected = {"LICENSE", "THIRD_PARTY_NOTICES.md", "Cargo.lock", "Cargo.toml", "rust-toolchain.toml",
                    "crates/app/src/ui/text_input.rs", "crates/app/migrations/001.sql", "crates/app/tests/source_test.rs",
                    "worker/package-lock.json", "worker/product/package-lock.json", "worker/product/src/index.mjs",
                    "worker/product/test/fixtures/session-requests.ndjson", "scripts/bootstrap.py", "scripts/build-ui.sh",
                    "tests/fixtures/analysis-proposal/corpus.py", "tests/fixtures/analysis-proposal/images/before.png",
                    "third-party/GPUI-LICENSE-APACHE", "docs/portable-baseline-archive.md", ".github/workflows/source.yml"}
        self.assertTrue(expected <= paths)
        self.assertTrue({"docs/"+name for name in helper.USER_DOCS} <= paths)
        self.assertTrue(helper.SKILL_FILES <= paths)

    def test_portable_skill_is_allowlisted_but_internal_reviews_are_not(self):
        for name in helper.INTERNAL_DOCS:
            put(self.root, "docs/" + name, SENTINEL)
        self.build()
        paths = {row["path"] for row in self.manifest()["source_files"]}
        self.assertTrue(helper.SKILL_FILES <= paths)
        self.assertFalse(paths & {"docs/" + name for name in helper.INTERNAL_DOCS})
        put(self.root, "skills/unreviewed/SKILL.md", "unreviewed")
        self.refused("unclassified-skill-entry", self.parent / "unreviewed-skill")

    def test_structural_private_internal_and_report_omission(self):
        omitted = [".runtime/installed/source.mjs", "target/debug/generated.rs", "artifacts/private.png",
                   "privateuserdata/accounts.json", "cache/private.txt", "credentials/passwords.json", "logs/run.log",
                   "keyfiles/production.key", "sessionstate/state.json", "worker/node_modules/pkg/index.mjs",
                   "worker/.env", "worker/cache/cookie.json", "scripts/__pycache__/private.pyc", "docs/ACTIVE_WORK.md",
                   "docs/IMPLEMENTATION_STATUS.md", "docs/phases/06-verification.md", "docs/research/private-report.md",
                   "docs/decisions/private.md", "tests/fixtures/analysis-proposal/validation.json",
                   "tests/fixtures/analysis-proposal/independent-validation-new.json", "PLAN.md", "DESIGN.md"]
        for path in omitted:
            put(self.root, path, SENTINEL)
        outside = self.parent / "do-not-follow"
        outside.mkdir()
        put(outside, "secret.txt", SENTINEL)
        (self.root / "private").symlink_to(outside, target_is_directory=True)
        self.build()
        manifest_bytes = (self.output / "manifest.json").read_bytes()
        self.assertNotIn(SENTINEL, manifest_bytes)
        self.assertNotIn(str(self.parent).encode(), manifest_bytes)
        self.assertNotIn(str(outside).encode(), manifest_bytes)
        paths = {r["path"] for r in self.manifest()["source_files"]}
        self.assertTrue(paths.isdisjoint(omitted))
        with tarfile.open(self.output / "source.tar.gz", "r:gz") as archive:
            for member in archive:
                self.assertNotIn(SENTINEL, archive.extractfile(member).read())
                self.assertFalse(member.issym() or member.islnk())

    def test_needed_file_symlinks_contained_outside_and_dangling_refused(self):
        target = self.root / "crates/app/src/lib.rs"
        for name in ("text_input.rs", str(self.parent / "outside-secret"), "nonexistent.rs"):
            with self.subTest(target=name):
                target.unlink()
                target.symlink_to(name)
                self.refused("needed-source-symlink-rejected")

    def test_needed_directory_symlink_refused_without_following(self):
        (self.root / "worker/linked").symlink_to(self.root / "crates", target_is_directory=True)
        self.refused("needed-source-symlink-rejected")

    def test_hardlink_and_fifo_refused(self):
        victim = self.root / "crates/app/src/linked.rs"
        os.link(self.root / "README.md", victim)
        self.refused("special-or-hardlinked-source-rejected")
        victim.unlink()
        os.mkfifo(victim)
        self.refused("special-or-hardlinked-source-rejected")

    def test_source_root_or_ancestor_symlink_refused(self):
        link = self.parent / "root-link"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(helper.Refusal, "directory-link-or-unavailable"):
            helper.build(str(link), str(self.output), EPOCH)
        parent_link = self.parent / "parent-link"
        parent_link.symlink_to(self.parent, target_is_directory=True)
        with self.assertRaisesRegex(helper.Refusal, "directory-link-or-unavailable"):
            helper.build(str(parent_link / "tree"), str(self.output), EPOCH)
        self.assertFalse(self.output.exists())

    def test_unsafe_portable_filenames_refused(self):
        for name in ("bad\\file.rs", "bad:file.rs", "CON.rs", "file..", "line\nfile.rs"):
            with self.subTest(name=name):
                target = self.root / "crates/app/src" / name
                target.write_text("// fixture\n")
                with self.assertRaises(helper.Refusal):
                    self.build()
                target.unlink()
                self.assertFalse(self.output.exists())

    def test_casefold_collision_refused(self):
        put(self.root, "crates/app/src/COLLISION.rs", "// a\n")
        put(self.root, "crates/app/src/collision.rs", "// b\n")
        self.refused("portable-path-collision")

    def test_missing_required_license_lock_tree_source_or_include_refused(self):
        for name in ("LICENSE", "Cargo.lock", "worker/product/package-lock.json", "crates/app/migrations/001.sql"):
            with self.subTest(name=name):
                path = self.root / name
                value = path.read_bytes()
                path.unlink()
                with self.assertRaises(helper.Refusal):
                    self.build()
                self.assertFalse(self.output.exists())
                path.write_bytes(value)
        put(self.root, "crates/app/src/lib.rs", 'include_str!("../../../.runtime/secret.rs");\n')
        self.refused("required-referenced-source-omitted")

    def test_unknown_new_root_or_unknown_source_extension_refused(self):
        put(self.root, "new-covered-source/app.rs", "// newly added covered source\n")
        self.refused("unclassified-root-entry")
        (self.root / "new-covered-source/app.rs").unlink()
        (self.root / "new-covered-source").rmdir()
        put(self.root, "crates/app/src/unclassified.custom", "new preferred source\n")
        self.refused("unclassified-source-tree-file")

    def test_private_source_directory_refused_instead_of_silent_omission(self):
        put(self.root, "crates/app/src/private/important.rs", "// needed covered source\n")
        self.refused("excluded-directory-in-required-source-tree")

    def test_workspace_escaping_missing_and_wildcard_members_refused(self):
        path = self.root / "Cargo.toml"
        original = path.read_text()
        for reference in ("../external", "/outside", "crates/absent", "crates/*"):
            with self.subTest(member=reference):
                path.write_text(original.replace("crates/app", reference))
                with self.assertRaises(helper.Refusal):
                    self.build()
                self.assertFalse(self.output.exists())

    def test_path_dependency_escape_and_local_npm_dependency_refused(self):
        path = self.root / "crates/app/Cargo.toml"
        original = path.read_text()
        path.write_text(original + '\n[dependencies]\noutside = { path = "../../../outside" }\n')
        self.refused("source-reference-escapes-root")
        path.write_text(original)
        npm = self.root / "worker/package.json"
        package = json.loads(npm.read_bytes())
        package["dependencies"]["tiny-dependency"] = "file:../../private"
        npm.write_text(json.dumps(package))
        lock_path = self.root / "worker/package-lock.json"
        lock = json.loads(lock_path.read_bytes())
        lock["packages"][""]["dependencies"] = package["dependencies"]
        lock_path.write_text(json.dumps(lock))
        self.refused("npm-local-dependency-requires-review")

    def test_implicit_rust_module_and_package_entrypoint_omissions_refused(self):
        path = self.root / "crates/app/src/lib.rs"
        original = path.read_bytes()
        path.write_text('pub mod absent;\n')
        self.refused("implicit-rust-module-missing-or-ambiguous")
        path.write_text('pub mod present;\n')
        put(self.root, "crates/app/src/present.rs", '// module source\n')
        self.build()
        # A separate candidate must still be refused if the library entrypoint
        # disappears, even though helper/test Rust files remain in the package.
        path.unlink()
        self.refused("cargo-package-entrypoint-missing", self.parent / "missing-entrypoint")
        path.write_bytes(original)

    def test_required_userdoc_notice_and_worker_omissions_refused(self):
        for name in ("docs/runtime-distribution.md", "third-party/GPUI-LICENSE-APACHE", "worker/product/package.json"):
            with self.subTest(name=name):
                path = self.root / name
                value = path.read_bytes()
                path.unlink()
                try:
                    expected = "required-source-tree-empty" if name == "third-party/GPUI-LICENSE-APACHE" else "required-worker-notice-or-user-documentation-missing"
                    self.refused(expected)
                finally:
                    path.write_bytes(value)

    def test_npm_lock_omissions_and_dependency_mismatch_refused(self):
        path = self.root / "worker/package-lock.json"
        original = path.read_bytes()
        lock = json.loads(original)
        lock["packages"][""]["dependencies"]["tiny-dependency"] = "2.0.0"
        path.write_text(json.dumps(lock))
        self.refused("npm-lock-dependency-mismatch")
        lock = json.loads(original)
        del lock["packages"]["node_modules/tiny-dependency"]
        path.write_text(json.dumps(lock))
        self.refused("npm-lock-required-dependency-omitted")

    def test_generated_assumptions_do_not_silently_omit_new_build_source(self):
        put(self.root, "scripts/build/pack.py", '# Needed packaging source\n')
        self.build()
        self.assertIn("scripts/build/pack.py", {r["path"] for r in self.manifest()["source_files"]})
        put(self.root, ".cargo/config.toml", '[build]\n# Needs a reviewed public/private boundary\n')
        self.refused("unclassified-root-entry", self.parent / "cargo-config-refusal")

    def test_private_key_marker_refused_without_leaking_content(self):
        # Construct the synthetic forbidden marker without putting a literal PEM
        # header in this shipped test source. Generated fixture is still refused.
        marker = b"-----BEGIN " + b"PRIVATE KEY-----\n" + SENTINEL + b"\n-----END PRIVATE KEY-----\n"
        put(self.root, "tests/fixture.json", marker)
        self.refused("private-key-material-in-source")

    def test_dependency_unknown_or_declared_is_never_approved(self):
        self.build()
        manifest = self.manifest()
        dependencies = manifest["dependencies"]
        self.assertIs(dependencies["approved"], False)
        self.assertEqual(dependencies["status"], "blocked-unknown-and-unreviewed")
        self.assertGreater(dependencies["unknown_count"], 0)
        self.assertEqual(dependencies["review_required_count"], len(dependencies["cargo"]) + len(dependencies["npm"]))
        for package in dependencies["cargo"] + dependencies["npm"]:
            self.assertEqual(set(package), {"name", "version", "license", "source", "checksum"})
            self.assertNotIn("approved", package)
        self.assertIs(manifest["release_gate"]["approved"], False)
        self.assertIsNone(manifest["git_commit"])
        self.assertTrue(all(not row["path"].startswith("/") and ".." not in row["path"].split("/") for row in manifest["source_files"]))

    def test_missing_local_license_is_unknown_not_missing_source(self):
        path = self.root / "crates/app/Cargo.toml"
        path.write_text(path.read_text().replace("license.workspace = true\n", ""))
        self.build()
        local = next(p for p in self.manifest()["dependencies"]["cargo"] if p["name"] == "synthetic-app")
        self.assertIsNone(local["license"])
        self.assertIs(self.manifest()["dependencies"]["approved"], False)

    def test_dependency_localpaths_credentials_and_bad_checksums_refused(self):
        path = self.root / "Cargo.lock"
        original = path.read_text()
        registry = "registry+https://github.com/rust-lang/crates.io-index"
        for source in ("file:///home/sentinel/.cargo/pkg", "registry+https://user:password@host/index", "git+https://example.test/private"):
            with self.subTest(source=source):
                path.write_text(original.replace(registry, source))
                self.refused("nonallowlisted-dependency-source")
        path.write_text(original.replace('name = "declared-upstream"', 'name = "/home/private/path"'))
        self.refused("unsafe-dependency-name")
        path.write_text(original.replace("a" * 64, "not-a-hash"))
        self.refused("invalid-dependency-checksum")

    def test_output_private_atomic_noclobber_and_original_preservation(self):
        self.build()
        original = {p.name: p.read_bytes() for p in self.output.iterdir()}
        self.refused("output-collision")
        self.assertEqual(original, {p.name: p.read_bytes() for p in self.output.iterdir()})
        self.assertEqual(stat.S_IMODE(self.output.stat().st_mode), 0o700)
        for path in self.output.iterdir():
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
        empty = self.parent / "preexisting-empty"
        empty.mkdir()
        with self.assertRaisesRegex(helper.Refusal, "output-collision"):
            self.build(empty)
        self.assertEqual(list(empty.iterdir()), [])

    def test_output_inside_source_and_nonprivate_parent_refused(self):
        self.refused("output-inside-source-root", self.root / "candidate")
        public = self.parent / "public-parent"
        public.mkdir(mode=0o755)
        self.refused("output-parent-not-owner-private", public / "candidate")

    def test_output_symlink_and_traversal_refused(self):
        link = self.parent / "parent-link"
        link.symlink_to(self.parent, target_is_directory=True)
        self.refused("directory-link-or-unavailable", link / "candidate")
        for output in (str(self.parent) + "/tree/../candidate", str(self.parent) + "/../candidate", "relative-candidate"):
            with self.subTest(output=output), self.assertRaises(helper.Refusal):
                helper.build(str(self.root), output, EPOCH)
        self.assertFalse(self.output.exists())
        # Preexisting output symlink must remain a symlink, never be replaced.
        self.output.symlink_to(self.parent / "outside-destination")
        with self.assertRaisesRegex(helper.Refusal, "output-collision"):
            self.build()
        self.assertTrue(self.output.is_symlink())

    def test_atomic_publication_race_refuses_empty_dir_collision(self):
        original = helper.rename_noreplace

        def raced(parent_fd, stage, final):
            os.mkdir(final, 0o700, dir_fd=parent_fd)
            return original(parent_fd, stage, final)

        with mock.patch.object(helper, "rename_noreplace", side_effect=raced):
            with self.assertRaisesRegex(helper.Refusal, "output-collision"):
                self.build()
        self.assertEqual(list(self.output.iterdir()), [])
        self.assertFalse(any(p.name.startswith(".source-stage-") for p in self.parent.iterdir()))

    def test_atomic_publication_failure_leaves_no_partial_output(self):
        with mock.patch.object(helper, "rename_noreplace", side_effect=helper.Refusal("injected-before-publish")):
            self.refused("injected-before-publish")

    def test_concurrent_source_change_refused_before_output(self):
        original = helper.make_archive

        def changed(files, epoch):
            data = original(files, epoch)
            put(self.root, "crates/app/src/lib.rs", '// concurrently changed\n')
            return data

        with mock.patch.object(helper, "make_archive", side_effect=changed):
            self.refused("selected-source-changed-during-snapshot")

    def test_epoch_missing_invalid_and_out_of_range_refused(self):
        for value in (-1, 2 ** 32, True, "123", None):
            with self.subTest(epoch=value), self.assertRaisesRegex(helper.Refusal, "invalid-SOURCE_DATE_EPOCH"):
                helper.build(str(self.root), str(self.output), value)
        self.assertFalse(self.output.exists())
        with mock.patch.dict(os.environ, {}, clear=True), mock.patch("sys.argv", ["source_release.py", "--source", str(self.root), "--output", str(self.output)]), mock.patch("sys.stderr", new_callable=io.StringIO) as error:
            self.assertEqual(helper.main(), 2)
            self.assertEqual(error.getvalue(), "REFUSED: explicit-SOURCE_DATE_EPOCH-required\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", required=True, help="NEW private result file")
    args = parser.parse_args()
    fd = os.open(args.report, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as report:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(SyntheticSourceTests)
        result = unittest.TextTestRunner(stream=report, verbosity=2).run(suite)
        report.flush()
        os.fsync(report.fileno())
    print(json.dumps({"tests_run": result.testsRun, "failures": len(result.failures), "errors": len(result.errors),
                      "skipped": len(result.skipped), "successful": result.wasSuccessful()}, sort_keys=True))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
