#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Verify the bounded Phase 1 correction against immutable original evidence."""
import argparse
import hashlib
import json
import pathlib
import struct

parser = argparse.ArgumentParser()
parser.add_argument("root", type=pathlib.Path)
args = parser.parse_args()
root = args.root.resolve()
repo = pathlib.Path(__file__).resolve().parents[1]
archive = repo / "docs/research/whole-app-ui-review-20261012"
superseded = root / "superseded-phase1-20261012"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text())


original = load(archive / "archive-receipt.json")
for record in original["files"]:
    assert sha(repo / record["archive"]) == record["sha256"], record["archive"]
product = {p: h for p, h in original["source_sha256_at_plan_creation"].items()
           if p.startswith("crates/") and not p.endswith(("ui_review_matrix.rs", "ui_renderer_capture.rs"))}
for path, expected in product.items():
    assert sha(repo / path) == expected, path
preserved = load(superseded / "preservation.json")
for record in preserved["affected"]:
    assert sha(superseded / record["path"]) == record["sha256"], record["path"]
for path, expected in preserved["unaffected_png_sha256"].items():
    assert sha(root / path) == expected, path
for name, archived in (("whole-app-matrix.json", "whole-app-matrix.initial.json"),
                       ("gallery-manifest.json", "gallery-manifest.initial.json"),
                       ("production-source-binding.json", "production-source-binding.original.json")):
    assert sha(superseded / name) == sha(archive / archived), name
matrix = load(root / "whole-app-matrix.json")
gallery = load(root / "gallery-manifest.json")
assert len(matrix) == 142 and len(gallery) == 182
assert len({entry["capture"] for entry in gallery}) == 182
affected = {pathlib.Path(r["path"]).stem for r in preserved["affected"]}
assert {e["capture"] for e in matrix if not e.get("retained_from_same_batch")} == affected
corrections = []
for entry in gallery:
    name = entry["capture"]
    png = root / f"{name}.png"
    metadata = load(root / f"{name}.json")
    assert sha(png) == entry["sha256"] == metadata["png_sha256"], name
    assert struct.unpack(">II", png.read_bytes()[16:24]) == (entry["width"], entry["height"])
    for flag in ("real_gpui_blade_offscreen", "real_linux_fonts", "synthetic_project_only"):
        assert metadata[flag] is True, (name, flag)
    assert metadata["os_input_verified"] is False
    assert metadata["os_presentation_verified"] is False
    if name in affected:
        item = next(e for e in matrix if e["capture"] == name)
        assert item["visibility_assertions"], name
        assert item["harness_sha256"] == sha(repo / "crates/app/src/ui_review_matrix.rs")
        corrections.append({"capture": name,
                            "old_png_sha256": sha(superseded / png.name),
                            "new_png_sha256": sha(png),
                            "old_metadata_sha256": sha(superseded / f"{name}.json"),
                            "new_metadata_sha256": sha(root / f"{name}.json"),
                            "pixels_changed": sha(png) != sha(superseded / png.name),
                            "visibility_assertions": item["visibility_assertions"]})
binding = {p: sha(repo / p) for p in load(archive / "production-source-binding.original.json")}
for path, expected in load(archive / "production-source-binding.original.json").items():
    if path != "crates/app/src/ui_review_matrix.rs":
        assert binding[path] == expected, path
binding["scripts/ui-review-phase1-receipt.py"] = sha(pathlib.Path(__file__).resolve())
artifacts = ["whole-app-matrix.json", "gallery-manifest.json", "index.html",
             "phase1-renderer.log", "phase1-renderer-final.log",
             "phase1-bin-tests.log", "phase1-gallery.log"]
receipt = {"date": "2026-10-12", "evidence_root": str(root),
           "superseded_directory": str(superseded), "corrected_pairs": corrections,
           "original_archive_records_verified": len(original["files"]),
           "unaffected_pngs_unchanged": len(preserved["unaffected_png_sha256"]),
           "product_ui_sha256_unchanged": product, "current_source_binding": binding,
           "gallery_entries": len(gallery), "whole_app_entries": len(matrix),
           "evidence_artifact_sha256": {p: sha(root / p) for p in artifacts},
           "contact_sheet_sha256": {p.name: sha(p) for p in sorted((root / "contact-sheets").glob("*.png"))},
           "original_inspection_batches": 1, "corrective_confirmation_batches": 0,
           "coverage": {"E1": "Blank unbound creation form, both sizes",
                        "E2": "Beginning of technical provenance and first complete advanced parameter; own bounded scroll body, both sizes. Not all lower content inspected.",
                        "E3": "Narrow Overlay/Heatmap selected through mounted Analysis controls then flyout closed; zoom popup and fit/actual options visible at both sizes"},
           "qualification": "Synthetic Linux GPUI/Blade offscreen only; no OS input, accessibility, Windows/macOS, real site, bulk approval, throughput or release qualification. Original A/B assess the original batch."}
output = archive / "phase1-correction-receipt.json"
output.write_text(json.dumps(receipt, indent=2) + "\n")
(root / "phase1-correction-receipt.json").write_bytes(output.read_bytes())
print(json.dumps({"receipt": str(output), "corrected_pairs": len(corrections),
                  "changed_pngs": sum(c["pixels_changed"] for c in corrections),
                  "unaffected_pngs_unchanged": receipt["unaffected_pngs_unchanged"]}, indent=2))
