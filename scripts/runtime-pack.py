#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Private unsigned v3 pack preparation. No signing keys/downloads/execution.

Output is a new private directory containing manifest.json and payload.zip.
Owner signs manifest.json externally; final assembly is deliberately separate.
Product install revalidates authenticated modes/links before smoke/publication.
"""
import argparse, hashlib, json, os, stat, zipfile, subprocess, tempfile
from pathlib import Path
from source_release import Refusal, safe_relative, physical_dir

def inventory(root):
    files={};links={};modes={};total=0
    # Pin physical root before traversal; callers must own a frozen private tree.
    fd=physical_dir(str(root));os.close(fd)
    def walk(path):
        nonlocal total
        for p in sorted(path.iterdir()):
            name=p.relative_to(root).as_posix();safe_relative(name);s=p.lstat()
            if stat.S_ISLNK(s.st_mode):links[name]=os.readlink(p)
            elif stat.S_ISDIR(s.st_mode):walk(p)
            elif stat.S_ISREG(s.st_mode) and s.st_nlink==1:
                if s.st_size>512*1024*1024:raise Refusal('runtime-file-bound')
                h=hashlib.sha256()
                with p.open('rb') as f:
                    while b:=f.read(65536):h.update(b)
                mode=stat.S_IMODE(s.st_mode)
                if mode & ~0o755 or not mode & 0o400:raise Refusal('unsafe-runtime-mode')
                files[name]={'sha256':h.hexdigest(),'bytes':s.st_size,'executable':bool(mode&0o111)};modes[name]=mode;total+=s.st_size
            else:raise Refusal('special-hardlinked-runtime-entry')
            if len(files)+len(links)>20000 or total>2*1024**3:raise Refusal('runtime-total-bound')
    walk(root)
    folded={}
    for n in files.keys()|links.keys():
        for i in range(1,len(n.split('/'))+1):
            p='/'.join(n.split('/')[:i]);old=folded.setdefault(p.lower(),p)
            if old!=p:raise Refusal('runtime-case-collision')
    for name,value in links.items():
        parts=name.split('/');bundle=next((i for i,p in enumerate(parts) if p.endswith(('.app','.framework'))),None)
        if bundle is None:raise Refusal('runtime-link-outside-bundle')
        boundary='/'.join(parts[:bundle+1])+'/'
        seen=set();resolved=name
        for _ in range(33):
            prefix=next((p for p in links if resolved==p or resolved.startswith(p+'/')),None)
            if prefix is None:break
            if prefix in seen:raise Refusal('runtime-link-cycle')
            seen.add(prefix);value=links[prefix]
            if not value or value.startswith('/') or '\\' in value or ':' in value:raise Refusal('runtime-link-absolute')
            target=prefix.split('/')[:-1]
            for part in value.split('/'):
                if part=='..':
                    if not target:raise Refusal('runtime-link-escape')
                    target.pop()
                elif part=='.':pass
                elif not part:raise Refusal('runtime-link-empty')
                else:target.append(part)
            resolved='/'.join(target)+resolved[len(prefix):];safe_relative(resolved)
            if not resolved.startswith(boundary):raise Refusal('runtime-link-escape')
        else:raise Refusal('runtime-link-depth')
        if resolved not in files and not any(n.startswith(resolved+'/') for n in files):raise Refusal('runtime-link-dangling')
    for n in files.keys()|links.keys():
        if any(n.startswith(p+'/') for p in files.keys()|links.keys()):raise Refusal('runtime-leaf-parent')
    return files,links,modes

def prepare(root,base,output):
    parent=physical_dir(str(output.parent.absolute()))
    try:
        s=os.fstat(parent)
        if s.st_uid!=os.getuid() or stat.S_IMODE(s.st_mode)&0o077:raise Refusal('private-runtime-output-parent-required')
    finally:os.close(parent)
    if output.absolute().is_relative_to(root):raise Refusal('runtime-output-inside-source')
    files,links,modes=inventory(root)
    if any(n in files or n in links for n in ['manifest.json','manifest.sig','READY']):raise Refusal('runtime-control-entry-in-payload')
    manifest=dict(base,version=3,files=files,links=links,modes=modes)
    if not all(k in base for k in ['id','os','arch','scope','support_until','identity','node','worker','browser_root']):raise Refusal('runtime-metadata-incomplete')
    if base['node'] not in files or base['worker'] not in files:raise Refusal('runtime-entrypoint-missing')
    if any(not n.startswith(base['browser_root']+'/') for n in links):raise Refusal('runtime-link-outside-browser')
    output.mkdir(mode=0o700,parents=False,exist_ok=False)
    try:
        (output/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,separators=(',',':'))+'\n')
        with zipfile.ZipFile(output/'payload.zip','x',compression=zipfile.ZIP_STORED,allowZip64=False) as z:
            for n in sorted(files.keys()|links.keys()):
                info=zipfile.ZipInfo(n,(1980,1,1,0,0,0));info.create_system=3
                if n in links:info.external_attr=(stat.S_IFLNK|0o777)<<16;z.writestr(info,links[n].encode())
                else:
                    info.external_attr=(stat.S_IFREG|modes[n])<<16
                    h=hashlib.sha256();count=0
                    with z.open(info,'w') as dst,(root/n).open('rb') as src:
                        while b:=src.read(65536):
                            count+=len(b)
                            if count>files[n]['bytes']:raise Refusal('runtime-file-changed')
                            dst.write(b);h.update(b)
                    if count!=files[n]['bytes'] or h.hexdigest()!=files[n]['sha256']:raise Refusal('runtime-file-changed')
        if inventory(root)!=(files,links,modes):raise Refusal('runtime-source-changed')
        report={'private_candidate':True,'signed':False,'release_approved':False,'source_observation_stable':True,
            'manifest_sha256':hashlib.sha256((output/'manifest.json').read_bytes()).hexdigest(),
            'pack_sha256':hashlib.sha256((output/'payload.zip').read_bytes()).hexdigest(),
            'gates':['external Ed25519 signature/final assembly','upstream provenance','target-specific licenses/sources','native signatures/sandbox/functional qualification']}
        (output/'candidate.json').write_text(json.dumps(report,indent=2))
    except Exception:
        # Retain private incomplete output for inspection; never overwrite/retry it.
        (output/'INCOMPLETE.txt').write_text('Preparation refused; do not sign or install.\n');raise
    return manifest
def assemble(candidate,signature,public_key,output):
    # Verify with owner-supplied PUBLIC key only. No key generation/custody.
    physical=physical_dir(str(output.parent.absolute()))
    try:
        s=os.fstat(physical)
        if s.st_uid!=os.getuid() or stat.S_IMODE(s.st_mode)&0o077:raise Refusal('private-runtime-output-parent-required')
    finally:os.close(physical)
    manifest=(candidate/'manifest.json').read_bytes();sig=signature.read_bytes()
    if len(manifest)>4*1024*1024 or len(sig)!=64:raise Refusal('runtime-signature-size')
    checked=subprocess.run(['/usr/bin/openssl','pkeyutl','-verify','-pubin','-inkey',str(public_key),'-rawin','-in',str(candidate/'manifest.json'),'-sigfile',str(signature)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=15)
    if checked.returncode:raise Refusal('runtime-owner-signature-rejected')
    # Owner signature covers every expected file/hash/mode/link in this manifest;
    # install verifies contents before smoke. No unsigned payload gets executed.
    with tempfile.NamedTemporaryFile(prefix='runtime-final-',dir=output.parent,delete=True) as temp:
        with zipfile.ZipFile(candidate/'payload.zip') as src,zipfile.ZipFile(temp,'w',compression=zipfile.ZIP_STORED,allowZip64=False) as dst:
            for name,content in [('manifest.json',manifest),('manifest.sig',sig)]:
                info=zipfile.ZipInfo(name,(1980,1,1,0,0,0));dst.writestr(info,content)
            seen=set();m=json.loads(manifest)
            if set(m['files'])&set(m['links']):raise Refusal('runtime-final-collision')
            for info in src.infolist():
                name=info.filename;safe_relative(name)
                if name in seen:raise Refusal('runtime-final-duplicate')
                seen.add(name)
                if name in m['links']:
                    if info.file_size>1024 or src.read(info)!=m['links'][name].encode():raise Refusal('runtime-final-link-mismatch')
                    dst.writestr(info,m['links'][name].encode())
                elif name in m['files']:
                    expected=m['files'][name];h=hashlib.sha256();count=0
                    if info.file_size!=expected['bytes']:raise Refusal('runtime-final-size')
                    if (info.external_attr>>16)&0o7777!=m['modes'][name]:raise Refusal('runtime-final-mode')
                    with src.open(info) as read,dst.open(info,'w') as write:
                        while b:=read.read(65536):count+=len(b);h.update(b);write.write(b)
                    if count!=expected['bytes'] or h.hexdigest()!=expected['sha256']:raise Refusal('runtime-final-hash')
                else:raise Refusal('runtime-final-unlisted')
            if seen!=m['files'].keys()|m['links'].keys():raise Refusal('runtime-final-missing')
        temp.flush();os.fsync(temp.fileno())
        # Exact owner-private same-volume no-clobber file publication; no replace.
        os.link(temp.name,output,follow_symlinks=False)
        fd=physical_dir(str(output.parent.absolute()))
        try:os.fsync(fd)
        finally:os.close(fd)
    return hashlib.sha256(output.read_bytes()).hexdigest()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path);p.add_argument('--manifest-base',type=Path);p.add_argument('--output',required=True,type=Path)
    p.add_argument('--assemble-candidate',type=Path);p.add_argument('--signature',type=Path);p.add_argument('--public-key',type=Path);a=p.parse_args()
    if a.assemble_candidate:
        if not a.signature or not a.public_key:p.error('signature and public-key required for assembly')
        print(assemble(a.assemble_candidate,a.signature,a.public_key,a.output))
    else:
        if not a.root or not a.manifest_base:p.error('root and manifest-base required for preparation')
        prepare(a.root.absolute(),json.loads(a.manifest_base.read_text()),a.output)
