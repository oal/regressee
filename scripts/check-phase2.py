#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial build/test/browser qualification with unique retained logs and resources.
No parallel commands, publication, global tool installation, or history copies.
"""
import argparse
import hashlib
import json
import os
import pathlib
import stat
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]

def protected():
    paths = list((ROOT / 'docs/phases').glob('0[01]-*.md'))
    paths += list((ROOT / 'docs/decisions').glob('000[1-6]*.md'))
    paths += [ROOT / 'PRODUCT.md', ROOT / 'PLAN.md', ROOT / 'DESIGN.md']
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

def resources(evidence):
    live=[]
    for entry in pathlib.Path('/proc').iterdir():
        if not entry.name.isdigit(): continue
        try:
            exe=(entry/'exe').resolve(strict=True)
            if exe.is_relative_to(ROOT/'target') or exe.is_relative_to(ROOT/'.runtime/installed') or exe.is_relative_to(ROOT/'artifacts') or exe.is_relative_to(evidence):
                live.append({'pid':int(entry.name),'exe':str(exe)})
        except (OSError,RuntimeError): pass
    # The v3 per-user namespace itself is stable infrastructure, not a leaked
    # worker. Report entries, never delete them or inspect unrelated temp roots.
    scratch=[];namespaces=[]
    parents={pathlib.Path('/tmp/opencode')/f'rg2-{os.geteuid()}',pathlib.Path('/tmp')/f'rg2-{os.geteuid()}'}
    if os.environ.get('REGRESSEE_WORKER_SCRATCH_PARENT'):
        parents.add(pathlib.Path(os.environ['REGRESSEE_WORKER_SCRATCH_PARENT']))
    for parent in sorted(parents):
        try:
            metadata=parent.lstat()
            if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid!=os.geteuid():continue
            namespaces.append(str(parent))
            scratch += [str(p) for p in parent.iterdir()]
        except OSError:pass
    return {'live_owned_candidate_processes':live,'product_scratch':scratch,'product_scratch_namespaces':namespaces}

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--output',type=pathlib.Path,required=True)
    p.add_argument('--native',action='store_true')
    p.add_argument('--checks-only',action='store_true',help='locked build/format/unit/UI-action/lint/protocol checks only; no browser/native repeat')
    args=p.parse_args()
    evidence=args.output.resolve();evidence.mkdir(parents=True,exist_ok=False)
    before=protected();(evidence/'preservation-before.json').write_text(json.dumps(before,indent=2))
    # Snapshot only the specific pipeline/checklist files we intend to overwrite.
    for name in ['docs/IMPLEMENTATION_STATUS.md','docs/phases/02-vertical-slice.md']:
        (evidence/pathlib.Path(name).name).write_bytes((ROOT/name).read_bytes())
    env=dict(os.environ,TMPDIR='/tmp/opencode',LIBRARY_PATH=str(ROOT/'.runtime/link-libs'),RUST_TEST_THREADS='1')
    env['REGRESSEE_PHASE2_EVIDENCE']=str(evidence/'product-flow')
    commands=[
        ['cargo','fmt','--all'],
        ['bash','scripts/check-foundation.sh'],
        [str(ROOT/'.runtime/installed/node/bin/node'),'--test','worker/product/test/protocol.test.mjs'],
        ['cargo','test','--locked','-p','regressee-app','--features','ui-tests','--test','product_flow','actual_signed_http_product_flow','--','--ignored','--nocapture','--test-threads=1'],
        ['cargo','run','--locked','--bin','regressee-foundation','--',str(evidence/'foundation-reopen')],
    ]
    if args.checks_only:commands=commands[:3]
    if args.native: commands.append(['python3','scripts/phase2-native.py','--output',str(evidence/'native')])
    results=[]
    for index,command in enumerate(commands):
        log=evidence/f'{index:02d}.log';start=time.monotonic()
        with log.open('w') as output:
            run=subprocess.run(['/usr/bin/time','-v',*command],cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
        item={'command':command,'exit':run.returncode,'seconds':time.monotonic()-start,'log':str(log)};results.append(item)
        print(json.dumps(item),flush=True)
        (evidence/'commands.json').write_text(json.dumps(results,indent=2))
        if run.returncode and 'native' not in ' '.join(command): break
    after=protected();audit=resources(evidence);audit['historical_reports_unchanged']=before==after
    (evidence/'resource-audit.json').write_text(json.dumps(audit,indent=2))
    print(json.dumps(audit),flush=True)
    return int(any(r['exit'] for r in results) or before!=after or bool(audit['live_owned_candidate_processes']))

if __name__=='__main__':raise SystemExit(main())
