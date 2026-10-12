#!/usr/bin/env python3
"""Read-only Phase 4 integrity check; never an input, AT or visual acceptance gate."""
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DOCS = REPO / "docs/research/whole-app-ui-review-20261012"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    receipt = json.loads((DOCS / "phase4-receipt.json").read_text())
    root = Path(receipt["confirmation_root"])
    archive = json.loads((DOCS / "archive-receipt.json").read_text())
    for item in archive["files"]:
        assert sha(REPO / item["archive"]) == item["sha256"], item["archive"]
    corrected = json.loads((DOCS / "phase1-correction-receipt.json").read_text())
    updates = {p["capture"]: p["new_png_sha256"] for p in corrected["corrected_pairs"]}
    originals = json.loads((DOCS / "gallery-manifest.initial.json").read_text())
    for item in originals:
        assert sha(Path(archive["evidence_root"]) / (item["capture"] + ".png")) == updates.get(item["capture"], item["sha256"]), item["capture"]
    matrix = json.loads((root / "whole-app-matrix.json").read_text())
    gallery = json.loads((root / "gallery-manifest.json").read_text())
    assert {p["capture"] for p in matrix} == {p["capture"] for p in gallery} == {p["capture"] for p in receipt["pairs"]}
    for item in receipt["pairs"]:
        name = item["capture"]
        assert sha(root / (name + ".png")) == item["png_sha256"]
        assert sha(root / (name + ".json")) == item["metadata_sha256"]
        meta = json.loads((root / (name + ".json")).read_text())
        assert meta["png_sha256"] == item["png_sha256"]
        assert meta["real_gpui_blade_offscreen"] and meta["synthetic_project_only"]
        assert not meta["os_input_verified"] and not meta["os_presentation_verified"]
        assert meta["main_source_sha256"] == receipt["capture_source_sha256"]["crates/app/src/main.rs"]
        assert meta["task_controls_source_sha256"] == receipt["capture_source_sha256"]["crates/app/src/ui/controls.rs"]
    repair = receipt.get("functional_repair")
    if repair:
        blocked = DOCS / "phase4-receipt.blocked.json"
        assert sha(blocked) == repair["blocked_receipt_sha256"]
        assert json.loads(blocked.read_text()) == {k: v for k, v in receipt.items() if k != "functional_repair"}
    for path, expected in ({} if repair else receipt["final_source_sha256"]).items():
        assert sha(REPO / path) == expected, path
        captured = receipt["capture_source_sha256"][path]
        if captured != expected:
            assert path == "crates/app/src/ui/zoom_select.rs", path
            # Sole post-capture edit is a unit-test expected width, not rendering.
            old = (REPO / path).read_bytes().replace(b"assert_eq!(button.size, size(px(112.), px(CONTROL_HEIGHT)));", b"assert_eq!(button.size, size(px(60.), px(CONTROL_HEIGHT)));")
            assert hashlib.sha256(old).hexdigest() == captured, path
    for path, expected in receipt["log_sha256"].items():
        assert sha(Path(path)) == expected, path
    assert len(receipt["pairs"]) == 38
    assert receipt["visual_gate"] == "BLOCKED: note input width"
    if repair:
        new_root = Path(repair["root"])
        new_matrix = json.loads((new_root / "whole-app-matrix.json").read_text())
        assert {p["capture"] for p in new_matrix} == {p["capture"] for p in repair["pairs"]}
        assert len(new_matrix) == 22
        for item in repair["pairs"]:
            name = item["capture"]
            assert sha(new_root / (name + ".png")) == item["png_sha256"]
            assert sha(new_root / (name + ".json")) == item["metadata_sha256"]
            meta = json.loads((new_root / (name + ".json")).read_text())
            assert meta["png_sha256"] == item["png_sha256"]
            assert meta["real_gpui_blade_offscreen"] and meta["synthetic_project_only"]
            assert not meta["os_input_verified"] and not meta["os_presentation_verified"]
            assert meta["main_source_sha256"] == repair["capture_source_sha256"]["crates/app/src/main.rs"]
            assert meta["task_controls_source_sha256"] == repair["capture_source_sha256"]["crates/app/src/ui/controls.rs"]
        for item in new_matrix:
            assert item["harness_sha256"] == repair["capture_source_sha256"]["crates/app/src/ui_review_matrix.rs"]
        for path, expected in repair["final_source_sha256"].items():
            assert sha(REPO / path) == expected, path
        for path, expected in repair["capture_source_sha256"].items():
            assert sha(REPO / path) == expected, path
        for path, expected in repair["files_sha256"].items():
            assert sha(Path(path)) == expected, path
        assert repair["acceptance"] == "PENDING independent re-verification"
        print(f"PASS integrity only: {len(archive['files'])} immutable archives, {len(originals)} original-root PNGs, 38 blocked pairs + 22 repair pairs; prior FIX preserved, re-verification PENDING")
    else:
        print(f"PASS integrity only: {len(archive['files'])} immutable archives, {len(originals)} original-root PNGs unchanged since Phase 1, 38 pairs, 62 source bindings; visual gate BLOCKED")


if __name__ == "__main__":
    main()
