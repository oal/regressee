#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Serial locked local Phase 4 verification; never public/native qualification."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

p=argparse.ArgumentParser()
p.add_argument('--output',required=True,type=Path)
p.add_argument('--runtime-only',action='store_true')
args=p.parse_args()
root=Path(__file__).resolve().parent.parent
args.output.mkdir(parents=True,exist_ok=False)
env=dict(os.environ,LIBRARY_PATH=str(root/'.runtime/link-libs'),RUST_TEST_THREADS='1',CARGO_BUILD_JOBS='4',REGRESSEE_PHASE4_EVIDENCE=str(args.output.resolve()/'product-flow'))
commands=[] if args.runtime_only else [('locked-ui-tests-clippy',['bash','scripts/check-foundation.sh']),('owned-os-store',['cargo','test','--locked','-p','regressee-runtime','secrets::tests::actual_owned_os_store_availability','--','--exact','--ignored','--nocapture','--test-threads=1'])]
commands.append(('signed-auth-capture',['cargo','test','--locked','-p','regressee-app','--features','ui-tests','--test','product_flow','actual_phase4_auth_environment_and_capture','--','--exact','--ignored','--nocapture','--test-threads=1']))
records=[]
for name,command in commands:
    start=time.monotonic()
    with (args.output/f'{name}.log').open('wb') as log:
        result=subprocess.run(command,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=1200)
    records.append({'name':name,'command':command,'exit':result.returncode,'wall_seconds':time.monotonic()-start,'qualification':'local implementer checks; not independent/native/human/public qualification'})
    (args.output/'commands.json').write_text(json.dumps(records,indent=2))
    if result.returncode:
        raise SystemExit(result.returncode)
