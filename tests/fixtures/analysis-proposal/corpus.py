#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Generate once, or independently decode/validate this synthetic PNG corpus.

Python standard library only. No browser, Rust, network, or site screenshots.
The PNG writer uses stored DEFLATE blocks for byte stability across zlib versions.
The reader does NOT invoke the generator to derive its expected counts.
"""

import argparse
import hashlib
import json
import struct
import sys
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SIGNATURE = b"\x89PNG\r\n\x1a\n"
BACKGROUND = (240, 240, 240)
INK = (32, 64, 96)
MAX_WIDTH = 256
MAX_HEIGHT = 2048
MAX_PIXELS = 262144
MAX_FILE_BYTES = 2 * 1024 * 1024


def rect(x, y, width, height):
    return dict(x=x, y=y, width=width, height=height)


MASKS = [rect(8, 8, 16, 16), rect(16, 8, 16, 16), rect(60, 10, 8, 8)]


def case(identifier, before, after, changed, ignored=0, masked_changed=0,
         structural=0, masks=None, components=0, largest=0, expectation="",
         formula="", comparable=True):
    """Literal mathematical oracles, not measurements of generated pixels."""
    canvas = [max(before[0], after[0]), max(before[1], after[1])]
    area = canvas[0] * canvas[1]
    return {
        "id": identifier,
        "before_dimensions": list(before),
        "after_dimensions": list(after),
        "canvas_dimensions": canvas,
        "comparable_geometry": comparable,
        "masks": masks or [],
        "reference_counts": {
            "canvas_pixels": area,
            "raw_changed_pixels": changed,
            "raw_overlap_rgb_changed_pixels": changed - structural,
            "raw_structural_pixels": structural,
            "ignored_union_pixels": ignored,
            "masked_raw_changed_pixels": masked_changed,
            "eligible_unmasked_pixels": area - ignored,
            "unmasked_raw_changed_pixels": changed - masked_changed,
            "unmasked_structural_pixels": structural,
            "unmasked_changed_fraction": {
                "numerator": changed - masked_changed,
                "denominator": area - ignored,
                "defined": area != ignored,
            },
            "unmasked_raw_4_connected_components": components,
            "largest_unmasked_raw_component_pixels": largest,
        },
        "dimensions_changed": before != after,
        "count_derivation": formula,
        "intended_evidence_and_semantic_expectation": expectation,
        "policy_classification": "not evaluated; see seed proposals in report",
    }


def specifications():
    # Connected components apply to exact raw inequality, NOT perceptual pixels.
    return [
        case("unchanged", (96, 64), (96, 64), 0,
             formula="96*64=6144 eligible; identical RGB samples.",
             expectation="Exact unchanged control; can establish no significant change only after comparability gates."),
        case("small-rgb-noise", (96, 64), (96, 64), 6144,
             components=1, largest=6144,
             formula="Every one of 96*64 pixels has +1 in each RGB channel.",
             expectation="Synthetic low-amplitude color drift, not real anti-aliasing; perceptual tolerance should suppress it, even though raw connectivity is global."),
        case("tiny-icon-long-page", (96, 2048), (96, 2048), 64,
             components=1, largest=64,
             formula="8*8=64 changed; denominator 96*2048=196608; fraction=1/3072.",
             expectation="Coherent high-contrast 8x8 synthetic icon across a tile seam; review-worthy candidate must not disappear in long-page global dilution."),
        case("large-coherent-region", (128, 96), (128, 96), 3840,
             components=1, largest=3840,
             formula="80*48=3840 changed; denominator 128*96=12288.",
             expectation="Large high-contrast contiguous rectangle; preserve bounds and area as visual-change evidence, not a defect verdict."),
        case("scattered-high-contrast-noise", (128, 96), (128, 96), 192,
             components=192, largest=1,
             formula="16 columns * 12 rows of isolated pixels = 192; fraction=1/64.",
             expectation="High-contrast isolated synthetic speckles: global percentage may flag them, but region evidence must distinguish them from one coherent component."),
        case("added-height-band", (96, 64), (96, 80), 1536,
             structural=1536, components=1, largest=1536,
             formula="96*(80-64)=1536 unmatched pixels; 96*80=7680 eligible.",
             expectation="Appended band has exactly the same RGB as background; it remains explicit structural evidence, never matching fabricated padding."),
        case("removed-height-band", (96, 80), (96, 64), 1536,
             structural=1536, components=1, largest=1536,
             formula="96*(80-64)=1536 unmatched pixels; symmetric denominator 7680.",
             expectation="Removed band must preserve direction of height delta while matching the added-band raw area."),
        case("inserted-middle-band", (96, 96), (96, 112), 3072,
             structural=1536, components=3, largest=1536,
             formula="96*8 old bar + 96*8 shifted bar + 96*16 unmatched tail = 3072; 10752 eligible.",
             expectation="Synthetic 16-row middle insertion shifts an 8-row bar; aligned coordinates preserve shift plus height evidence without automatic warping."),
        case("ignored-rectangles", (96, 64), (96, 64), 400,
             ignored=448, masked_changed=384, masks=MASKS,
             components=1, largest=16,
             formula="24*16 + 4*4 =400 raw; mask union (16*16+16*16-8*16)+8*8=448; eligible=5696; unmasked=16.",
             expectation="Two overlapping masks hide a changed rectangle; a third covers unchanged pixels. Count mask union once and retain the separate 4x4 unmasked change."),
        case("ignored-only-change", (96, 64), (96, 64), 384,
             ignored=448, masked_changed=384, masks=MASKS,
             formula="24*16=384 raw, all masked; same 448-pixel mask union; eligible=5696; unmasked=0.",
             expectation="No raw change outside masks; may establish no significant change only in eligible area, while originals and ignored coverage remain visible."),
        case("entire-image-masked", (96, 64), (96, 64), 6144,
             ignored=6144, masked_changed=6144, masks=[rect(0, 0, 96, 64)],
             formula="6144 raw and ignored; eligible=0, changed=0; 0/0 is undefined, NOT zero percent.",
             expectation="Insufficient eligible evidence; must not claim unchanged or divide by zero. Current enum could use NotComparable plus an explicit all-masked reason."),
        case("near-entire-image-masked", (96, 64), (96, 64), 6144,
             ignored=6048, masked_changed=6048, masks=[rect(0, 0, 96, 63)],
             components=1, largest=96,
             formula="96*63=6048 ignored; remaining 96 pixels all changed, fraction=96/96.",
             expectation="98.4375% ignored: warn about limited coverage; report 100% change of the remaining area, not 100% of the page."),
        case("unexpected-width-change", (96, 64), (104, 64), 512,
             structural=512, components=1, largest=512, comparable=False,
             formula="Illustrative coordinate union: (104-96)*64=512 unmatched; canvas=6656. Not a valid automatic profile comparison.",
             expectation="Unexpected width change requires profile-integrity gate; illustrative counts must not authorize automatic comparison or approval."),
    ]


def bounded_dimensions(width, height):
    if not (0 < width <= MAX_WIDTH and 0 < height <= MAX_HEIGHT
            and width * height <= MAX_PIXELS):
        raise ValueError("image dimensions exceed corpus bounds")


def solid(width, height, color=BACKGROUND):
    bounded_dimensions(width, height)
    return bytearray(bytes(color) * (width * height))


def paint(pixels, width, box, color):
    x, y, w, h = (box[k] for k in ("x", "y", "width", "height"))
    for row in range(y, y + h):
        offset = (row * width + x) * 3
        pixels[offset:offset + w * 3] = bytes(color) * w


def generate_pair(spec):
    aw, ah = spec["before_dimensions"]
    bw, bh = spec["after_dimensions"]
    a, b = solid(aw, ah), solid(bw, bh)
    identifier = spec["id"]
    if identifier == "small-rgb-noise":
        b = solid(bw, bh, (241, 241, 241))
    elif identifier == "tiny-icon-long-page":
        box = rect(30, 1020, 8, 8)
        paint(a, aw, box, INK)
        paint(b, bw, box, (208, 64, 32))
    elif identifier == "large-coherent-region":
        paint(b, bw, rect(16, 16, 80, 48), INK)
    elif identifier == "scattered-high-contrast-noise":
        for y in range(2, bh, 8):
            for x in range(2, bw, 8):
                paint(b, bw, rect(x, y, 1, 1), INK)
    elif identifier == "inserted-middle-band":
        paint(a, aw, rect(0, 32, 96, 8), INK)
        paint(b, bw, rect(0, 48, 96, 8), INK)
    elif identifier in ("ignored-rectangles", "ignored-only-change"):
        paint(b, bw, rect(8, 8, 24, 16), INK)
        if identifier == "ignored-rectangles":
            paint(b, bw, rect(40, 40, 4, 4), INK)
    elif identifier in ("entire-image-masked", "near-entire-image-masked"):
        b = solid(bw, bh, INK)
    elif identifier not in ("unchanged", "added-height-band",
                            "removed-height-band", "unexpected-width-change"):
        raise ValueError("unhandled fixture")
    return bytes(a), bytes(b)


def chunk(kind, payload):
    return (struct.pack(">I", len(payload)) + kind + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xffffffff))


def png_bytes(width, height, pixels):
    bounded_dimensions(width, height)
    if len(pixels) != width * height * 3:
        raise ValueError("wrong RGB sample count")
    rows = b"".join(b"\x00" + pixels[y * width * 3:(y + 1) * width * 3]
                    for y in range(height))
    # RFC1950 header, RFC1951 byte-aligned stored blocks, Adler32 trailer.
    stream = bytearray(b"\x78\x01")
    for start in range(0, len(rows), 65535):
        block = rows[start:start + 65535]
        final = int(start + len(block) == len(rows))
        stream.extend(bytes([final]))
        stream.extend(struct.pack("<HH", len(block), len(block) ^ 0xffff))
        stream.extend(block)
    stream.extend(struct.pack(">I", zlib.adler32(rows) & 0xffffffff))
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (SIGNATURE + chunk(b"IHDR", header)
            + chunk(b"IDAT", bytes(stream)) + chunk(b"IEND", b""))


def write_new(path, data):
    with path.open("xb") as output:
        output.write(data)


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def generate():
    # Fail before writing if there is ANY pre-existing generated output.
    if (ROOT / "images").exists() or (ROOT / "manifest.json").exists():
        raise ValueError("refusing to overwrite an existing corpus")
    specs = specifications()
    (ROOT / "images").mkdir()
    for spec in specs:
        a, b = generate_pair(spec)
        for side, pixels in (("before", a), ("after", b)):
            width, height = spec[side + "_dimensions"]
            payload = png_bytes(width, height, pixels)
            relative = "images/" + spec["id"] + "-" + side + ".png"
            write_new(ROOT / relative, payload)
            spec[side] = {
                "path": relative,
                "sha256": hashlib.sha256(payload).hexdigest(),
                "bytes": len(payload),
                "width": width,
                "height": height,
                "encoding": "PNG RGB8 opaque, non-interlaced, filter 0",
            }
    manifest = {
        "schema": "regressee.synthetic-analysis-corpus.v1",
        "license": "MPL-2.0",
        "provenance": "Original algorithmic color blocks generated by corpus.py; NOT site screenshots, real photos, font rasterizations, or calibrated user examples.",
        "generator": "corpus.py",
        "bounds": {"max_width": MAX_WIDTH, "max_height": MAX_HEIGHT,
                   "max_pixels": MAX_PIXELS, "max_file_bytes": MAX_FILE_BYTES},
        "measurement_contract": {
            "raw_pixel_difference": "Any unequal RGB sample at aligned coordinates, or exactly one image has a pixel there; no tolerance, padding, resampling, alignment, or anti-alias suppression.",
            "canvas": "max(width) by max(height); all cases have a rectangular union with no absent-in-both corner. Width-mismatch case is illustrative, not automatically comparable.",
            "masks": "Half-open capture-pixel rectangles; geometric union counted once. Same-coordinate coverage applied to both images; every mask here fits both dimensions; no dimension-change case uses masks.",
            "denominator": "Union canvas pixels minus mask union; zero denominator yields undefined fraction, never zero percent.",
            "connected_components": "4-neighbor connectivity of the UNMASKED exact raw change bitmap, including unmatched bands; no dilation or bounding-box-area substitution.",
            "perceptual_policy": "No perceptual metric or policy has been implemented/evaluated by this helper; raw oracles are separate from seed proposals.",
        },
        "cases": specs,
    }
    write_new(ROOT / "manifest.json", json_bytes(manifest))
    print("Generated 13 cases / 26 PNGs without overwriting existing outputs.")


def decode_png(data):
    """Independent bounded PNG parser; verify every chunk CRC and full zlib EOF."""
    if len(data) > MAX_FILE_BYTES or data[:8] != SIGNATURE:
        raise ValueError("invalid PNG signature or file budget")
    pos, chunks, payloads = 8, [], []
    while pos < len(data):
        if len(data) - pos < 12:
            raise ValueError("truncated chunk")
        length = struct.unpack_from(">I", data, pos)[0]
        end = pos + 12 + length
        if end > len(data):
            raise ValueError("chunk length exceeds file")
        kind = data[pos + 4:pos + 8]
        payload = data[pos + 8:pos + 8 + length]
        crc = struct.unpack_from(">I", data, pos + 8 + length)[0]
        if crc != (zlib.crc32(kind + payload) & 0xffffffff):
            raise ValueError("PNG chunk CRC mismatch")
        chunks.append(kind)
        payloads.append(payload)
        pos = end
        if kind == b"IEND":
            if payload or pos != len(data):
                raise ValueError("nonempty IEND or trailing bytes")
            break
    if chunks != [b"IHDR", b"IDAT", b"IEND"]:
        raise ValueError("incomplete or unexpected corpus PNG chunk sequence")
    if len(payloads[0]) != 13:
        raise ValueError("invalid IHDR length")
    width, height, depth, color, compression, filtering, interlace = struct.unpack(
        ">IIBBBBB", payloads[0])
    bounded_dimensions(width, height)
    if (depth, color, compression, filtering, interlace) != (8, 2, 0, 0, 0):
        raise ValueError("unsupported corpus image encoding")
    expected = height * (1 + 3 * width)
    decoder = zlib.decompressobj()
    rows = decoder.decompress(payloads[1], expected + 1)
    if (len(rows) != expected or not decoder.eof or decoder.unused_data
            or decoder.unconsumed_tail):
        raise ValueError("wrong decompressed size or incomplete/trailing zlib stream")
    pixels = bytearray()
    stride = 1 + width * 3
    for start in range(0, len(rows), stride):
        if rows[start] != 0:
            raise ValueError("unexpected scanline filter")
        pixels.extend(rows[start + 1:start + stride])
    return width, height, bytes(pixels), len(chunks)


def measure(a, b, masks):
    """Measure decoded samples with coordinate/mask sets, not paint operations."""
    aw, ah, ap = a[:3]
    bw, bh, bp = b[:3]
    width, height = max(aw, bw), max(ah, bh)
    ignored = set()
    for box in masks:
        x, y, w, h = (box[k] for k in ("x", "y", "width", "height"))
        if (min(x, y) < 0 or min(w, h) <= 0 or x + w > min(aw, bw)
                or y + h > min(ah, bh)):
            raise ValueError("mask requires geometry revalidation")
        ignored.update((row, col) for row in range(y, y + h)
                       for col in range(x, x + w))
    changed, structural = set(), set()
    for y in range(height):
        for x in range(width):
            in_a, in_b = x < aw and y < ah, x < bw and y < bh
            if not (in_a or in_b):
                raise ValueError("unsupported absent-in-both canvas corner")
            if in_a != in_b:
                structural.add((y, x))
                changed.add((y, x))
            else:
                ai, bi = (y * aw + x) * 3, (y * bw + x) * 3
                if ap[ai:ai + 3] != bp[bi:bi + 3]:
                    changed.add((y, x))
    unmasked = changed - ignored
    remaining = set(unmasked)
    components, largest = 0, 0
    while remaining:
        components += 1
        pending = [remaining.pop()]
        size = 0
        while pending:
            y, x = pending.pop()
            size += 1
            for neighbor in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if neighbor in remaining:
                    remaining.remove(neighbor)
                    pending.append(neighbor)
        largest = max(largest, size)
    eligible = width * height - len(ignored)
    return {
        "canvas_pixels": width * height,
        "raw_changed_pixels": len(changed),
        "raw_overlap_rgb_changed_pixels": len(changed - structural),
        "raw_structural_pixels": len(structural),
        "ignored_union_pixels": len(ignored),
        "masked_raw_changed_pixels": len(changed & ignored),
        "eligible_unmasked_pixels": eligible,
        "unmasked_raw_changed_pixels": len(unmasked),
        "unmasked_structural_pixels": len(structural - ignored),
        "unmasked_changed_fraction": {"numerator": len(unmasked),
                                      "denominator": eligible, "defined": eligible > 0},
        "unmasked_raw_4_connected_components": components,
        "largest_unmasked_raw_component_pixels": largest,
    }


def corruption_checks():
    # Test the reader against deliberately invalid in-memory objects; no files.
    valid = png_bytes(1, 1, bytes(BACKGROUND))
    # Change compressed data without repairing its outer CRC.
    bad_crc = bytearray(valid)
    bad_crc[45] ^= 1
    # Replace IDAT with an invalid zlib checksum, but give it a VALID PNG CRC.
    idat_start = 33
    length = struct.unpack_from(">I", valid, idat_start)[0]
    compressed = bytearray(valid[idat_start + 8:idat_start + 8 + length])
    compressed[-1] ^= 1
    bad_adler = valid[:idat_start] + chunk(b"IDAT", bytes(compressed)) + chunk(b"IEND", b"")
    malformed = {
        "bad_signature": b"wrong!!!" + valid[8:],
        "chunk_crc_mismatch": bytes(bad_crc),
        "truncated_chunk": valid[:-1],
        "missing_iend": valid[:-12],
        "trailing_bytes": valid + b"x",
        "invalid_zlib_adler_with_valid_chunk_crc": bad_adler,
        "oversized_ihdr_with_valid_crc": SIGNATURE + chunk(
            b"IHDR", struct.pack(">IIBBBBB", MAX_WIDTH + 1, 1, 8, 2, 0, 0, 0))
            + valid[33:],
    }
    for name, payload in malformed.items():
        try:
            decode_png(payload)
        except (ValueError, zlib.error):
            continue
        raise ValueError("reader incorrectly accepted " + name)
    return sorted(malformed)


def validate():
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    if manifest["schema"] != "regressee.synthetic-analysis-corpus.v1":
        raise ValueError("wrong schema")
    specs = specifications()
    if len(manifest["cases"]) != len(specs):
        raise ValueError("wrong case count")
    files, results, crc_count, total_bytes, decoded_pixels = set(), [], 0, 0, 0
    for actual, expected in zip(manifest["cases"], specs):
        # Verify every literal oracle independently of manifest contents.
        for key, value in expected.items():
            if actual[key] != value:
                raise ValueError("manifest oracle mismatch: " + expected["id"] + "/" + key)
        images = []
        for side in ("before", "after"):
            entry = actual[side]
            relative = "images/" + actual["id"] + "-" + side + ".png"
            if entry["path"] != relative or relative in files:
                raise ValueError("unexpected or duplicate image path")
            path = ROOT / relative
            if path.is_symlink() or path.stat().st_size > MAX_FILE_BYTES:
                raise ValueError("symlink or image file budget exceeded")
            data = path.read_bytes()
            if (len(data) != entry["bytes"]
                    or hashlib.sha256(data).hexdigest() != entry["sha256"]):
                raise ValueError("image SHA256/size mismatch: " + relative)
            image = decode_png(data)
            if ([image[0], image[1]] != actual[side + "_dimensions"]
                    or image[:2] != (entry["width"], entry["height"])):
                raise ValueError("decoded dimension mismatch")
            images.append(image)
            files.add(relative)
            crc_count += image[3]
            total_bytes += len(data)
            decoded_pixels += image[0] * image[1]
        counts = measure(*images, actual["masks"])
        if counts != actual["reference_counts"]:
            raise ValueError("decoded mathematical counts mismatch: " + actual["id"])
        results.append({"id": actual["id"], "counts": counts, "passed": True})
    found = {"images/" + p.name for p in (ROOT / "images").iterdir()}
    if found != files:
        raise ValueError("unexpected or missing image files")
    return {
        "schema": "regressee.synthetic-analysis-validation.v1",
        "status": "PASS",
        "validation_scope": "Independent decode/CRC/zlib/SHA256/raw mathematical oracles only; no Rust, perceptual-policy, site, runtime, or defect-detection verification.",
        "case_count": len(results),
        "png_count": len(files),
        "chunk_crcs_verified": crc_count,
        "sha256_verified": len(files),
        "total_png_bytes": total_bytes,
        "total_decoded_pixels": decoded_pixels,
        "corruption_cases_rejected": corruption_checks(),
        "cases": results,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("generate", "validate"))
    parser.add_argument("--report", type=Path,
                        help="new JSON validation report inside this corpus only")
    args = parser.parse_args()
    if args.operation == "generate":
        if args.report:
            parser.error("--report applies to validate only")
        generate()
    else:
        report = validate()
        if args.report:
            target = args.report.resolve()
            if target.parent != ROOT or target.exists():
                parser.error("report must be a new file directly inside the corpus")
            write_new(target, json_bytes(report))
        sys.stdout.write(json_bytes(report).decode("utf-8"))


if __name__ == "__main__":
    main()
