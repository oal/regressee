#!/usr/bin/env python3
"""Integrity only; a blocked Phase 5 gate is never upgraded by hashing."""
import hashlib
import json
from pathlib import Path

repo = Path(__file__).resolve().parents[1]
docs = repo / "docs/research/whole-app-ui-review-20261012"
receipt = json.loads((docs / "phase5-receipt.json").read_text())


def check(path, expected):
    assert hashlib.sha256(path.read_bytes()).hexdigest() == expected, str(path)


archive = json.loads((docs / "archive-receipt.json").read_text())
for entry in archive["files"]:
    check(repo / entry["archive"], entry["sha256"])
repair = receipt.get("functional_repair")
if repair:
    check(docs / "phase5-receipt.blocked.json", repair["blocked_receipt_sha256"])
    check(docs / "phase5-verification.md", repair["original_fix_sha256"])
    historical_binding = json.loads((Path(receipt["confirmation_root"]) / "source-binding.json").read_text())
    assert historical_binding == receipt["source_sha256"]
    for path, digest in repair["final_source_sha256"].items():
        check(repo / path, digest)
    changed = [p for p in repair["capture_source_sha256"] if repair["capture_source_sha256"][p] != repair["final_source_sha256"][p]]
    assert changed == ["crates/app/src/phase5_repair_tests.rs"]
    check(Path(repair["root"]) / "phase5_repair_tests.capture.rs", repair["capture_source_sha256"][changed[0]])
    for path, digest in repair["files_sha256"].items():
        check(Path(path), digest)
else:
    for path, digest in receipt["source_sha256"].items():
        check(repo / path, digest)
for path, digest in receipt["artifacts_sha256"].items():
    check(Path(path), digest)
for path, digest in receipt["original_png_sha256"].items():
    check(Path(path), digest)
root = Path(receipt["confirmation_root"])
matrix = json.loads((root / "whole-app-matrix.json").read_text())
assert len(matrix) == 38
for entry in matrix:
    metadata = json.loads((root / (entry["capture"] + ".json")).read_text())
    check(root / (entry["capture"] + ".png"), metadata["png_sha256"])
    assert metadata["real_gpui_blade_offscreen"] is True
    assert metadata["os_input_verified"] is False
    assert metadata["main_source_sha256"] == receipt["source_sha256"]["crates/app/src/main.rs"]
    assert entry["visibility_assertions"]
for filename in receipt["confirmation_originals_read"]:
    assert (root / filename).is_file()
if repair:
    repaired_root = Path(repair["root"])
    repaired_matrix = json.loads((repaired_root / "whole-app-matrix.json").read_text())
    assert len(repaired_matrix) == 6
    for entry in repaired_matrix:
        metadata = json.loads((repaired_root / (entry["capture"] + ".json")).read_text())
        check(repaired_root / (entry["capture"] + ".png"), metadata["png_sha256"])
        assert metadata["main_source_sha256"] == repair["capture_source_sha256"]["crates/app/src/main.rs"]
        assert metadata["real_gpui_blade_offscreen"] and not metadata["os_input_verified"]
        clips = [v for v in entry["visibility_assertions"] if "painted_field_with_error_help" in v or v.get("selector") == "check-Guided(3)"]
        assert len(clips) == 1
        for v in clips:
            assert v["fully_visible"] and v["y"] >= v["clip_y"] and v["y"] + v["height"] <= v["clip_y"] + v["clip_height"]
            assert v["x"] >= v["clip_x"] and v["x"] + v["width"] <= v["clip_x"] + v["clip_width"]
    assert repair["external_probe"]["passed"] == 2 and repair["packages"]["passed"] == 308
print(f"Integrity PASS: {len(archive['files'])} immutable archives, "
      f"{len(receipt['original_png_sha256'])} original PNGs, 38 confirmation pairs, "
      + ("6 repair pairs / 66 current source bindings. Re-verification PENDING." if repair else f"{len(receipt['source_sha256'])} source bindings. Phase 5 remains BLOCKED."))
