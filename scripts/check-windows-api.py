#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Private Windows std/API type-check scaffold; not linking or OS execution.

Uses official Rust 1.98.1 target digest; installs nothing into rustup/system dirs.
Scaffold includes the actual platform and Job implementation, not copied stubs.
"""
import argparse, hashlib, json, os, subprocess, tarfile, tomllib, urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);a=p.parse_args()
a.output.mkdir(mode=0o700,parents=True,exist_ok=False);out=a.output.resolve()
root=Path(__file__).resolve().parent.parent
url='https://static.rust-lang.org/dist/channel-rust-1.98.1.toml'
with urllib.request.urlopen(url,timeout=30) as r:manifest=r.read(2*1024*1024)
(out/'rust-manifest.toml').write_bytes(manifest)
target='x86_64-pc-windows-msvc';entry=tomllib.loads(manifest.decode())['pkg']['rust-std']['target'][target]
pack=out/'rust-std.tar.xz';h=hashlib.sha256();size=0
with urllib.request.urlopen(entry['xz_url'],timeout=30) as response,pack.open('xb') as f:
    while block:=response.read(65536):
        size+=len(block)
        if size>80*1024*1024:raise ValueError('target download budget')
        f.write(block);h.update(block)
if h.hexdigest()!=entry['xz_hash']:raise ValueError('official Rust target checksum mismatch')
sysroot=out/'sysroot';sysroot.mkdir(mode=0o700)
prefix=f'rust-std-1.98.1-{target}/rust-std-{target}/'
with tarfile.open(pack) as t:
    for member in t:
        if not member.name.startswith(prefix+'lib/rustlib/'+target+'/'):continue
        relative=Path(member.name[len(prefix):])
        if relative.is_absolute() or '..' in relative.parts or not (member.isfile() or member.isdir()):raise ValueError('unsafe target member')
        dst=sysroot/relative
        if member.isdir():dst.mkdir(mode=0o700,parents=True,exist_ok=True)
        else:
            dst.parent.mkdir(mode=0o700,parents=True,exist_ok=True)
            with dst.open('xb') as f:f.write(t.extractfile(member).read())
scaffold=out/'scaffold';(scaffold/'src').mkdir(mode=0o700,parents=True)
(scaffold/'Cargo.toml').write_text('''[package]
name="regressee-windows-api-check"
version="0.0.0"
edition="2024"
[dependencies]
anyhow="=1.0.100"
windows-sys={version="=0.61.2",features=["Win32_Foundation","Win32_Storage_FileSystem","Win32_Security","Win32_Security_Authorization","Win32_System_Threading","Win32_System_JobObjects","Win32_System_Pipes","Win32_System_IO"]}
''')
(scaffold/'src/lib.rs').write_text(f'#![allow(dead_code)]\n#[path="{root}/crates/core/src/platform.rs"] pub mod platform;\n#[path="{root}/crates/runtime/src/windows_process.rs"] mod windows_process;\n')
env=dict(os.environ,CARGO_TARGET_DIR=str(out/'target'))
env['CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS']='--sysroot '+str(sysroot)
commands=[['cargo','generate-lockfile','--offline'],['cargo','check','--locked','--offline','--all-targets','--target',target]]
results=[]
for i,cmd in enumerate(commands):
    with (out/f'command-{i}.log').open('xb') as f:result=subprocess.run(cmd,cwd=scaffold,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=180)
    results.append({'command':cmd,'exit':result.returncode})
    if result.returncode:break
(out/'result.json').write_text(json.dumps({'commands':results,'official_target_sha256':h.hexdigest(),'target_bytes':size,
    'source_hashes':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'crates/core/src/platform.rs',root/'crates/core/src/platform/windows.rs',root/'crates/runtime/src/windows_process.rs']},
    'scope':'Actual Windows std/win32 API type-check only; full runtime/app linking, Windows functional ACL/Job/crash tests NOT RUN'},indent=2))
raise SystemExit(results[-1]['exit'])
