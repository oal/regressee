#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Build unsigned CLI-only / combined desktop archives from trusted native builds.

No signing, runtime bundling, upload or publication. Metadata probing executes
the caller's trusted binaries with isolated paths and a deadline, NOT an OS sandbox.
"""
import argparse
import gzip
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

import source_release as source

TARGETS = {
    "x86_64-unknown-linux-gnu": ("linux-x86_64", ""),
    "x86_64-pc-windows-msvc": ("windows-x86_64", ".exe"),
}
PRODUCTS = {"cli": ("regressee",), "desktop": ("regressee", "regressee-desktop")}
HELP_FILES = tuple(sorted(source.USER_DOCS))
FEATURES = {"ui": False, "developer-tools": False, "test-support": False,
            "ui-inspection": False, "ui-renderer-capture": False}


def command(root, *args, timeout=120, env=None):
    return subprocess.check_output(args, cwd=root, text=True, encoding="utf-8",
                                   timeout=timeout, env=env).strip()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def physical(path):
    path = Path(os.path.abspath(path))
    if path.resolve(strict=True) != path or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("Physical, non-symlink paths are required")
    return path


def ordinary(path, limit, single_link=True):
    physical(path)
    info = path.stat()
    if not stat.S_ISREG(info.st_mode) or (single_link and info.st_nlink != 1) or not 0 < info.st_size <= limit:
        raise ValueError("Expected a nonempty bounded ordinary file (single-link for sources/notices)")
    data = path.read_bytes()
    after = path.stat()
    if len(data) != info.st_size or any(getattr(after, key) != getattr(info, key) for key in
                                      ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns", "st_mode", "st_nlink")):
        raise ValueError("Input changed during read")
    return data


def selected_sources(root):
    """Portable preview counterpart of the source helper's reviewed allowlist.

    Never walk excluded private directories. This is a trusted-checkout build
    tool, not fd-pinned defense against concurrent hostile same-user filesystem edits.
    """
    files = {}
    cases = set()
    total = 0

    def walk(directory, prefix=""):
        nonlocal total
        for path in sorted(directory.iterdir()):
            rel = prefix + path.name
            action = source.classify(rel, path.is_dir())
            if action == "omit":
                continue
            source.safe_relative(rel)
            if rel.lower() in cases or path.is_symlink():
                raise ValueError("Source links / portable path collisions are forbidden")
            cases.add(rel.lower())
            if action == "walk":
                walk(path, rel + "/")
            else:
                data = ordinary(path, source.MAX_FILE)
                if re.search(rb"-----BEGIN (?:[A-Z0-9 ]+ )?PRIVATE KEY-----", data):
                    raise ValueError("Private key material in selected source")
                if rel in source.FIXTURE_ARCHIVES and digest(data) != source.FIXTURE_ARCHIVES[rel]:
                    raise ValueError("Reviewed archive fixture changed")
                total += len(data)
                if total > source.MAX_TOTAL or len(files) >= source.MAX_FILES:
                    raise ValueError("Source inventory budget exceeded")
                mode = 0o755 if path.suffix in {".py", ".sh"} else 0o644
                files[rel] = (data, mode)
    walk(physical(root))
    source.validate_sources(files)
    if not source.REQUIRED <= files.keys():
        raise ValueError("Required root source is missing")
    return dict(sorted(files.items()))


def dependency_notices(root, target, product):
    queries = [["cargo", "tree", "--locked", "--offline", "-p", "regressee-cli"]]
    if product == "desktop":
        queries.append(["cargo", "tree", "--locked", "--offline", "-p", "regressee-gui", "--features", "ui"])
    graph = "\n".join(command(root, *args, "--target", target, "--edges", "normal,build",
                              "--prefix", "none", "--format", "{p}") for args in queries)
    packages = set(re.findall(r"^([A-Za-z0-9_-]+) v([^\s]+)", graph, re.MULTILINE))
    if not packages or (product == "cli" and any(n in {"gpui", "regressee-gui"} for n, _ in packages)):
        raise ValueError("Empty or GUI-contaminated CLI dependency graph")
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
        physical(directory)
        meta = tomllib.loads((directory / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        paths = {p for p in directory.iterdir()
                 if p.name.upper().startswith(("LICENSE", "NOTICE", "COPYING")) and not p.is_dir()}
        if meta.get("license-file"):
            source.safe_relative(meta["license-file"])
            paths.add(directory / meta["license-file"])
        retained = []
        for path in sorted(paths):
            relative = path.relative_to(directory).as_posix()
            source.safe_relative(relative)
            data = ordinary(path, 4 * 1024 * 1024)
            name_in_archive = f"notices/{name}-{version}/{relative}"
            texts[name_in_archive] = (data, 0o644)
            retained.append({"path": name_in_archive, "sha256": digest(data)})
        rows.append({"name": name, "version": version, "declared_license": meta.get("license"),
                     "cargo_checksum": entry.get("checksum"), "texts": retained,
                     "status": "unreviewed" if retained else "missing-license-text"})
    inventory = {"target": target, "product": product,
                 "packages_built": ["regressee-cli"] + (["regressee-gui"] if product == "desktop" else []),
                 "features": ["regressee-gui/ui"] if product == "desktop" else [],
                 "edges": ["normal", "build"], "packages": rows,
                 "release_approved": False, "native_inventory_complete": False}
    validate_inventory(inventory, texts, target, product)
    return inventory, texts


def validate_inventory(inventory, texts, target, product):
    expected_packages = ["regressee-cli"] + (["regressee-gui"] if product == "desktop" else [])
    if (inventory.get("target") != target or inventory.get("product") != product
            or inventory.get("features") != (["regressee-gui/ui"] if product == "desktop" else [])
            or inventory.get("packages_built") != expected_packages
            or inventory.get("edges") != ["normal", "build"] or not inventory.get("packages")
            or inventory.get("release_approved") is not False
            or inventory.get("native_inventory_complete") is not False):
        raise ValueError("Missing or mismatched dependency inventory")
    included = set()
    identities = set()
    for row in inventory["packages"]:
        identity = (row["name"], row["version"])
        if identity in identities or (product == "cli" and row["name"] in {"gpui", "regressee-gui"}):
            raise ValueError("Duplicate or GUI-contaminated CLI dependency inventory")
        identities.add(identity)
        if row["status"] != ("unreviewed" if row["texts"] else "missing-license-text"):
            raise ValueError("License review status must remain truthful")
        for text in row["texts"]:
            path = source.safe_relative(text["path"])
            if not path.startswith(f"notices/{row['name']}-{row['version']}/") or path in included:
                raise ValueError("Invalid notice path")
            if path not in texts or digest(texts[path][0]) != text["sha256"] or texts[path][1] != 0o644:
                raise ValueError("Missing or mismatched license text")
            included.add(path)
    if included != texts.keys():
        raise ValueError("Uninventoried notice data")


def probe(root, binary, name, version):
    with tempfile.TemporaryDirectory(prefix="regressee-preview-check-") as isolated:
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("REGRESSEE_", "XDG_")) and k not in {"DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS"}}
        env.update({"HOME": isolated, "USERPROFILE": isolated, "APPDATA": isolated,
                    "LOCALAPPDATA": isolated, "XDG_CONFIG_HOME": isolated,
                    "XDG_DATA_HOME": isolated, "XDG_CACHE_HOME": isolated,
                    "REGRESSEE_DEV_DATA": isolated})
        build = json.loads(command(root, str(binary), "--build-info", timeout=15, env=env))
        if list(Path(isolated).iterdir()):
            raise ValueError("Build metadata probe unexpectedly wrote application data")
    if (set(build) != {"build_info_schema", "executable", "version", "engine_protocol", "cli_schema", "features"}
            or type(build["build_info_schema"]) is not int or build["build_info_schema"] != 1
            or build["executable"] != name or build["version"] != version
            or type(build["cli_schema"]) is not int or build["cli_schema"] != 1 or type(build["engine_protocol"]) is not int
            or build["engine_protocol"] <= 0
            or not isinstance(build["features"], dict) or any(type(v) is not bool for v in build["features"].values())
            or build["features"] != {**FEATURES, "ui": name == "regressee-desktop"}):
        raise ValueError("Requires matching production binaries without developer/test features")
    return build


def tar_bytes(prefix, files, epoch):
    stream = io.BytesIO()
    with gzip.GzipFile(filename="", mode="wb", fileobj=stream, mtime=epoch) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as archive:
            for name, (data, mode) in sorted(files.items()):
                source.safe_relative(name)
                entry = tarfile.TarInfo(f"{prefix}/{name}")
                entry.size, entry.mode, entry.mtime = len(data), mode, epoch
                archive.addfile(entry, io.BytesIO(data))
    return stream.getvalue()


def zip_bytes(prefix, files):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, (data, mode) in sorted(files.items()):
            source.safe_relative(name)
            info = zipfile.ZipInfo(f"{prefix}/{name}", date_time=(2000, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | mode) << 16
            archive.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED)
    return stream.getvalue()


def package(root, output, target, revision, product="all", binary_dir=None, allow_dirty=False, build_receipt=None):
    root = physical(root)
    if not re.fullmatch(r"[0-9a-f]{40}", revision) or command(root, "git", "rev-parse", "HEAD") != revision:
        raise ValueError("A matching full source commit SHA is required")
    dirty = bool(command(root, "git", "status", "--porcelain", "--untracked-files=normal"))
    if dirty and not allow_dirty:
        raise ValueError("Dirty checkout: use --allow-dirty only for explicitly labelled local qualification")
    epoch = int(command(root, "git", "show", "-s", "--format=%ct", revision))
    if not 0 <= epoch <= 0xffffffff:
        raise ValueError("Source epoch outside archive bounds")
    output = Path(os.path.abspath(output))
    physical(output.parent)
    if output.exists() or output.is_symlink():
        raise FileExistsError("Fresh output directory required")
    if output.is_relative_to(root) and not output.is_relative_to(root / "artifacts"):
        raise ValueError("Output inside source is allowed only under artifacts/")
    selected = selected_sources(root)
    rows = source.inventory(selected)
    identity = digest(source.json_bytes(rows))
    source_manifest = {"schema": 1, "source_commit": revision, "dirty": dirty,
                       "source_inventory_sha256": identity, "files": rows,
                       "omissions": ["private-runtime-build-data-credentials", "internal-docs-reviews-evidence-design-context"],
                       "approved": False}
    blobs = {"regressee-source.tar.gz": tar_bytes("regressee-source", selected, epoch),
             "regressee-source.json": json_bytes(source_manifest)}
    platform, suffix = TARGETS[target]
    binary_dir = physical(binary_dir or root / "target" / target / "release")
    products = tuple(PRODUCTS) if product == "all" else (product,)
    binaries = {}
    versions = set()
    protocols = set()
    for name in sorted({name for p in products for name in PRODUCTS[p]}):
        path = binary_dir / (name + suffix)
        # Cargo normally hardlinks its top-level executable to deps/. Reading
        # that trusted build output is safe; archive entries are fresh bytes.
        data = ordinary(path, 512 * 1024 * 1024, single_link=False)
        version = tomllib.loads(selected[f"crates/{'cli' if name == 'regressee' else 'gui'}/Cargo.toml"][0].decode())["package"]["version"]
        build = probe(root, path, name, version)
        if ordinary(path, 512 * 1024 * 1024, single_link=False) != data:
            raise ValueError("Binary changed during metadata probe")
        binaries[name] = (data, build)
        versions.add(version)
        protocols.add(build["engine_protocol"])
    if len(versions) != 1 or len(protocols) != 1:
        raise ValueError("CLI and desktop version/protocol must match")
    # Bind to the source protocol too, not only to two mutually stale binaries.
    protocol_source = selected["crates/app/src/protocol.rs"][0].decode()
    match = re.search(r"pub const VERSION: u32 = (\d+);", protocol_source)
    if not match or protocols != {int(match[1])}:
        raise ValueError("Binary protocol does not match selected source")
    if build_receipt is not None:
        expected = {name + suffix: digest(data) for name, (data, _) in binaries.items()}
        if (build_receipt.get("source_inventory_sha256") != identity
                or build_receipt.get("target") != target or build_receipt.get("product") != product
                or build_receipt.get("binaries") != expected):
            raise ValueError("Build receipt does not bind these sources / binaries / product")
    compiler = command(root, "rustc", "--version")
    for p in products:
        inventory, notices = dependency_notices(root, target, p)
        validate_inventory(inventory, notices, target, p)
        files = {**notices}
        info = {"schema": 2, "kind": "unsigned-test-preview", "product": p, "target": target,
                "version": next(iter(versions)), "source_commit": revision, "dirty": dirty,
                "source_inventory_sha256": identity, "source_archive_sha256": digest(blobs["regressee-source.tar.gz"]),
                "rustc": compiler, "runtime_bundled": False, "signed": False, "installer": False,
                "source_binding": "frozen-before-and-after-build" if build_receipt else "caller-supplied-binaries-no-compile-attestation",
                "release_approved": False, "executables": {}}
        for name in PRODUCTS[p]:
            data, build = binaries[name]
            files[name + suffix] = (data, 0o755)
            info["executables"][name + suffix] = {"sha256": digest(data), "bytes": len(data), "build": build}
        files["build-info.json"] = (json_bytes(info), 0o644)
        files["source-inventory.json"] = (blobs["regressee-source.json"], 0o644)
        if build_receipt is not None:
            files["build-receipt.json"] = (json_bytes(build_receipt), 0o644)
        files["license-inventory.json"] = (json_bytes(inventory), 0o644)
        for name in ("LICENSE", "THIRD_PARTY_NOTICES.md", "README.md"):
            files[name] = selected[name]
        cli_command = ".\\regressee.exe" if suffix else "./regressee"
        desktop_note = ("Keep both executables together; launch regressee-desktop" + suffix + " for the GUI.\n"
                        if p == "desktop" else "CLI-only: no GPUI or desktop executable is included.\n")
        files["README.md"] = ((f"# Regressee {p} — unsigned test preview\n\n" + desktop_note
            + f"\nRun `{cli_command} --help`, `{cli_command} --build-info` and `{cli_command} --json capabilities`.\n"
            + "Use explicit --data-dir / --cache-dir for isolated work. Headless capture is public-sites-only.\n\n"
            + "[Terminal guide](help/cli.md) · [Install/build requirements](help/release-building.md) · "
            + "[Support limits](help/support-matrix.md) · [Portable skill](skills/regressee/SKILL.md) · "
            + "[Manual skill installation](help/agent-skill.md)\n\n"
            + "Browser libraries/fonts and its default sandbox remain prerequisites. Do not disable protections.\n"
            + "No runtime bundle, installer, signing, upload or native/clean-machine/release qualification is implied.\n"
            + "Internal status/research links in source help are intentionally not distributed; the public support matrix governs claims.\n\n"
            + "build-info.json binds each executable, source-inventory.json the selected sources, and build-receipt.json "
            + "(when supplied by the integrated builder) the before/after build. SHA256SUMS detects corruption, not publisher authenticity.\n"
            + "License/native inventory remains unreviewed; see LICENSE, THIRD_PARTY_NOTICES.md and license-inventory.json.\n").encode(), 0o644)
        for name in HELP_FILES:
            files["help/" + name] = selected["docs/" + name]
        for name in source.SKILL_FILES:
            files[name] = selected[name]
        files["SHA256SUMS"] = ("".join(f"{digest(data)}  {name}\n" for name, (data, _) in sorted(files.items())).encode(), 0o644)
        prefix = f"regressee-{p}-{platform}"
        extension = ".zip" if suffix else ".tar.gz"
        blobs[prefix + extension] = zip_bytes(prefix, files) if suffix else tar_bytes(prefix, files, epoch)
    if source.inventory(selected_sources(root)) != rows:
        raise ValueError("Selected source changed across packaging")
    blobs[f"SHA256SUMS-{platform}"] = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(blobs.items())).encode()
    output.mkdir(mode=0o700, exist_ok=False)
    for name, data in sorted(blobs.items()):
        with (output / name).open("xb") as file:
            file.write(data)
    return sorted(blobs)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--product", choices=(*PRODUCTS, "all"), default="all")
    parser.add_argument("--revision", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--binary-dir", type=Path, help="Explicit trusted native build directory; defaults to target/TARGET/release")
    parser.add_argument("--allow-dirty", action="store_true", help="Local qualification only: bind selected working-tree hashes, never pretend they are the base commit")
    args = parser.parse_args()
    print("\n".join(package(Path(__file__).resolve().parent.parent, args.output, args.target,
                          args.revision, args.product, args.binary_dir, args.allow_dirty)))
