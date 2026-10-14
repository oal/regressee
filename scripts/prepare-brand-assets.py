#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Prepare embedded UI logos without modifying the user-supplied originals.

Developer-only tool; requires Pillow. Crop shared transparent padding, preserving
the eye artwork and its alpha, then downsample for small HiDPI wordmarks.
"""
import hashlib
import json
from pathlib import Path

from PIL import Image
from PIL.PngImagePlugin import PngInfo


ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
CROP = (40, 280, 1216, 1000)
SIZE = (294, 180)


def main():
    output = ASSETS / "brand"
    output.mkdir(exist_ok=True)
    records = []
    for source, target, background in [
        ("logo.png", "logo-light.png", "light"),
        ("logo-dark.png", "logo-dark.png", "dark"),
    ]:
        path = ASSETS / source
        source_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        metadata = PngInfo()
        metadata.add_text(
            "impeccable:prompt",
            f"Origin: user-supplied Regressee logo assets/{source}, 2026-10-14. "
            f"Source SHA-256 {source_hash}. Crop {CROP}, Lanczos downsample to "
            f"{SIZE[0]}x{SIZE[1]}; alpha retained, no recoloring. Original unchanged.",
        )
        with Image.open(path) as image:
            image.crop(CROP).resize(SIZE, Image.Resampling.LANCZOS).save(
                output / target, optimize=True, pnginfo=metadata
            )
        records.append({
            "file": target,
            "origin": "User-supplied Regressee logo, 2026-10-14",
            "source": str(path.relative_to(ROOT)),
            "source_sha256": source_hash,
            "sha256": hashlib.sha256((output / target).read_bytes()).hexdigest(),
            "background": background,
            "crop_xyxy": CROP,
            "size_px": SIZE,
            "transform": "Shared padding crop and Lanczos downsample; no recoloring",
        })
    (output / "provenance.json").write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
