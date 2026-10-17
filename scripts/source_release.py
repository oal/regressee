#!/usr/bin/env python3
"""Private source-candidate preparation, not publication or legal approval.

Python >=3.11 stdlib only. Linux is required for fd-relative NOFOLLOW reads and
atomic renameat2(RENAME_NOREPLACE) bundle publication. No subprocesses/network.
Source content is NEVER executed. Read the integration handoff before using.
"""
import argparse
import ctypes
import errno
import gzip
import hashlib
import io
import json
import os
import re
import stat
import sys
import tarfile
import tomllib
from pathlib import PurePosixPath


class Refusal(Exception):
    """Intentionally sanitized: never includes absolute paths or file content."""


POLICY = "regressee-source-candidate-v1"
FIXTURE_ARCHIVES = {
    "tests/fixtures/phase6-corrections/long-inflate.zip": "b4f0f2404da5652c164662a19ce3d887c36637a0f76f255a50af08d20c732e0e",
    "tests/fixtures/phase6-corrections/unknown-coverage-field.zip": "1c14a76b93b5a7e8a41575a68018f33bf00c9310ba898a490654972f334de03e",
}
ROOT_FILES = frozenset({"Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                        "LICENSE", "THIRD_PARTY_NOTICES.md", "README.md", ".gitignore"})
REQUIRED = ROOT_FILES - {".gitignore"}
TREES = frozenset({"crates", "worker", "scripts", "tests", "third-party"})
USER_DOCS = frozenset({"auth-and-capture.md", "portable-baseline-archive.md",
                       "runtime-distribution.md", "first-run-guide.md", "support-matrix.md",
                        "release-building.md", "updates-and-recovery.md",
                        "preview-release-notes.md"})
INTERNAL_DOCS = frozenset({"ACTIVE_WORK.md", "IMPLEMENTATION_STATUS.md",
                            "agent-cli-plan.md", "agent-cli-status.md",
                           "phase0-design.md", "spike-surface.md", "foundation-surface.md",
                           "crawling-surface.md", "auth-and-capture-surface.md",
                           "analysis-and-review-surface.md", "portability-and-storage-surface.md"})
INTERNAL_ROOT = frozenset({"DESIGN.md", "PLAN.md", "PRODUCT.md", "AGENTS.md", ".impeccable"})
PRIVATE_DIRS = frozenset({".runtime", "target", "artifacts", "node_modules", "__pycache__",
                          ".git", ".npm", ".cache", "cache", "caches", "logs",
                          "private", "privateuserdata", "private-user-data", "userdata",
                          "user-data", "credentials", "credential-store", "secrets",
                          "keys", "keyfiles", "sessionstate", "session-state", "sessions",
                          "browser-profile", "profiles", ".ssh", ".aws", ".gnupg",
                          ".pytest_cache", ".venv", "venv"})
PRIVATE_FILES = frozenset({"ACTIVE_WORK.md", "IMPLEMENTATION_STATUS.md", "id_rsa",
                           "id_ed25519", "cookies.json", "storage-state.json", "state.json",
                           "credentials.json", "credentials", "session-state.json",
                           "installation-id", ".DS_Store"})
PRIVATE_SUFFIX = (".pem", ".key", ".p12", ".pfx", ".keystore", ".log", ".pyc",
                  ".sqlite", ".sqlite3", ".db", ".sqlite3-wal", ".sqlite3-shm")
EXTENSIONS = frozenset({".rs", ".toml", ".lock", ".mjs", ".js", ".cjs", ".ts", ".tsx",
                        ".json", ".ndjson", ".md", ".sql", ".py", ".sh", ".yml", ".yaml",
                        ".html", ".css", ".svg", ".png", ".txt", ".h", ".c", ".cc",
                        ".cpp", ".wgsl", ".vert", ".frag"})
MAX_FILE = 64 * 1024 * 1024
MAX_TOTAL = 512 * 1024 * 1024
MAX_FILES = 20000
NOFOLLOW = os.O_NOFOLLOW if hasattr(os, "O_NOFOLLOW") else 0
DIRECTORY = os.O_DIRECTORY if hasattr(os, "O_DIRECTORY") else 0


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(",", ":")) + "\n").encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def safe_relative(path):
    if not isinstance(path, str) or not path or path.startswith("/") or "\\" in path:
        raise Refusal("unsafe-relative-path")
    parts = path.split("/")
    reserved = {"CON", "PRN", "AUX", "NUL", *("COM" + str(i) for i in range(1, 10)),
                *("LPT" + str(i) for i in range(1, 10))}
    for part in parts:
        if (part in {"", ".", ".."} or len(part.encode()) > 200 or
                not re.fullmatch(r"[A-Za-z0-9_.+-]+", part) or part.endswith((".", " ")) or
                part.split(".")[0].upper() in reserved):
            raise Refusal("unsafe-or-nonportable-path")
    if len(path.encode()) > 240:
        raise Refusal("archive-path-too-long")
    return path


