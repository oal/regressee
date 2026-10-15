#!/usr/bin/env python3
"""Linux developer qualification with actual process and per-component RSS evidence."""
import argparse, hashlib, json, os, pathlib, signal, subprocess, threading, time
from evidence import inspect_captures
ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = ROOT / 'target/debug/regressee-spike'
ARTIFACTS = ROOT / 'artifacts'
SCRATCH = pathlib.Path('/tmp') / f'rg0-{os.geteuid()}'

def proc_table():
    table = {}
    for p in pathlib.Path('/proc').iterdir():
        if not p.name.isdecimal(): continue
        try:
            stat = (p / 'stat').read_text().rsplit(')', 1)[1].split()
            rss = int(stat[21]) * os.sysconf('SC_PAGE_SIZE')
            exe = os.readlink(p / 'exe')
            table[int(p.name)] = {'ppid': int(stat[1]), 'group': int(stat[2]), 'state': stat[0], 'start': stat[19], 'rss': rss, 'exe': exe}
        except (OSError, ValueError, IndexError): pass
    return table

def track(parent, table):
    found = {parent}
    while True:
        added = {pid for pid, p in table.items() if p['ppid'] in found}
        if added <= found: return found
        found |= added

def scoped_audit():
    live=[]
    for pid,data in proc_table().items():
        if data['state']=='Z': continue
        try: cmd=(pathlib.Path('/proc')/str(pid)/'cmdline').read_bytes()
        except OSError: continue
        owned=False
        for runtime in ['installed','source']:
            base=ROOT/'.runtime'/runtime
            if data['exe'].startswith(str(base/'browsers')+'/') or (data['exe']==str(base/'node/bin/node') and str(base/'worker').encode() in cmd): owned=True
        if data['exe'] in [str(ROOT/'target/debug/regressee-spike'),str(ROOT/'target/debug/regressee-ui')]: owned=True
        if owned:live.append(pid)
    scratch=[str(p) for p in SCRATCH.glob('worker-*')]
    pending=[str(p) for p in [ROOT/'.runtime/installed.stage',ROOT/'.runtime/installed.lock',ROOT/'.runtime/installed.previous'] if p.exists()]
    return {'live_private_processes':live,'scratch':scratch,'setup_paths':pending}

def stop_observed(observed):
    table=proc_table()
    for pid,data in observed.items():
        if pid in table and table[pid]['start']==data['start'] and table[pid]['state']!='Z':
            try: os.kill(pid,signal.SIGKILL)
            except ProcessLookupError: pass

