#!/usr/bin/env python3
"""Refresh only managed builder worker sources; no download, install or PATH edits.
Requires the already bootstrapped, version-matched private dependency tree.
Follow with pack/repair to verify and publish the installed generation.
"""
import pathlib, shutil
ROOT=pathlib.Path(__file__).resolve().parents[1]
source=ROOT/'.runtime/source/worker'
assert (source/'node_modules/playwright/package.json').is_file(), 'bootstrap required'
for name in ['package.json','package-lock.json']:
    assert (source/name).read_bytes()==(ROOT/'worker'/name).read_bytes(), f'{name} changed: bootstrap required'
for name in ['src','test']:
    shutil.copytree(ROOT/'worker'/name,source/name,dirs_exist_ok=True)
print('Refreshed managed builder worker src/test only; pack and repair required')