def physical_dir(path):
    """Pin every directory component without resolving/following symlinks."""
    if sys.platform != "linux" or not NOFOLLOW or not DIRECTORY:
        raise Refusal("unsupported-safe-filesystem-platform")
    raw = os.fspath(path)
    if not raw.startswith("/") or raw == "/" or "\\" in raw:
        raise Refusal("absolute-physical-directory-required")
    parts = raw.split("/")[1:]
    if any(p in {"", ".", ".."} for p in parts):
        raise Refusal("noncanonical-directory-path")
    fd = os.open("/", os.O_RDONLY | DIRECTORY)
    try:
        for part in parts:
            next_fd = os.open(part, os.O_RDONLY | DIRECTORY | NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except OSError:
        os.close(fd)
        raise Refusal("directory-link-or-unavailable") from None


def private_name(name):
    lower = name.lower()
    return (name in PRIVATE_FILES or lower.endswith(PRIVATE_SUFFIX) or
            lower == ".env" or lower.startswith(".env.") or lower.startswith("credentials.") or
            lower.startswith("storage-state.") or lower.startswith("session-state.") or
            lower.endswith(("-status.md", "-verification.md", "-evidence.json")))


def classify(path, is_dir):
    parts = path.split("/")
    name = parts[-1]
    if path in FIXTURE_ARCHIVES and not is_dir:
        return "file"  # Reviewed exact synthetic include_bytes inputs, not arbitrary ZIPs.
    if path == "tests/fixtures/analysis-proposal/validation.json" or (path.startswith("tests/fixtures/analysis-proposal/") and name.startswith("independent-validation") and name.endswith(".json")):
        return "omit"  # Decoder reports are not the test corpus's preferred source.
    if name.lower() in PRIVATE_DIRS or private_name(name):
        # A source/ dependency tree containing a private child requires review:
        # never pretend dropping preferred modification files preserves source.
        if is_dir and any(p in {"src", "migrations"} for p in parts[:-1]):
            raise Refusal("excluded-directory-in-required-source-tree")
        if not is_dir and PurePosixPath(path).suffix in {".rs", ".mjs", ".py", ".sql"}:
            raise Refusal("excluded-source-file-requires-review")
        return "omit"
    if parts[0] in INTERNAL_ROOT:
        return "omit"
    if parts[0] == "docs":
        if len(parts) == 1:
            return "walk"
        if parts[1] in {"phases", "research", "decisions"} or parts[1] in INTERNAL_DOCS:
            return "omit"
        if len(parts) == 2 and name in USER_DOCS and not is_dir:
            return "file"
        raise Refusal("unclassified-documentation-entry")
    if parts[0] == ".github":
        if len(parts) == 1 or (len(parts) == 2 and name == "workflows" and is_dir):
            return "walk"
        if len(parts) == 3 and parts[1] == "workflows" and PurePosixPath(name).suffix in {".yml", ".yaml"}:
            return "file"
        raise Refusal("unclassified-ci-entry")
    if len(parts) == 1 and path in ROOT_FILES and not is_dir:
        return "file"
    if parts[0] in TREES:
        if is_dir:
            return "walk"
        if (PurePosixPath(name).suffix in EXTENSIONS or name in {"LICENSE", "NOTICE", "Makefile"} or
                name.startswith(("LICENSE-", "NOTICE-", "GPUI-LICENSE-"))):
            return "file"
        raise Refusal("unclassified-source-tree-file")
    raise Refusal("unclassified-root-entry")


def collect(root_fd):
    files = {}
    seen = set()
    total = 0

    def walk(fd, prefix):
        nonlocal total
        for name in sorted(os.listdir(fd)):
            rel = prefix + name
            st = os.stat(name, dir_fd=fd, follow_symlinks=False)
            try:
                action = classify(rel, stat.S_ISDIR(st.st_mode))
            except Refusal:
                if stat.S_ISLNK(st.st_mode):
                    raise Refusal("needed-source-symlink-rejected") from None
                raise
            if action == "omit":
                continue  # Do not enter or read excluded private subtrees.
            safe_relative(rel)
            case = rel.lower()
            if case in seen:
                raise Refusal("portable-path-collision")
            seen.add(case)
            if stat.S_ISLNK(st.st_mode):
                raise Refusal("needed-source-symlink-rejected")
            if action == "walk":
                if not stat.S_ISDIR(st.st_mode):
                    raise Refusal("required-tree-not-directory")
                child = os.open(name, os.O_RDONLY | DIRECTORY | NOFOLLOW, dir_fd=fd)
                try:
                    now = os.fstat(child)
                    if (st.st_dev, st.st_ino) != (now.st_dev, now.st_ino):
                        raise Refusal("source-changed-during-open")
                    walk(child, rel + "/")
                finally:
                    os.close(child)
                continue
            if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1:
                raise Refusal("special-or-hardlinked-source-rejected")
            if st.st_size > MAX_FILE or len(files) >= MAX_FILES:
                raise Refusal("source-size-bound")
            file_fd = os.open(name, os.O_RDONLY | NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
            try:
                before = os.fstat(file_fd)
                if (st.st_dev, st.st_ino) != (before.st_dev, before.st_ino) or not stat.S_ISREG(before.st_mode):
                    raise Refusal("source-changed-during-open")
                chunks = []
                size = 0
                while chunk := os.read(file_fd, min(1024 * 1024, MAX_FILE + 1 - size)):
                    chunks.append(chunk)
                    size += len(chunk)
                    if size > MAX_FILE:
                        raise Refusal("source-size-bound")
                after = os.fstat(file_fd)
                if (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_size, after.st_mtime_ns, after.st_ctime_ns):
                    raise Refusal("source-mutated-during-read")
                data = b"".join(chunks)
            finally:
                os.close(file_fd)
            if re.search(rb"-----BEGIN (?:[A-Z0-9 ]+ )?PRIVATE KEY-----", data):
                raise Refusal("private-key-material-in-source")
            if rel in FIXTURE_ARCHIVES and digest(data) != FIXTURE_ARCHIVES[rel]:
                raise Refusal("reviewed-fixture-archive-changed")
            total += len(data)
            if total > MAX_TOTAL:
                raise Refusal("source-total-bound")
            # Do not inherit executable bits, umask, or incidental host permissions.
            mode = 0o755 if PurePosixPath(name).suffix in {".py", ".sh"} else 0o644
            files[rel] = (data, mode)

    walk(root_fd, "")
    if not REQUIRED <= files.keys():
        raise Refusal("required-root-source-or-license-missing")
    if not files["LICENSE"][0].startswith(b"Mozilla Public License Version 2.0"):
        raise Refusal("expected-mpl-license-missing")
    for tree in TREES:
        if not any(p.startswith(tree + "/") for p in files):
            raise Refusal("required-source-tree-empty")
    return dict(sorted(files.items()))


def local_reference(base, reference, files, directory=False):
    """Allow lexical parent steps inside the source root, never outside it."""
    if not isinstance(reference, str) or not reference or reference.startswith(("/", "\\")) or "\\" in reference or ":" in reference:
        raise Refusal("unsafe-source-reference")
    parts = base.split("/") if base else []
    for part in reference.split("/"):
        if part == "..":
            if not parts:
                raise Refusal("source-reference-escapes-root")
            parts.pop()
        elif part not in {"", "."}:
            parts.append(part)
    path = safe_relative("/".join(parts))
    if directory:
        path += "/Cargo.toml"
    if path not in files:
        raise Refusal("required-referenced-source-omitted")
    return path


def parse_toml(data):
    try:
        return tomllib.loads(data.decode("utf-8"))
    except (ValueError, UnicodeError):
        raise Refusal("invalid-cargo-toml") from None


def validate_sources(files):
    essential = {"worker/package.json", "worker/package-lock.json", "worker/product/package.json",
                 "worker/product/package-lock.json", "third-party/GPUI-LICENSE-APACHE"}
    essential.update("docs/" + name for name in USER_DOCS)
    if not essential <= files.keys():
        raise Refusal("required-worker-notice-or-user-documentation-missing")
    for tree in {"worker/src", "worker/test", "worker/product/src", "worker/product/test"}:
        if not any(p.startswith(tree + "/") and p.endswith((".mjs", ".js", ".ts")) for p in files):
            raise Refusal("required-worker-source-or-test-tree-empty")
    workspace = parse_toml(files["Cargo.toml"][0]).get("workspace", {})
    members = workspace.get("members")
    if not isinstance(members, list) or not members:
        raise Refusal("workspace-members-required")
    for member in members:
        local_reference("", member, files, directory=True)
    if workspace.get("exclude"):
        raise Refusal("workspace-exclusions-require-policy-review")

    def dependencies(value, base):
        if isinstance(value, dict):
            for key, child in value.items():
                if key in {"dependencies", "dev-dependencies", "build-dependencies", "patch", "replace"}:
                    inspect_dependencies(child, base)
                else:
                    dependencies(child, base)
        elif isinstance(value, list):
            for child in value:
                dependencies(child, base)

    def inspect_dependencies(value, base):
        if isinstance(value, dict):
            if "path" in value:
                local_reference(base, value["path"], files, directory=True)
            for child in value.values():
                inspect_dependencies(child, base)

    for path, (data, _) in files.items():
        base = str(PurePosixPath(path).parent)
        if base == ".":
            base = ""
        if path.endswith("Cargo.toml"):
            document = parse_toml(data)
            dependencies(document, base)
            package = document.get("package", {})
            for key in {"license-file", "readme", "build"}:
                value = package.get(key)
                if isinstance(value, str):
                    local_reference(base, value, files)
            for kind in {"lib", "bin", "test", "bench", "example"}:
                entries = document.get(kind, [])
                if isinstance(entries, dict):
                    entries = [entries]
                for entry in entries:
                    if "path" in entry:
                        local_reference(base, entry["path"], files)
            if package and base:
                prefix = base + "/"
                entrypoints = {prefix + "src/lib.rs", prefix + "src/main.rs"}
                entrypoints.update(prefix + entry["path"] for entry in document.get("bin", []) if "path" in entry)
                if isinstance(document.get("lib"), dict) and "path" in document["lib"]:
                    entrypoints.add(prefix + document["lib"]["path"])
                if not (entrypoints & files.keys()) and not any(p.startswith(prefix + "src/bin/") and p.endswith(".rs") for p in files):
                    raise Refusal("cargo-package-entrypoint-missing")
            if base and not any(p.startswith(base + "/") and p.endswith(".rs") for p in files):
                raise Refusal("cargo-package-source-missing")
        elif path.endswith(".rs"):
            # Explicit literal includes/path attributes are checked. Dynamic macros,
            # generated code, scripts and runtime file references need owner review.
            text = data.decode("utf-8")
            for match in re.finditer(r'(?:include_(?:str|bytes)!\s*\(\s*|#\s*\[\s*path\s*=\s*)"([^"\n]+)"', text):
                local_reference(base, match.group(1), files)
            for module in re.finditer(r'(?m)^[ \t]*((?:#\[[^\n]*\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z0-9_]+)\s*;', text):
                attributes, name = module.groups()
                override = re.search(r'\bpath\s*=\s*"([^"\n]+)"', attributes)
                if override:
                    local_reference(base, override.group(1), files)
                else:
                    stem = PurePosixPath(path).stem
                    module_base = base if stem in {"lib", "main", "mod"} else base + "/" + stem
                    options = {module_base + "/" + name + ".rs", module_base + "/" + name + "/mod.rs"}
                    if len(options & files.keys()) != 1:
                        raise Refusal("implicit-rust-module-missing-or-ambiguous")
        elif path.endswith("package.json"):
            local_reference(base, "package-lock.json", files)
            try:
                package = json.loads(data)
                lock = json.loads(files[(base + "/" if base else "") + "package-lock.json"][0])
                if package.get("name") != lock.get("name") or package.get("version") != lock.get("version"):
                    raise Refusal("npm-lock-identity-mismatch")
                if lock.get("lockfileVersion") not in {2, 3} or not isinstance(lock.get("packages"), dict):
                    raise Refusal("unsupported-npm-lock-format")
                locked_root = lock["packages"].get("", {})
                if package.get("name") != locked_root.get("name") or package.get("version") != locked_root.get("version"):
                    raise Refusal("npm-lock-root-identity-mismatch")
                for key in {"dependencies", "devDependencies", "optionalDependencies"}:
                    mapping = package.get(key, {})
                    if mapping != locked_root.get(key, {}):
                        raise Refusal("npm-lock-dependency-mismatch")
                    for spec in mapping.values():
                        if not isinstance(spec, str) or spec.startswith(("file:", "link:", "workspace:")):
                            raise Refusal("npm-local-dependency-requires-review")
                    for name in mapping:
                        if "node_modules/" + name not in lock["packages"]:
                            raise Refusal("npm-lock-required-dependency-omitted")
            except (ValueError, TypeError, AttributeError):
                raise Refusal("invalid-npm-manifest-or-lock") from None


def public_text(value, field):
    if value is None:
        return None
    patterns = {"name": r"(?:@[A-Za-z0-9_.-]+/)?[A-Za-z0-9_][A-Za-z0-9_.-]*",
                "version": r"[0-9][A-Za-z0-9.+-]*",
                "license": r"[A-Za-z0-9][A-Za-z0-9_.+() \-]*"}
    if not isinstance(value, str) or len(value) > 300 or not re.fullmatch(patterns[field], value):
        raise Refusal("unsafe-dependency-" + field)
    return value


def public_source(value):
    if value is None:
        return None
    # Deliberately small public-source allowlist. Never serialize cargo IDs,
    # manifests/targets, local paths, arbitrary repository URLs or credentials.
    if value in {"registry+https://github.com/rust-lang/crates.io-index",
                 "sparse+https://index.crates.io/"}:
        return value
    raise Refusal("nonallowlisted-dependency-source")


def dependency_inventory(files):
    """Locks only; declarations do NOT constitute a license audit or SBOM."""
    packages = parse_toml(files["Cargo.lock"][0]).get("package", [])
    entries = []
    seen = set()
    for item in packages:
        name = public_text(item.get("name"), "name")
        version = public_text(item.get("version"), "version")
        if not name or not version:
            raise Refusal("dependency-identity-missing")
        source = public_source(item.get("source"))
        checksum = item.get("checksum")
        if source is not None and checksum is None:
            raise Refusal("registry-dependency-checksum-missing")
        if checksum is not None and not re.fullmatch(r"[0-9a-f]{64}", checksum):
            raise Refusal("invalid-dependency-checksum")
        license_value = None
        if source is None:
            found = False
            for path, (data, _) in files.items():
                if path.endswith("Cargo.toml"):
                    pkg = parse_toml(data).get("package", {})
                    if pkg.get("name") == name and pkg.get("version") == version:
                        found = True
                        declaration = pkg.get("license")
                        if isinstance(declaration, dict) and declaration.get("workspace") is True:
                            declaration = parse_toml(files["Cargo.toml"][0]).get("workspace", {}).get("package", {}).get("license")
                        license_value = public_text(declaration, "license")
            if not found:
                raise Refusal("local-lock-package-source-not-accounted-for")
        identity = (name, version, source)
        if identity in seen:
            raise Refusal("duplicate-dependency-identity")
        seen.add(identity)
        entries.append({"name": name, "version": version, "license": license_value,
                        "source": source, "checksum": checksum})
    if not entries:
        raise Refusal("cargo-lock-inventory-empty")
    entries.sort(key=lambda e: (e["name"], e["version"], e["source"] or ""))
    npm = []
    for path, (data, _) in files.items():
        if not path.endswith("package-lock.json"):
            continue
        lock = json.loads(data)
        for location, pkg in sorted(lock.get("packages", {}).items()):
            if not location:
                continue
            name = pkg.get("name") or location.rsplit("node_modules/", 1)[-1]
            name = public_text(name, "name")
            version = public_text(pkg.get("version"), "version")
            if not name or not version:
                raise Refusal("npm-dependency-identity-missing")
            resolved = pkg.get("resolved")
            if not isinstance(resolved, str) or not re.fullmatch(r"https://registry\.npmjs\.org/[A-Za-z0-9@_./+%-]+", resolved) or ".." in resolved:
                raise Refusal("nonallowlisted-npm-source")
            integrity = pkg.get("integrity")
            if not isinstance(integrity, str) or not re.fullmatch(r"sha512-[A-Za-z0-9+/=]+", integrity):
                raise Refusal("invalid-npm-integrity")
            # No conversion of npm's SHA512 SRI into an invented SHA256.
            npm.append({"name": name, "version": version, "license": public_text(pkg.get("license"), "license"),
                        "source": resolved, "checksum": integrity})
    return {"cargo": entries, "npm": sorted(npm, key=lambda e: (e["name"], e["version"], e["source"])),
            "status": "blocked-unknown-and-unreviewed", "approved": False,
            "scope": "lock-inventory-not-target-specific-not-binary-sbom",
            "unknown_count": sum(e["license"] is None for e in entries + npm),
            "review_required_count": len(entries) + len(npm)}


def inventory(files):
    return [{"path": path, "sha256": digest(data), "size": len(data), "mode": format(mode, "04o")}
            for path, (data, mode) in files.items()]


def make_archive(files, epoch):
    stream = io.BytesIO()
    with gzip.GzipFile(filename="", mode="wb", fileobj=stream, mtime=epoch, compresslevel=9) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as tar:
            for path, (data, mode) in files.items():
                entry = tarfile.TarInfo("regressee-source/" + path)
                entry.size = len(data)
                entry.mtime = epoch
                entry.uid = entry.gid = 0
                entry.uname = entry.gname = ""
                entry.mode = mode
                tar.addfile(entry, io.BytesIO(data))
    return stream.getvalue()


def rename_noreplace(parent_fd, stage, final):
    # No unsafe fallback: ordinary rename may overwrite another empty directory.
    libc = ctypes.CDLL(None, use_errno=True)
    try:
        rename = libc.renameat2
    except AttributeError:
        raise Refusal("atomic-noclobber-unavailable") from None
    rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
    rename.restype = ctypes.c_int
    if rename(parent_fd, os.fsencode(stage), parent_fd, os.fsencode(final), 1):
        error = ctypes.get_errno()
        if error in {errno.EEXIST, errno.ENOTEMPTY}:
            raise Refusal("output-collision")
        raise Refusal("atomic-publication-failed")


def publish(output, source_fd, blobs):
    raw = os.fspath(output)
    parent, name = os.path.split(raw)
    safe_relative(name)
    if "/" in name:
        raise Refusal("invalid-output-name")
    parent_fd = physical_dir(parent)
    stage = None
    stage_fd = None
    try:
        st = os.fstat(parent_fd)
        if st.st_uid != os.getuid() or stat.S_IMODE(st.st_mode) & 0o077:
            raise Refusal("output-parent-not-owner-private")
        # Reject outputs inside the source tree, including via renamed ancestors.
        ancestor = os.dup(parent_fd)
        try:
            source_st = os.fstat(source_fd)
            while True:
                here = os.fstat(ancestor)
                if (here.st_dev, here.st_ino) == (source_st.st_dev, source_st.st_ino):
                    raise Refusal("output-inside-source-root")
                up = os.open("..", os.O_RDONLY | DIRECTORY | NOFOLLOW, dir_fd=ancestor)
                above = os.fstat(up)
                if (here.st_dev, here.st_ino) == (above.st_dev, above.st_ino):
                    os.close(up)
                    break
                os.close(ancestor)
                ancestor = up
        finally:
            os.close(ancestor)
        try:
            os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            raise Refusal("output-collision")
        for _ in range(10):
            stage = ".source-stage-" + os.urandom(16).hex()
            try:
                os.mkdir(stage, 0o700, dir_fd=parent_fd)
                break
            except FileExistsError:
                stage = None
        if stage is None:
            raise Refusal("private-staging-collision")
        stage_fd = os.open(stage, os.O_RDONLY | DIRECTORY | NOFOLLOW, dir_fd=parent_fd)
        os.fchmod(stage_fd, 0o700)
        for filename, data in blobs.items():
            fd = os.open(filename, os.O_WRONLY | os.O_CREAT | os.O_EXCL | NOFOLLOW, 0o600, dir_fd=stage_fd)
            try:
                os.fchmod(fd, 0o600)
                with os.fdopen(fd, "wb", closefd=False) as file:
                    file.write(data)
                    file.flush()
                    os.fsync(fd)
            finally:
                os.close(fd)
        os.fsync(stage_fd)
        rename_noreplace(parent_fd, stage, name)
        stage = None
        os.fsync(parent_fd)
    finally:
        # Exact helper-owned staging only; no broad recursive cleanup.
        if stage_fd is not None:
            if stage is not None:
                for filename in blobs:
                    try:
                        os.unlink(filename, dir_fd=stage_fd)
                    except FileNotFoundError:
                        pass
            os.close(stage_fd)
        if stage is not None:
            os.rmdir(stage, dir_fd=parent_fd)
        os.close(parent_fd)


def build(source, output, epoch):
    if isinstance(epoch, bool) or not isinstance(epoch, int) or not 0 <= epoch <= 0xffffffff:
        raise Refusal("invalid-SOURCE_DATE_EPOCH")
    fd = physical_dir(source)
    try:
        files = collect(fd)
        validate_sources(files)
        dependencies = dependency_inventory(files)
        rows = inventory(files)
        identity = digest(json_bytes(rows))
        archive = make_archive(files, epoch)
        # Verify a second read: root inode stays pinned; concurrent edits fail closed.
        if inventory(collect(fd)) != rows:
            raise Refusal("selected-source-changed-during-snapshot")
        manifest = {"schema": 1, "policy": POLICY, "project": "Regressee",
                    "kind": "private-candidate-source-not-release", "git_commit": None,
                    "source_date_epoch": epoch, "source_identity": {"algorithm": "sha256-canonical-inventory-v1", "sha256": identity},
                    "source_files": rows, "archive": {"name": "source.tar.gz", "sha256": digest(archive), "size": len(archive)},
                    "dependencies": dependencies,
                    "release_gate": {"approved": False, "status": "blocked-unreviewed",
                                     "requires": ["source-completeness-and-content-privacy-review", "matching-executable-binding",
                                                  "target-specific-third-party-notices-and-sources", "owner-legal-security-review",
                                                  "source-availability-and-hosting-authorization", "phase6-and-phase7-qualification"]},
                    "omission_policy": ["runtime-build-output", "private-userdata-cache-credentials-logs-keys-sessionstate",
                                        "internal-work-status-research-decisions-and-evidence", "no-symlink-following"],
                    "normalization": {"uid": 0, "gid": 0, "uname": "", "gname": "", "mtime": epoch,
                                      "file_mode": "0644", "script_mode": "0755", "tar": "ustar", "gzip_filename": ""}}
        encoded = json_bytes(manifest)
        checksums = (digest(archive) + "  source.tar.gz\n" + digest(encoded) + "  manifest.json\n").encode()
        publish(output, fd, {"source.tar.gz": archive, "manifest.json": encoded, "SHA256SUMS": checksums})
        return {"source_sha256": identity, "archive_sha256": digest(archive), "manifest_sha256": digest(encoded),
                "file_count": len(rows), "unknown_licenses": dependencies["unknown_count"], "approved": False}
    except (OSError, ValueError, TypeError, UnicodeError, tarfile.TarError):
        raise Refusal("io-or-source-format-failure") from None
    finally:
        os.close(fd)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True)
    parser.add_argument("--output", required=True, help="NEW directory beneath an existing owner-private physical directory")
    parser.add_argument("--epoch", help="override SOURCE_DATE_EPOCH; decimal seconds, required if env unset")
    args = parser.parse_args()
    value = args.epoch if args.epoch is not None else os.environ.get("SOURCE_DATE_EPOCH")
    try:
        if value is None or not re.fullmatch(r"[0-9]+", value):
            raise Refusal("explicit-SOURCE_DATE_EPOCH-required")
        result = build(args.source, args.output, int(value))
        print(json_bytes(result).decode(), end="")
    except Refusal as error:
        print("REFUSED: " + str(error), file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