def run(args, expected=0, reason=None):
    start = time.monotonic()
    wall_start=time.time()
    p = subprocess.Popen([str(BIN), *args], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    observed = {}; peaks = {'rust': 0, 'node': 0, 'browser': 0}; done = threading.Event()
    def sample():
        while not done.wait(.02):
            table = proc_table(); descendants = track(p.pid, table)
            # The Rust parent inherits the harness's shared group; it is not owned.
            groups = {data['group'] for pid,data in observed.items() if pid != p.pid}
            descendants |= {pid for pid,data in table.items() if data['group'] in groups}
            totals = dict.fromkeys(peaks, 0)
            for pid in descendants:
                if pid not in table: continue
                data=table[pid];observed[pid]=data
                kind='rust' if pid==p.pid else 'node' if data['exe'].endswith('/node') else 'browser'
                totals[kind]+=data['rss']
            for kind in peaks: peaks[kind]=max(peaks[kind], totals[kind])
    thread=threading.Thread(target=sample);thread.start()
    try: out, err=p.communicate(timeout=150)
    except subprocess.TimeoutExpired:
        p.kill();out,err=p.communicate();stop_observed(observed)
        raise AssertionError(f'{args}: timed out: {err}')
    finally: done.set();thread.join()
    time.sleep(.4)
    table=proc_table()
    orphans=[pid for pid,data in observed.items() if pid in table and table[pid]['start']==data['start'] and table[pid]['state']!='Z']
    if orphans:stop_observed(observed)
    assert not orphans, f'Live owned processes remain (scoped cleanup attempted): {orphans}'
    assert p.returncode==expected, f'{args}: exit {p.returncode}: {err}'
    if reason: assert reason in err, f'{args}: wrong rejection reason: {err}'
    result={'command':args,'exit':p.returncode,'expected_reason':reason,'stdout':out,'stderr':err,'seconds':round(time.monotonic()-start,3),'sampled_peak_rss_bytes':peaks,'live_owned_processes_after':0,'observed_process_count':len(observed)}
    if args[0]=='smoke' and expected==0:
        result['actual_capture_evidence']=inspect_captures(since=wall_start)
    print(json.dumps(result),flush=True)
    return result

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--headed',action='store_true');parser.add_argument('--audit-only',action='store_true');args=parser.parse_args()
    ARTIFACTS.mkdir(exist_ok=True)
    if args.audit_only:
        audit=scoped_audit();(ARTIFACTS/'corrections-final-audit.json').write_text(json.dumps({'generated_at_unix':time.time(),**audit},indent=2));print(json.dumps(audit));assert not any(audit.values()),'scoped audit failed';return
    results=[]
    results.append(run(['smoke']+(['--headed'] if args.headed else [])))
    results.append(run(['protocol-test']))
    protected=[ROOT/'.runtime/installed/READY',ARTIFACTS/'public-before.png']
    hashes={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in protected}
    results.append(run(['repair','--cancel-after-ms','100'],expected=1,reason='runtime setup cancelled'))
    assert all(hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()==h for p,h in hashes.items()), 'cancel changed original evidence/ready runtime'
    # Cancel must preserve the ready runtime and remove temporary stage/lock.
    assert not (ROOT/'.runtime/installed.stage').exists()
    assert not (ROOT/'.runtime/installed.lock').exists()
    results.append(run(['smoke']))
    installed=ROOT/'.runtime/installed/worker/src/fixtures.mjs'
    original=installed.read_bytes();installed.write_bytes(original+b'\n// deliberate local corruption\n')
    try:
        results.append(run(['smoke'],expected=1,reason='runtime damaged; repair required'))
        results.append(run(['repair']))
        results.append(run(['smoke']))
    finally:
        if installed.read_bytes()!=original: installed.write_bytes(original)
    if os.uname().sysname=='Linux':
        results.append(run(['actor-test']))
        p=subprocess.Popen([str(BIN),'parent-hang'],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            worker=int(p.stdout.readline().strip());time.sleep(1)
            table=proc_table();children={pid:table[pid] for pid in track(p.pid,table) if pid in table and pid!=p.pid}
            p.kill();p.wait(timeout=3);time.sleep(3)
            table=proc_table();live=[pid for pid,d in children.items() if pid in table and table[pid]['start']==d['start'] and table[pid]['state']!='Z']
            assert not live, f'Parent-death left live processes: {live}'
            results.append({'command':['parent-hang','SIGKILL parent'],'exit':p.returncode,'live_owned_processes_after':0,'observed_process_count':len(children)})
            results.append(run(['protocol-test']))  # restart also recovers app-owned dead scratch
            assert not list(SCRATCH.glob('worker-*')), 'private browser scratch remains after recovery'
        finally:
            if p.poll() is None: p.kill();p.wait()
    audit=scoped_audit()
    assert not any(audit.values()), f'Final scoped audit failed: {audit}'
    evidence={'platform':os.uname().sysname,'generated_at_unix':time.time(),'headed_requested':args.headed,'results':results,'final_scoped_audit':audit,'note':'Development environment only, not a clean-machine or cross-OS qualification. RSS samples are sums, not unique/PSS memory.'}
    (ARTIFACTS/'corrections-qualification.json').write_text(json.dumps(evidence,indent=2))
    print('PASS: captures, fault recovery, setup cancel, tamper/repair, parent death, no live owned processes')

if __name__=='__main__': main()
