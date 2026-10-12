#!/usr/bin/env python3
"""Index/freeze/check Phase 7 qualification, without upgrading historical gates."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REPO = Path(__file__).resolve().parents[1]
DOCS = REPO / "docs/research/whole-app-ui-review-20261012"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def index(directory):
    matrix_path = directory / "whole-app-matrix.json"
    matrix = json.loads(matrix_path.read_text())
    names = {m["capture"] for m in matrix}
    for png in sorted(directory.glob("all-*-bulk-*.png")):
        if png.stem in names:
            continue
        data = json.loads(png.with_suffix(".json").read_text())
        width, height = map(int, png.stem.split("-")[1].split("x"))
        matrix.append({"capture": png.stem, "state": png.stem.split("-", 2)[-1],
                       "width": width, "height": height, "synthetic": True,
                       "distinct_persisted_pages": 100, "os_input_verified": False,
                       "main_source_sha256": data["main_source_sha256"],
                       "bulk_harness_sha256": digest(REPO / "crates/app/src/phase7_qualification_tests.rs"),
                       "geometry_receipt": f"bulk-{width}x{height}-geometry.json"})
    assert len(matrix) == 28
    matrix_path.write_text(json.dumps(matrix, indent=2) + "\n")
    subprocess.run(["python3", str(REPO / "scripts/ui-review-gallery.py"), str(directory)], check=True)


def collect(root, checks):
    archive = json.loads((DOCS / "archive-receipt.json").read_text())
    for item in archive["files"]:
        assert digest(REPO / item["archive"]) == item["sha256"]
    original = json.loads((DOCS / "phase5-receipt.json").read_text())["original_png_sha256"]
    for name, expected in original.items():
        assert digest(Path(archive["evidence_root"]) / name) == expected
    batches = {}
    for name, directory in [("inspection", root), ("confirmation", root / "confirmation")]:
        pairs = []
        matrix = json.loads((directory / "whole-app-matrix.json").read_text())
        gallery = json.loads((directory / "gallery-manifest.json").read_text())
        assert len(matrix) == len(gallery) == 28
        assert {m["capture"] for m in matrix} == {m["capture"] for m in gallery}
        for png in sorted(directory.glob("all-*.png")):
            metadata = png.with_suffix(".json")
            data = json.loads(metadata.read_text())
            assert digest(png) == data["png_sha256"]
            assert data["real_gpui_blade_offscreen"] and not data["os_input_verified"]
            entry = next(m for m in gallery if m["capture"] == png.stem)
            assert entry["sha256"] == digest(png)
            assert float(entry["width"]) == data["width"] and float(entry["height"]) == data["height"]
            assert png.name in (directory / "index.html").read_text()
            if name == "confirmation":
                assert data["main_source_sha256"] == digest(REPO / "crates/app/src/main.rs")
                assert data["task_source_sha256"] == digest(REPO / "crates/app/src/task_ux.rs")
                assert data["task_controls_source_sha256"] == digest(REPO / "crates/app/src/ui/controls.rs")
            pairs.append({"capture": png.stem, "png": digest(png), "metadata": digest(metadata)})
        assert len(pairs) == 28
        for entry in matrix:
            if "harness_sha256" in entry and name == "confirmation":
                assert entry["harness_sha256"] == digest(REPO / "crates/app/src/ui_review_matrix.rs")
            if "bulk_harness_sha256" in entry and name == "confirmation":
                assert entry["bulk_harness_sha256"] == digest(REPO / "crates/app/src/phase7_qualification_tests.rs")
        batches[name] = {"pairs": pairs, "matrix": digest(directory / "whole-app-matrix.json"),
                         "gallery": digest(directory / "gallery-manifest.json"),
                         "html": digest(directory / "index.html"),
                         "auxiliary": {str(p.relative_to(directory)): digest(p)
                                       for p in sorted(directory.rglob("*"))
                                       if p.is_file() and (p.name in {"tails-matrix.original.json", "renderer-proof.png", "proof-reviewed.json"}
                                                           or "website-fixtures" in p.relative_to(directory).parts)
                                       and "confirmation" not in p.relative_to(directory).parts},
                         "geometry": {p.name: digest(p) for p in sorted(directory.glob("bulk-*-geometry.json"))},
                        "sheets": {p.name: digest(p) for p in sorted((directory / "contact-sheets").glob("*.png"))}}
    # Historical receipts remain historical: validate their frozen pixels, not
    # current-source hashes invalidated by this phase's named functional repairs.
    phase6 = json.loads((DOCS / "phase6-receipt.json").read_text())
    for batch, data in phase6["batches"].items():
        directory = Path(phase6["root"]) / ("confirmation" if batch == "confirmation" else "")
        for pair in data["pairs"]:
            assert digest(directory / (pair["capture"] + ".png")) == pair["png"]
            assert digest(directory / (pair["capture"] + ".json")) == pair["metadata"]
    sources = {str(p.relative_to(REPO)): digest(p) for p in sorted((REPO / "crates").glob("*/src/**/*.rs"))}
    for p in sorted((REPO / "crates/app/tests").rglob("*.rs")):
        sources[str(p.relative_to(REPO))] = digest(p)
    for p in sorted((REPO / "worker/product").rglob("*.mjs")):
        sources[str(p.relative_to(REPO))] = digest(p)
    for p in [REPO / "Cargo.lock", REPO / "crates/app/Cargo.toml", *sorted((REPO / "scripts").glob("phase7-*.py"))]:
        sources[str(p.relative_to(REPO))] = digest(p)
    results = json.loads((checks / "results.json").read_text())
    assert len(results["results"]) == 9 and all(r["exit"] == 0 for r in results["results"]), results
    for name, expected in json.loads((checks / "source-start.json").read_text()).items():
        assert digest(REPO / name) == expected, f"source changed during feature matrix: {name}"
    files = {str(p): digest(p) for p in sorted(checks.glob("*")) if p.is_file()}
    for attempt in sorted(root.parent.glob("regressee-phase7-checks-*")):
        # Freeze earlier failed/aborted attempts too, without treating their
        # historical source snapshots or results as the final acceptance matrix.
        for p in sorted(attempt.glob("*")):
            if p.is_file():
                files[str(p)] = digest(p)
    for p in sorted(root.glob("*.log")):
        files[str(p)] = digest(p)
    for p in sorted(root.glob("*-boundary.json")):
        data = json.loads(p.read_text())
        for name, expected in data.get("source_hashes", {}).items():
            assert digest(REPO / "worker/product" / name) == expected
        files[str(p)] = digest(p)
    for directory in sorted(root.parent.glob("regressee-phase7-native-*")):
        # A failed Xlib preflight has no receipt; its preserved log still counts.
        for p in sorted(directory.glob("*")):
            if p.is_file(): files[str(p)] = digest(p)
    for p in sorted(root.parent.glob("phase7-*.log")):
        files[str(p)] = digest(p)
    for p in sorted(root.parent.glob("phase7-*.json")):
        files[str(p)] = digest(p)
    stores = sorted(root.glob("storage-*/qualification.json"))
    assert {p.parent.name for p in stores} == {"storage-100", "storage-1000"}
    for p in stores:
        data=json.loads(p.read_text())
        assert data["pages"] in [100,1000] and data["corruption_after_preview_rejected"]
        for retained in sorted(p.parent.rglob("*")):
            if retained.is_file():
                files[str(retained)] = digest(retained)
    reuse = {p.name: digest(p) for p in DOCS.glob("phase[1-6]-*.md")}
    return {"phase": 7, "base": "135d3eb", "status": "implementation handoff; independent verification pending; native/a11y/usability/release gates BLOCKED",
            "root": str(root), "checks": str(checks), "immutable_archives":len(archive["files"]),
            "original_pngs_unchanged":len(original), "preserved_phase6_pairs":68,
            "sources":sources, "batches":batches, "files":files, "reused_reports":reuse,
            "feature_matrix":results, "qualification_limits":["distinct generated synthetic PNGs/identities, not browser or participant throughput", "real Linux offscreen pixels, not OS input", "AT-SPI registry available; owned native app unregistered; no accessible task pass", "Windows/NVDA and macOS/VoiceOver unavailable: NOT TESTED", "zero PM participants: NOT TESTED", "trusted runtime distribution and release qualification unresolved"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--checks", type=Path)
    parser.add_argument("--index", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.index:
        index(args.root)
        return
    assert args.checks
    current = collect(args.root, args.checks)
    receipt = DOCS / "phase7-receipt.json"
    if args.write:
        assert not receipt.exists(), "never silently rewrite frozen qualification"
        receipt.write_text(json.dumps(current, indent=2) + "\n")
    else:
        assert current == json.loads(receipt.read_text()), "source/evidence/log drift"
    print("PASS: 12 archives, 182 original PNGs, 68 historical Phase6 pairs, 28+28 Phase7 pairs and final feature matrix; NOT native/release acceptance")


if __name__ == "__main__":
    main()
