#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Freeze selected sources, build exact production products, then package locally.

Linux/Windows x86_64 only. No workflow dispatch/signing/upload/publication. This
trusted-checkout receipt is reproducible build provenance, not signed attestation,
clean-machine reproducibility, native qualification or legal approval.
"""
import argparse
import importlib.util
import os
from pathlib import Path
import subprocess

spec = importlib.util.spec_from_file_location("package_preview", Path(__file__).with_name("package-preview.py"))
preview = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preview)


def build(root, output, target, product="all", allow_dirty=False, offline=False):
    root = preview.physical(root)
    revision = preview.command(root, "git", "rev-parse", "HEAD")
    dirty = bool(preview.command(root, "git", "status", "--porcelain", "--untracked-files=normal"))
    if dirty and not allow_dirty:
        raise ValueError("Refusing a dirty build; --allow-dirty is labelled local qualification only")
    rows = preview.source.inventory(preview.selected_sources(root))
    identity = preview.digest(preview.source.json_bytes(rows))
    commands = [["cargo", "build", "--release", "--locked", "-p", "regressee-cli", "--bin", "regressee"]]
    if product in {"desktop", "all"}:
        commands.append(["cargo", "build", "--release", "--locked", "-p", "regressee-gui",
                         "--features", "ui", "--bin", "regressee-desktop"])
    # Separate invocations keep GUI feature unification out of the CLI binary
    # and make each executable's inventory describe its actual build closure.
    for args in commands:
        args += ["--target", target]
        if offline:
            args += ["--offline"]
        subprocess.run(args, cwd=root, check=True)
        if preview.source.inventory(preview.selected_sources(root)) != rows:
            raise ValueError("Selected source changed across build")
    target_dir = Path(os.environ.get("CARGO_TARGET_DIR", root / "target"))
    if not target_dir.is_absolute():
        target_dir = root / target_dir
    binary_dir = preview.physical(target_dir / target / "release")
    suffix = preview.TARGETS[target][1]
    names = {name for p in (preview.PRODUCTS if product == "all" else (product,)) for name in preview.PRODUCTS[p]}
    receipt = {"schema": 1, "target": target, "product": product, "source_commit": revision,
               "dirty": dirty, "source_inventory_sha256": identity, "commands": commands,
               "rustc": preview.command(root, "rustc", "--version"),
               "binaries": {name + suffix: preview.digest(preview.ordinary(binary_dir / (name + suffix), 512 * 1024 * 1024, single_link=False))
                            for name in sorted(names)}, "signed_attestation": False,
               "clean_machine_qualified": False}
    return preview.package(root, output, target, revision, product, binary_dir, allow_dirty, receipt)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=preview.TARGETS)
    parser.add_argument("--product", choices=(*preview.PRODUCTS, "all"), default="all")
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--allow-dirty", action="store_true")
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    print("\n".join(build(Path(__file__).resolve().parent.parent, args.output, args.target,
                          args.product, args.allow_dirty, args.offline)))
