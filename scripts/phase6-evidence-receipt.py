#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Freeze/check Phase 6 local native evidence, source, logs and immutable archives."""
import argparse
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
REPORT = REPO / "docs/research/whole-app-ui-review-20261012"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def collect(root):
    archive = json.loads((REPORT / "archive-receipt.json").read_text())
    for item in archive["files"]:
        assert digest(REPO / item["archive"]) == item["sha256"], item["archive"]
    old = json.loads((REPORT / "phase5-receipt.json").read_text())["original_png_sha256"]
    original_root = Path(archive["evidence_root"])
    for filename, expected in old.items():
        assert digest(original_root / filename) == expected, filename
    batches = {}
    for name, directory in [("inspection", root), ("confirmation", root / "confirmation")]:
        pairs = []
        for png in sorted(directory.glob("*.png")):
            if not png.name.startswith(("all-", "task-")):
                continue
            metadata = png.with_suffix(".json")
            data = json.loads(metadata.read_text())
            assert digest(png) == data["png_sha256"], png
            assert data["real_gpui_blade_offscreen"] and not data["os_input_verified"]
            if name == "confirmation":
                assert data["main_source_sha256"] == digest(REPO / "crates/gui/src/main.rs")
                assert data["task_source_sha256"] == digest(REPO / "crates/gui/src/task_ux.rs")
            pairs.append({"capture": png.stem, "png": digest(png), "metadata": digest(metadata)})
        assert len(pairs) == 34, (name, len(pairs))
        matrix = json.loads((directory / "whole-app-matrix.json").read_text())
        assert len(matrix) == 28
        tail = [entry for entry in matrix if entry["state"] in ("history-bottom", "history-baselines")]
        if name == "confirmation":
            assert len(tail) == 4
            for entry in tail:
                assert any(a.get("selected_last_row") and a.get("full_width") and a.get("fully_visible") and a.get("actual_inner_scroll_owner") for a in entry["visibility_assertions"]), entry["capture"]
        batches[name] = {"pairs": pairs, "matrix": digest(directory / "whole-app-matrix.json"), "gallery": digest(directory / "gallery-manifest.json"), "sheets": {p.name: digest(p) for p in sorted((directory / "contact-sheets").glob("*.png"))}}
    sources = {str(p.relative_to(REPO)): digest(p) for p in sorted((REPO / "crates").glob("*/src/**/*.rs"))}
    for p in [REPO / "crates/app/tests/support/approval_fixture.rs", Path(__file__)]:
        sources[str(p.relative_to(REPO))] = digest(p)
    logs = {str(p): digest(p) for p in sorted(root.parent.glob("phase6-*.log"))}
    return {"phase": 6, "base": "a37cf8c", "status": "implementation complete; independent verification pending", "root": str(root), "packages": {"passed": 311, "failed": 0, "ignored": 40}, "immutable_archive_files": len(archive["files"]), "original_root_pngs_unchanged": len(old), "sources": sources, "batches": batches, "logs": logs, "limits": ["synthetic local evidence; not live websites", "Linux GPUI/Blade offscreen, not OS presentation/input/accessibility", "one inspection and one confirmation; no release or throughput claim", "native complete task uses existing 148-target presentation-only fixture", "initial corrupt project read cannot name an unread target; retained validated metadata failure routes tested"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    path = REPORT / "phase6-receipt.json"
    current = collect(args.root)
    if args.write:
        path.write_text(json.dumps(current, indent=2) + "\n")
    else:
        assert current == json.loads(path.read_text()), "receipt/source/evidence/log drift"
    print(f"PASS: {current['immutable_archive_files']} archives, {len(current['sources'])} sources, 34 inspection + 34 confirmation pairs; four actual-inner-owner tail assertions")


if __name__ == "__main__":
    main()
