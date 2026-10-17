#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""One budgeted actual soak; wrapper owns/reaps children, this driver samples RSS.

Requires a preverified source-matched installed signed fixture runtime. Never
downloads or promotes it, and writes only fresh evidence/app-data directories.
"""
import argparse,json,os,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);p.add_argument('--distribution',required=True,type=Path);p.add_argument('--runtime-cache',required=True,type=Path);a=p.parse_args()
a.output.mkdir(mode=0o700,parents=True,exist_ok=False);out=a.output.resolve();root=Path(__file__).resolve().parent.parent
env=dict(os.environ,REGRESSEE_PHASE7_EVIDENCE=str(out/'data'),REGRESSEE_PHASE7_DISTRIBUTION=str(a.distribution.resolve()),REGRESSEE_PHASE7_RUNTIME_CACHE=str(a.runtime_cache.resolve()))
cmd=['python3','scripts/check-phase6-corrections.py','--output',str(out/'command'),'--timeout','1560','--','cargo','test','--locked','-p','regressee-app','--features','test-support','--test','product_flow','phase7_actual_1000_page_soak_pause_resume_cancel','--','--exact','--ignored','--nocapture','--test-threads=1']
known={};peaks={'AppRust':0,'Node':0,'Chromium':0,'HarnessBuild':0};samples=0;start=time.monotonic()
with (out/'wrapper.log').open('xb') as log:
    child=subprocess.Popen(cmd,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
    while child.poll() is None:
        rows={}
        for p in Path('/proc').iterdir():
            if not p.name.isdecimal():continue
            try:
                text=(p/'stat').read_text();comm=text[text.index('(')+1:text.rindex(')')];v=text[text.rindex(')')+2:].split()
                rows[int(p.name)]=(int(v[1]),v[19],comm,int(v[21])*os.sysconf('SC_PAGE_SIZE'))
            except (OSError,ValueError,IndexError):pass
        owned={child.pid}
        while True:
            before=len(owned);owned.update(pid for pid,v in rows.items() if v[0] in owned)
            if len(owned)==before:break
        totals=dict.fromkeys(peaks,0)
        for pid,v in rows.items():
            if pid not in owned and known.get(pid)!=v[1]:continue
            known[pid]=v[1]
            group='Node' if v[2]=='node' else 'Chromium' if 'chrome' in v[2].lower() or 'chromium' in v[2].lower() else 'AppRust' if v[2].startswith('product_flow') else 'HarnessBuild'
            totals[group]+=v[3]
        for k in peaks:peaks[k]=max(peaks[k],totals[k])
        samples+=1;time.sleep(0.25)
    code=child.wait()
(out/'memory.json').write_text(json.dumps({'exit':code,'wall_seconds':time.monotonic()-start,'peak_group_rss_bytes':peaks,'samples':samples,'interval_ms':250,
    'note':'Observed descendants / same-start-time matches; shared pages may be counted twice and short-lived processes missed. AppRust is actual product_flow process, build/harness separate. NOT native UI latency.'},indent=2))
raise SystemExit(code)
