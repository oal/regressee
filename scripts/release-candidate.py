#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Frozen-source private Linux app candidate + actual normal-target license texts.

No publication/signing/install/uninstall. Debug build intentionally remains ABI-
unqualified; source/binary identity is checked before/after the actual locked build.
"""
import argparse, gzip, hashlib, io, json, os, re, subprocess, tarfile, tomllib
from pathlib import Path
import source_release as source

def run(argv,root,env):
    r=subprocess.run(argv,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=600)
    if r.returncode:raise source.Refusal('candidate-command-failed')
    return r.stdout

def notices(root,env):
    graph=run(['cargo','tree','--locked','--offline','-p','regressee-gui','--features','ui','--target','x86_64-unknown-linux-gnu','--edges','normal','--prefix','none','--format','{p}'],root,env).decode()
    packages=set(re.findall(r'^([A-Za-z0-9_-]+) v([^\s]+)',graph,re.M));rows=[];texts={}
    cache=Path(os.environ.get('CARGO_HOME',str(Path.home()/'.cargo')))/'registry/src'
    lock=tomllib.loads((root/'Cargo.lock').read_text())['package']
    for name,version in sorted(packages):
        matching=next((p for p in lock if p['name']==name and p['version']==version),{})
        if name.startswith('regressee-'):continue
        candidates=list(cache.glob(f'*/{name}-{version}'))
        declaration=None;found=[]
        if len(candidates)==1:
            p=candidates[0];meta=tomllib.loads((p/'Cargo.toml').read_text())['package'];declaration=meta.get('license')
            for f in sorted(p.iterdir()):
                if f.is_file() and not f.is_symlink() and f.name.upper().startswith(('LICENSE','NOTICE','COPYING')) and f.stat().st_size<=1024*1024:
                    path=f'{name}-{version}/{f.name}';texts[path]=f.read_bytes();found.append({'path':path,'sha256':source.digest(texts[path])})
            if meta.get('license-file') and (p/meta['license-file']).is_file():
                f=p/meta['license-file'];path=f'{name}-{version}/{f.name}';texts[path]=f.read_bytes();found.append({'path':path,'sha256':source.digest(texts[path])})
        rows.append({'name':name,'version':version,'declared_license':declaration,'cargo_checksum':matching.get('checksum'),
            'available_texts':found,'approved':False,'status':'unreviewed' if declaration and found else 'unknown-or-missing-text'})
    return {'target':'x86_64-unknown-linux-gnu','features':['ui'],'edges':'normal','packages':rows,
        'release_approved':False,'native_inventory_complete':False,
        'remaining':['embedded/native/static/system libraries and actual SQLite source disclaimer','Node/browser/codec/FFmpeg if runtime shipped','all redistribution/corresponding-source obligations and owner review']},texts

def bundle(files,epoch):
    b=io.BytesIO()
    with gzip.GzipFile(filename='',mode='wb',fileobj=b,mtime=epoch) as g,tarfile.open(fileobj=g,mode='w',format=tarfile.USTAR_FORMAT) as tar:
        for name,(data,mode) in sorted(files.items()):
            info=tarfile.TarInfo('regressee-private/'+name);info.size=len(data);info.mode=mode;info.mtime=epoch
            tar.addfile(info,io.BytesIO(data))
    return b.getvalue()

def build(root,output,epoch):
    fd=source.physical_dir(str(root))
    try:
        selected=source.collect(fd);source.validate_sources(selected);rows=source.inventory(selected)
        identity=source.digest(source.json_bytes(rows))
        env=dict(os.environ,LIBRARY_PATH=str(root/'.runtime/link-libs'),CARGO_BUILD_JOBS='4')
        log=run(['cargo','build','--locked','-p','regressee-gui','--features','ui','--bin','regressee-desktop'],root,env)
        binary=root/'target/debug/regressee-desktop';data=binary.read_bytes()
        if len(data)>256*1024*1024:raise source.Refusal('app-size-budget')
        inventory,texts=notices(root,env)
        inspect=run(['readelf','--version-info',str(binary)],root,env).decode()
        glibc=sorted(set(re.findall(r'GLIBC_(\d+\.\d+)',inspect)),key=lambda v:tuple(map(int,v.split('.'))))
        needed=run(['readelf','-d',str(binary)],root,env).decode()
        deps=re.findall(r'Shared library: \[([^\]]+)\]',needed)
        if source.inventory(source.collect(fd))!=rows:raise source.Refusal('source-changed-across-build')
        src=source.make_archive(selected,epoch)
        inventory_bytes=source.json_bytes(inventory)
        shipped={'bin/regressee-desktop':(data,0o755),'LICENSE':((root/'LICENSE').read_bytes(),0o644),
            'THIRD_PARTY_NOTICES.md':((root/'THIRD_PARTY_NOTICES.md').read_bytes(),0o644),
            'license-inventory.json':(inventory_bytes,0o644)}
        for name,content in texts.items():shipped['notices/'+name]=(content,0o644)
        for name in source.USER_DOCS:shipped['help/'+name]=((root/'docs'/name).read_bytes(),0o644)
        app=bundle(shipped,epoch)
        if source.inventory(source.collect(fd))!=rows:raise source.Refusal('source-changed-across-package')
        manifest={'schema':1,'kind':'private-debug-candidate-not-release','approved':False,'git_commit':None,
            'source_identity':identity,'source_date_epoch':epoch,'source_files':rows,
            'binary_sha256':source.digest(data),'binary_bytes':len(data),'source_archive_sha256':source.digest(src),
            'app_archive_sha256':source.digest(app),'license_inventory_sha256':source.digest(inventory_bytes),
            'target':'x86_64-unknown-linux-gnu','features':['ui'],'glibc_required':glibc,'dt_needed':deps,
            'ubuntu_24_04_abi_pass':not glibc or tuple(map(int,glibc[-1].split('.')))<=(2,39),
            'runtime_bundled':False,'signed':False,'installer':False,
            'blocked_gates':['minimum Linux ABI/default sandbox package','actual Windows/macOS/native/human/keystore','macOS supervision code gap','maintained authenticated runtime','unreviewed licenses/native sources','owner signing/hosting/naming'],
            'signature_interface':'Owner may sign exact manifest bytes externally; no production private key or trusted identity is supplied.'}
        manifest_bytes=source.json_bytes(manifest)
        blobs={'source.tar.gz':src,'app.tar.gz':app,'manifest.json':manifest_bytes,'license-inventory.json':inventory_bytes,
            'SHA256SUMS':(''.join(f'{source.digest(b)}  {n}\n' for n,b in [('source.tar.gz',src),('app.tar.gz',app),('manifest.json',manifest_bytes),('license-inventory.json',inventory_bytes)])).encode()}
        source.publish(str(output),fd,blobs)
        print(json.dumps({k:manifest[k] for k in ['source_identity','binary_sha256','app_archive_sha256','source_archive_sha256','ubuntu_24_04_abi_pass','approved']}))
    finally:os.close(fd)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--epoch',required=True,type=int);a=p.parse_args()
    if not 0<=a.epoch<=0xffffffff:p.error('epoch outside gzip bound')
    build(a.source.resolve(),a.output,a.epoch)
