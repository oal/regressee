#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Refresh app-bundled worker sources/catalogs without downloading dependencies.

Verify and retain the existing pinned dependency bytes and vendor component
inventories. This changes the build's trust root, not any installed user runtime.
"""
import json
from pathlib import Path
import runpy
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    build = runpy.run_path(str(ROOT / 'scripts/official-runtime.py'))
    out = build['OUT']
    bundle = out / 'bundled-worker.zip'
    old_hash = build['digest_file'](bundle)
    catalogs = {p: json.loads((out / (p + '.json')).read_text()) for p in build['PLATFORMS']}
    if any(c['bundled_sha256'] != old_hash for c in catalogs.values()):
        raise ValueError('Existing bundle/catalog checksum mismatch')
    provenance = json.loads((out / 'provenance.json').read_text())
    if provenance['bundled_sha256'] != old_hash:
        raise ValueError('Existing bundle provenance mismatch')
    entries = {}
    with zipfile.ZipFile(bundle) as archive:
        build['check_archive_names']((i.filename, i.is_dir()) for i in archive.infolist())
        for info in archive.infolist():
            data = archive.read(info)
            mode = (info.external_attr >> 16) & 0o777
            expected = build['file_entry'](data, mode)
            for c in catalogs.values():
                if c['manifest']['files'].get(info.filename) != expected or c['manifest']['modes'].get(info.filename) != mode:
                    raise ValueError('Bundled entry differs from pinned catalog: ' + info.filename)
            if info.filename.startswith('worker/node_modules/'):
                entries[info.filename] = (data, mode)
    for name in ('playwright', 'playwright-core'):
        package = json.loads(entries[f'worker/node_modules/{name}/package.json'][0])
        if package['version'] != build['PW']:
            raise ValueError('Dependency version differs from build pin')
    bundled_hash = build['worker_payload'](entries)
    for platform, c in catalogs.items():
        m = c['manifest']
        for name in list(m['files']):
            if name.startswith('worker/'):
                del m['files'][name]
                del m['modes'][name]
        for name, (data, mode) in entries.items():
            m['files'][name] = build['file_entry'](data, mode)
            m['modes'][name] = mode
        m['identity']['Pinned']['worker'] = build['WORKER']
        m['id'] = f"official-{build['NODE']}-{build['PW']}-{build['WORKER']}-{platform}"
        c['bundled_sha256'] = bundled_hash
        build['write_json'](out / (platform + '.json'), c)
    for entry in provenance['catalogs']:
        entry['catalog_sha256'] = build['digest_file'](out / (entry['platform'] + '.json'))
    provenance['worker'] = build['WORKER']
    provenance['bundled_sha256'] = bundled_hash
    provenance['worker_source_refresh'] = {
        'previous_bundled_sha256': old_hash,
        'verification': 'Pinned dependency bytes and vendor component inventories retained; only local worker sources refreshed',
    }
    build['write_json'](out / 'provenance.json', provenance)
    print(f"Refreshed bundled worker {build['WORKER']}: {bundled_hash}")


if __name__ == '__main__':
    main()
