#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Bounded primary metadata observation ONLY. Never downloads/executes payloads."""
import argparse, hashlib, json, urllib.request, urllib.error
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);a=p.parse_args()
a.output.mkdir(mode=0o700,parents=True,exist_ok=False)
def get(name,url):
    with urllib.request.urlopen(url,timeout=30) as r:
        value=r.read(8*1024*1024+1)
        if len(value)>8*1024*1024: raise ValueError('metadata bound')
    (a.output/name).write_bytes(value)
    return value
index=json.loads(get('node-index.json','https://nodejs.org/dist/index.json'))
node=next(r for r in index if r['version'].startswith('v24.') and r['lts'])
pw=json.loads(get('playwright.json','https://registry.npmjs.org/playwright/latest'))
browsers=json.loads(get('browsers.json',f"https://raw.githubusercontent.com/microsoft/playwright/v{pw['version']}/packages/playwright-core/browsers.json"))
chrome=next(b for b in browsers['browsers'] if b['name']=='chromium')
sums=get('node-shasums.txt',f"https://nodejs.org/dist/{node['version']}/SHASUMS256.txt").decode()
cft=json.loads(get('cft.json','https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json'))
matching=next(v for v in cft['versions'] if v['version']==chrome['browserVersion'])
url=next(d['url'] for d in matching['downloads']['chrome'] if d['platform']=='linux64')
probes=[]
for suffix in ['.sha256','.sha256sum']:
    endpoint=url+suffix
    try:
        with urllib.request.urlopen(urllib.request.Request(endpoint,method='HEAD'),timeout=20) as r:status=r.status
    except urllib.error.HTTPError as e:status=e.code
    probes.append({'url':endpoint,'status':status})
report={'node':node,'playwright':pw['version'],'browser':chrome,'matched_cft_revision':matching['revision'],
    'linux_node_sha256':next(line.split()[0] for line in sums.splitlines() if line.endswith(f"node-{node['version']}-linux-x64.tar.xz")),
    'browser_digest_probes':probes,'candidate_install':'BLOCKED / NOT RUN',
    'reason':'No reviewed upstream Node signing-key trust root or authenticated SHA-256/signature for exact Linux browser obtained. TLS metadata/SRI availability alone does not authorize executing the complete new tuple. No payload download, unverified execution or public promotion.',
    'historical_runtime':'unchanged; literal-loopback-only',
    'files':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(a.output.iterdir())}}
(a.output/'observation.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report,indent=2))
