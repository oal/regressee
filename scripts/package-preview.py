#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Package a native CI binary for testing, without signing or bundling a browser."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile


TARGETS = {
    "x86_64-unknown-linux-gnu": ("linux-x86_64", "regressee-desktop"),
    "x86_64-pc-windows-msvc": ("windows-x86_64", "regressee-desktop.exe"),
}
HELP_FILES = (
    "first-run-guide.md", "auth-and-capture.md", "portable-baseline-archive.md",
    "updates-and-recovery.md", "support-matrix.md", "release-building.md",
    "preview-release-notes.md",
)


def command(root, *args, timeout=None, env=None):
    return subprocess.check_output(
        args, cwd=root, text=True, encoding="utf-8", timeout=timeout, env=env,
    ).strip()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def dependency_notices(root, target):
    """Retain cached license texts for this target's normal/build Cargo graph.

    This is an inventory, not a declaration of legal review or native-library
    completeness. In particular it does not redistribute a development runtime.
    """
    graph = command(
        root, "cargo", "tree", "--locked", "--offline", "-p", "regressee-gui",
        "--features", "ui", "--target", target, "--edges", "normal,build",
        "--prefix", "none", "--format", "{p}",
    )
    packages = set(re.findall(r"^([A-Za-z0-9_-]+) v([^\s]+)", graph, re.MULTILINE))
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))["package"]
    registry = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")) / "registry/src"
    rows, texts = [], {}
    for name, version in sorted(packages):
        if name.startswith("regressee-"):
            continue
        entry = next(p for p in lock if p["name"] == name and p["version"] == version)
        if name == "gpui" and "source" not in entry:
            directory = root / "vendor/gpui"
        else:
            candidates = list(registry.glob(f"*/{name}-{version}"))
            if len(candidates) != 1:
                raise ValueError(f"Expected one cached source for {name} {version}")
            directory = candidates[0]
        meta = tomllib.loads((directory / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        files = {p for p in directory.iterdir()
                 if p.is_file() and p.name.upper().startswith(("LICENSE", "NOTICE", "COPYING"))}
        if meta.get("license-file"):
            files.add(directory / meta["license-file"])
        retained = []
        for file in sorted(files):
            relative = file.relative_to(directory).as_posix()
            if file.is_symlink() or ".." in Path(relative).parts or not file.is_file():
                raise ValueError(f"Invalid license file for {name} {version}")
            path = f"notices/{name}-{version}/{relative}"
            data = file.read_bytes()
            texts[path] = (data, 0o644)
            retained.append({"path": path, "sha256": hashlib.sha256(data).hexdigest()})
        rows.append({
            "name": name, "version": version, "declared_license": meta.get("license"),
            "cargo_checksum": entry.get("checksum"), "texts": retained,
            "status": "unreviewed" if retained else "missing-license-text",
        })
    if not rows:
        raise ValueError("Dependency inventory is empty")
    inventory = {
        "target": target, "features": ["ui"], "edges": ["normal", "build"],
        "packages": rows, "release_approved": False, "native_inventory_complete": False,
    }
    return inventory, texts


def package(root, output, target, revision):
    platform, executable = TARGETS[target]
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("A full source commit SHA is required")
    if command(root, "git", "rev-parse", "HEAD") != revision:
        raise ValueError("Source checkout does not match the requested commit")
    binary = root / "target" / target / "release" / executable
    data = binary.read_bytes()
    if not data:
        raise ValueError("The application binary is empty")
    version = tomllib.loads((root / "crates/gui/Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    # Metadata is data-free in current builds. Isolate and bound an accidentally
    # supplied older executable too, before it can be accepted for packaging.
    with tempfile.TemporaryDirectory(prefix="regressee-preview-check-") as isolated:
        build = json.loads(command(root, str(binary), "--build-info", timeout=15,
                                   env={**os.environ, "REGRESSEE_DEV_DATA": isolated}))
    if build != {"version": version, "features": {
            "ui": True, "developer-tools": False, "test-support": False,
            "ui-inspection": False, "ui-renderer-capture": False}}:
        raise ValueError("Preview requires the matching production UI build without developer/test features")
    inventory, notices = dependency_notices(root, target)
    files = {
        executable: (data, 0o755),
        "build-info.json": (json_bytes({
            "kind": "unsigned-test-preview", "version": version,
            "compiled_features": build["features"],
            "source_commit": revision, "target": target, "features": ["ui"],
            "rustc": command(root, "rustc", "--version"),
            "binary_sha256": hashlib.sha256(data).hexdigest(),
            "runtime_bundled": False, "signed": False, "installer": False,
        }), 0o644),
        "license-inventory.json": (json_bytes(inventory), 0o644),
        **notices,
    }
    for name in ("LICENSE", "THIRD_PARTY_NOTICES.md", "README.md"):
        files[name] = ((root / name).read_bytes(), 0o644)
    for name in HELP_FILES:
        files[f"help/{name}"] = ((root / "docs" / name).read_bytes(), 0o644)
    prefix = f"regressee-desktop-{platform}"
    # Fresh output only: do not accidentally upload a stale binary or archive.
    output.mkdir(parents=True, exist_ok=False)
    if executable.endswith(".exe"):
        (output / f"{prefix}.exe").write_bytes(data)
        with zipfile.ZipFile(output / f"{prefix}.zip", "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, (content, mode) in sorted(files.items()):
                info = zipfile.ZipInfo(f"{prefix}/{name}")
                info.create_system = 3
                info.external_attr = (stat.S_IFREG | mode) << 16
                archive.writestr(info, content, compress_type=zipfile.ZIP_DEFLATED)
    else:
        with tarfile.open(output / f"{prefix}.tar.gz", "w:gz") as archive:
            for name, (content, mode) in sorted(files.items()):
                info = tarfile.TarInfo(f"{prefix}/{name}")
                info.size, info.mode = len(content), mode
                archive.addfile(info, io.BytesIO(content))
    return sorted(p.name for p in output.iterdir())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    print("\n".join(package(root, args.output, args.target, args.revision)))
