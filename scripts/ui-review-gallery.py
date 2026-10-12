#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Index unmodified native PNG evidence; contact sheets are labelled derivatives."""
import argparse
import hashlib
import html
import json
import pathlib
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("root", type=pathlib.Path)
args = parser.parse_args()
root = args.root.resolve()
matrix = json.loads((root / "whole-app-matrix.json").read_text())
files = sorted(root.glob("task-*.png"))
matrix.extend({"capture": p.stem, "state": p.stem.split("-", 2)[-1], "width": int(p.stem.split("-")[1].split("x")[0]), "height": int(p.stem.split("-")[1].split("x")[1]), "synthetic": True} for p in files)
cards = []
for index, entry in enumerate(matrix, 1):
    name = entry["capture"]
    p = root / f"{name}.png"
    entry["sha256"] = hashlib.sha256(p.read_bytes()).hexdigest()
    entry["number"] = index
    cards.append(f'<article data-search="{html.escape(name)}"><h2>{index:03} · {html.escape(entry["state"])} <small>{entry["width"]}×{entry["height"]}</small></h2><a href="{name}.png" target="_blank"><img loading="lazy" src="{name}.png" alt="Actual native UI: {html.escape(name)}"></a><p><a href="{name}.png">Open full-resolution PNG</a> · <a href="{name}.json">Capture metadata</a></p></article>')
page = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Regressee — whole-app evidence</title><style>
body{margin:0;background:#e9edf0;color:#22313d;font:15px/1.5 system-ui}header{padding:24px;background:white;border-bottom:1px solid #ced9e2}h1{margin:0;font-size:26px}header p{max-width:1000px}input{padding:12px;font:inherit;width:min(600px,80vw)}main{padding:24px;display:grid;grid-template-columns:repeat(auto-fit,minmax(min(600px,100%),1fr));gap:24px}article{background:white;padding:16px}h2{font-size:17px;margin:0 0 12px}small{font-weight:normal;float:right}img{display:block;width:100%;height:auto}a{color:#1765ac}article[hidden]{display:none}article:target{outline:3px solid #1765ac}</style><header><h1>Regressee · whole-app visual evidence</h1><p>Actual Linux GPUI/Blade pixels at normal and minimum desktop window sizes. Synthetic local projects and website imagery. No user project was opened or changed. These are app screenshots, not redesigned mockups. Offscreen rendering does not verify OS input, accessibility or other platforms.</p><label>Find a screen or state <input id="search" placeholder="review, access, history, 820x600…" type="search"></label><p id="count"></p></header><main>''' + "\n".join(cards) + '''</main><script>const input=document.querySelector('#search'),cards=[...document.querySelectorAll('article')];function filter(){let n=0;for(const card of cards){card.hidden=!card.dataset.search.includes(input.value.toLowerCase());if(!card.hidden)n++}document.querySelector('#count').textContent=`${n} of ${cards.length} captures shown · click any image for original pixels`}input.addEventListener('input',filter);filter();</script></html>'''
(root / "index.html").write_text(page)
(root / "gallery-manifest.json").write_text(json.dumps(matrix, indent=2))
sheets = root / "contact-sheets"
sheets.mkdir(exist_ok=True)
for prefix in ["all-1280", "all-820", "task-1280", "task-820"]:
    entries = [m for m in matrix if m["capture"].startswith(prefix)]
    for start in range(0, len(entries), 6):
        group = entries[start:start + 6]
        output = sheets / f"{prefix}-{start // 6 + 1:02}.png"
        command = ["ffmpeg", "-hide_banner", "-loglevel", "error"]
        for m in group:
            command.extend(["-i", str(root / (m["capture"] + ".png"))])
        filters = []
        for i, m in enumerate(group):
            title = f'{m["number"]:03} {m["state"]} {m["width"]}x{m["height"]}'
            filters.append(f"[{i}:v]scale=640:400:force_original_aspect_ratio=decrease,pad=640:400:0:0:color=white,pad=640:430:0:30:color=0xe9edf0,drawtext=fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf:text='{title}':x=8:y=7:fontsize=15:fontcolor=0x22313d[s{i}]")
        layout = "|".join(f"{i % 2 * 640}_{i // 2 * 430}" for i in range(len(group)))
        if len(group) == 1:
            filters.append("[s0]null[out]")
        else:
            filters.append("".join(f"[s{i}]" for i in range(len(group))) + f"xstack=inputs={len(group)}:layout={layout}:fill=white[out]")
        command.extend(["-filter_complex_threads", "1", "-filter_complex", ";".join(filters), "-map", "[out]", "-frames:v", "1", "-y", str(output)])
        subprocess.run(command, check=True)
print(json.dumps({"images": len(matrix), "gallery": str(root / "index.html"), "contact_sheets": len(list(sheets.glob('*.png')))}, indent=2))
