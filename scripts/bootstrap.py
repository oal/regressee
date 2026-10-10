#!/usr/bin/env python3
"""Developer-only pack builder. Never installs host Node/npm or OS packages."""
import argparse, hashlib, json, os, pathlib, platform, shutil, subprocess, tarfile, urllib.request, zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / '.runtime/source'
VERSION = '22.20.0'
TARGETS = {
    ('Linux', 'x86_64'): ('linux-x64.tar.xz', '00bbd05e306ea68b6e13e17360d0e2f680b493ef95f2fea1c4296ff7437530bc'),
    ('Darwin', 'arm64'): ('darwin-arm64.tar.xz', 'f2ed1fbdfb79afb9343dfbb609efde657b41e7efcd5215264ce78f8906356b4d'),
    ('Darwin', 'x86_64'): ('darwin-x64.tar.xz', '2a291f0a9555f5d6685d96ce9429da3d0ea3cd896c012702c6ee0015f818684e'),
    ('Windows', 'AMD64'): ('win-x64.zip', 'bb819d6eb8f5bfda294bbc83a7e4ec6539da67c4233d54b0d655b9248b15e29d'),
}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--linux-compat', choices=['ubuntu24.04-x64'], help='Explicit unsupported-host development probe; does not qualify the host')
    args = parser.parse_args()
    suffix, digest = TARGETS[(platform.system(), platform.machine())]
    SOURCE.mkdir(parents=True, exist_ok=True)
    archive = ROOT / '.runtime' / ('node-' + suffix)
    if not archive.exists() or hashlib.sha256(archive.read_bytes()).hexdigest() != digest:
        print('Downloading pinned private Node', flush=True)
        with urllib.request.urlopen(f'https://nodejs.org/dist/v{VERSION}/node-v{VERSION}-{suffix}', timeout=60) as r:
            with archive.with_suffix('.partial').open('wb') as out:
                shutil.copyfileobj(r, out)
        assert hashlib.sha256(archive.with_suffix('.partial').read_bytes()).hexdigest() == digest
        archive.with_suffix('.partial').replace(archive)
    node_dir = SOURCE / 'node'
    if not node_dir.exists():
        extract = ROOT / '.runtime/node-extract'
        extract.mkdir(exist_ok=True)
        if suffix.endswith('.zip'):
            with zipfile.ZipFile(archive) as z:
                for name in z.namelist():
                    assert not name.startswith('/') and '..' not in pathlib.PurePosixPath(name).parts
                z.extractall(extract)
        else:
            with tarfile.open(archive) as t:
                t.extractall(extract, filter='data')
        (extract / f'node-v{VERSION}-{suffix.split(".")[0]}').rename(node_dir)
    win = platform.system() == 'Windows'
    node = node_dir / ('node.exe' if win else 'bin/node')
    npm = node_dir / ('node_modules/npm/bin/npm-cli.js' if win else 'lib/node_modules/npm/bin/npm-cli.js')
    worker = SOURCE / 'worker'
    worker.mkdir(exist_ok=True)
    for name in ('package.json', 'package-lock.json'):
        if (ROOT / 'worker' / name).exists():
            shutil.copy2(ROOT / 'worker' / name, worker / name)
    env = {**os.environ, 'PLAYWRIGHT_BROWSERS_PATH': str(SOURCE / 'browsers'),
           'npm_config_cache': str(ROOT / '.runtime/npm-cache'), 'PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD': '1'}
    env['PATH'] = str(node.parent) + os.pathsep + env.get('PATH', '')
    if args.linux_compat:
        assert platform.system() == 'Linux' and platform.machine() == 'x86_64'
        print('EXPERIMENTAL: requesting Ubuntu 24.04 browser on this unsupported host; sandbox remains enabled', flush=True)
        env['PLAYWRIGHT_HOST_PLATFORM_OVERRIDE'] = args.linux_compat
    if not (worker / 'package-lock.json').exists():
        subprocess.run([str(node), str(npm), 'install', '--package-lock-only', '--ignore-scripts'], cwd=worker, env=env, check=True)
        shutil.copy2(worker / 'package-lock.json', ROOT / 'worker/package-lock.json')
    subprocess.run([str(node), str(npm), 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], cwd=worker, env=env, check=True)
    shutil.copytree(ROOT / 'worker/src', worker / 'src', dirs_exist_ok=True)
    shutil.copytree(ROOT / 'worker/test', worker / 'test', dirs_exist_ok=True)
    subprocess.run([str(node), str(worker / 'node_modules/playwright/cli.js'), 'install', '--no-shell', 'chromium'], env=env, check=True)
    print('Private source ready. Next: cargo run --bin regressee-spike -- pack')

if __name__ == '__main__':
    main()
