#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Read-only scoped final hashes/CAS/privacy/resource evidence audit (no kills)."""
import argparse,hashlib,json,os,sqlite3
import source_release
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--evidence',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--candidate',type=Path);a=p.parse_args()
root=Path(__file__).resolve().parent.parent;failures=[];results={}
def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        while b:=f.read(65536):h.update(b)
    return h.hexdigest()
for name,value in json.loads((a.evidence/'historical-docs-hashes.json').read_text()).items():
    if digest(root/name)!=value:failures.append('historical-doc-changed:'+name)
results['historical_docs']='unchanged' if not failures else 'failed'
runtime=json.loads((a.evidence/'retained-runtime-input.json').read_text())
installed=Path(runtime['runtime']);m=json.loads((installed/'manifest.json').read_text())
if digest(installed/'manifest.json')!=runtime['manifest_sha256']:failures.append('historical-runtime-manifest')
for name,entry in m['files'].items():
    path=installed/name
    if path.stat().st_size!=entry['bytes'] or digest(path)!=entry['sha256']:failures.append('historical-runtime-file:'+name)
results['historical_runtime_files']=len(m['files'])
databases=sorted((a.evidence/'soak/data').glob('**/regressee.sqlite3'))
if len(databases)!=1:failures.append('expected-one-soak-database')
for db in databases:
    conn=sqlite3.connect(f'file:{db}?mode=ro',uri=True)
    if conn.execute('PRAGMA integrity_check').fetchone()[0]!='ok':failures.append('sqlite-integrity')
    if conn.execute('PRAGMA foreign_key_check').fetchall():failures.append('sqlite-foreign-keys')
    rows=conn.execute('SELECT hash,bytes FROM artifacts').fetchall()
    for name,size in rows:
        path=db.parent/'artifacts'/name
        if not path.is_file() or path.is_symlink() or path.stat().st_size!=size or digest(path)!=name:failures.append('cas-integrity:'+name)
    results['soak_artifacts_checked']=len(rows)
    results['soak_artifact_bytes']=sum(size for _,size in rows)
    results['soak_projects']=conn.execute('SELECT count(*) FROM projects').fetchone()[0]
    results['soak_runs']=conn.execute('SELECT count(*) FROM runs').fetchone()[0]
    results['soak_baselines']=conn.execute('SELECT count(*) FROM baselines').fetchone()[0]
    results['pending_gc']=conn.execute('SELECT count(*) FROM pending_gc').fetchone()[0]
    if results['pending_gc']:failures.append('pending-gc')
    conn.close()
command=a.evidence/'soak/command/result.json'
if command.exists():
    settled=json.loads(command.read_text())
    if settled['remaining_children'] or settled['forced_children'] or settled['timeout']:failures.append('soak-command-not-cleanly-settled')
    results['soak_owned_children']=settled['remaining_children'];results['soak_reaped']=len(settled['reaped'])
    results['soak_command_exit']=settled['exit']
    for start in [a.evidence/'soak/command/start.json']:
        pid=json.loads(start.read_text())['pid']
        if Path(f'/proc/{pid}').exists():
            # A reused PID is not a live owned process. Compare kernel birth value.
            old=json.loads(start.read_text())['stat'].rsplit(')',1)[1].split()[19]
            now=Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()[19]
            if old==now:failures.append('owned-soak-command-still-alive')
else:failures.append('missing-soak-command-settlement')
coverage=json.loads((a.evidence/'soak/data/coverage.json').read_text())
paused=json.loads((a.evidence/'soak/data/paused.json').read_text())
completed=json.loads((a.evidence/'soak/data/completed.json').read_text())
captures={json.dumps(c['metadata']['key'],sort_keys=True):c for c in completed['captures']}
if coverage['captured']!=len(captures) or coverage['visits']!=coverage['captured']:failures.append('coverage-capture-count-mismatch')
for c in paused['captures']:
    if captures.get(json.dumps(c['metadata']['key'],sort_keys=True))!=c:failures.append('paused-capture-changed')
results['pause_retained_captures']=len(paused['captures'])
results['resume_added_captures']=len(completed['captures'])-len(paused['captures'])
results['coverage']=coverage
results['maximum_scale_gate_pass']=settled['exit']==0 and coverage['captured']==3000 and coverage['visits']==3000
results['scope']='resource/data/history integrity only; a failed coverage gate is NOT a successful soak or release'
pending=list((a.evidence/'soak/data').glob('**/*.pending'))
results['pending_files']=len(pending)
if pending:failures.append('pending-write-files')
results['privacy']='diagnostic structural/schema tests required separately; this audit never exports databases/logs/screenshots'
product_databases=sorted((a.evidence/'product-analysis-data').glob('**/regressee.sqlite3'))
if len(product_databases)!=1:failures.append('expected-one-product-regression-database')
for db in product_databases:
    conn=sqlite3.connect(f'file:{db}?mode=ro',uri=True)
    if conn.execute('PRAGMA integrity_check').fetchone()[0]!='ok':failures.append('product-sqlite-integrity')
    if conn.execute('PRAGMA foreign_key_check').fetchall():failures.append('product-sqlite-foreign-keys')
    rows=conn.execute('SELECT hash,bytes FROM artifacts').fetchall()
    for name,size in rows:
        path=db.parent/'artifacts'/name
        if not path.is_file() or path.is_symlink() or path.stat().st_size!=size or digest(path)!=name:failures.append('product-cas-integrity:'+name)
    results['product_artifacts_checked']=len(rows)
    results['product_runs']=conn.execute('SELECT count(*) FROM runs').fetchone()[0]
    results['product_baselines']=conn.execute('SELECT count(*) FROM baselines').fetchone()[0]
    if conn.execute('SELECT count(*) FROM pending_gc').fetchone()[0]:failures.append('product-pending-gc')
    conn.close()
commands=[]
for path in sorted(a.evidence.glob('*/result.json')):
    record=json.loads(path.read_text())
    if 'remaining_children' not in record:continue
    if record['remaining_children'] or record['forced_children'] or record['timeout']:failures.append('unsettled-command:'+path.parent.name)
    commands.append({'name':path.parent.name,'exit':record['exit'],'reaped':len(record['reaped'])})
results['settled_commands']=commands
results['command_scope']='failed checks remain failures; this list checks settlement, not aggregate test acceptance'
if a.candidate:
    manifest=json.loads((a.candidate/'manifest.json').read_text())
    fd=source_release.physical_dir(str(root))
    try:
        if source_release.inventory(source_release.collect(fd))!=manifest['source_files']:failures.append('candidate-source-changed')
    finally:os.close(fd)
    if digest(root/'target/debug/regressee-desktop')!=manifest['binary_sha256']:failures.append('candidate-binary-changed')
    for line in (a.candidate/'SHA256SUMS').read_text().splitlines():
        expected,name=line.split('  ',1)
        if name not in {'source.tar.gz','app.tar.gz','manifest.json','license-inventory.json'} or digest(a.candidate/name)!=expected:failures.append('candidate-hash-mismatch')
    results['private_candidate_binding']='checked; NOT release approval'
results['failures']=failures;results['pass']=not failures
with a.output.open('x') as f:json.dump(results,f,indent=2)
print(json.dumps(results,indent=2));raise SystemExit(bool(failures))
