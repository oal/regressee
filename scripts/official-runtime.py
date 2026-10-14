#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Build the app-trusted official-source catalogs, never execute vendor code.

Only the worker/source ZIP and catalogs enter the repository. Node and Chrome
archives stay in a private temporary directory. This is a release/build tool,
not a runtime installer. Regeneration changes the app's explicit trust root.
"""
import argparse
import base64
import concurrent.futures
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import tarfile
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'crates/runtime/assets/official'
TEMP = Path('/path/to/private-tests/opencode')
NODE = '24.21.0'
PW = '1.64.0'
WORKER = '5.0.1'
NPM_KEY_ID = 'SHA256:DhQ8wR5APBvFHLF/+Tc+AYvPOdTpcIDqOhxsBHRwC7U'
NPM_KEY = 'MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEY6Ya7W++7aUPzvMTrezH6Ycx3c+HOKYCcNGybJZSCJq/fd7Qa8uuAKtdIkUQtQiEKERhAmE5lMMJhP8OkDOa2g=='
# 2027-01-10 UTC: pin updates require a new reviewed catalog, never "latest".
SUPPORT_UNTIL = 1799539200
SRI = {
    'playwright': 'sha512-kVzTnYFYEyQ9RGzBjNCd2+yEqWmVbxl3AHPynv3qQ6JTjv4a+kwRcnqQwIpuP3NnrnIrE/Q6wWAIAvMfIE2/Yg==',
    'playwright-core': 'sha512-T9r+MZkTECl2+oUcZ26YgZzTrYidTsIhYYkJix+6iDOymOb1FB2RLLSOxRGnZGv5DStt+aduIVbt+E8WygXYpg==',
}
NODE_KEYS = (
    '5BE8A3F6C8A5C01D106C0AD820B1A390B168D356',
    'DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7',
    'CC68F5A3106FF448322E48ED27F5E38D5B0A215F',
    '890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4',
    'C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C',
    '108F52B48DB57BB0CC439B2997B01419BD92F80A',
    '655F3B5C1FB3FA8D1A0CA6BDE4A7D232B936D2FD',
    'A363A499291CBBC940DD62E41F10027AF002F8B0',
)
PLATFORMS = {
    'linux-x86_64': ('linux', 'x86_64', 'linux-x64', 'linux64'),
    'windows-x86_64': ('windows', 'x86_64', 'win-x64', 'win64'),
    'macos-aarch64': ('macos', 'aarch64', 'darwin-arm64', 'mac-arm64'),
    'macos-x86_64': ('macos', 'x86_64', 'darwin-x64', 'mac-x64'),
}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError(f'Unreviewed redirect: {req.full_url} -> {newurl}')


OPENER = urllib.request.build_opener(NoRedirect())


def download(url, destination, limit=600 * 1024 * 1024):
    if not url.startswith('https://'):
        raise ValueError('HTTPS required')
    if destination.exists():
        return destination
    partial = destination.with_name(destination.name + '.partial')
    request = urllib.request.Request(url, headers={'User-Agent': 'Regressee-official-catalog/1'})
    with OPENER.open(request, timeout=90) as response, partial.open('wb') as output:
        count = 0
        while chunk := response.read(1024 * 1024):
            count += len(chunk)
            if count > limit:
                raise ValueError('Download exceeds budget')
            output.write(chunk)
        declared = response.headers.get('Content-Length')
        if declared is not None and int(declared) != count:
            raise ValueError('Truncated download')
    os.replace(partial, destination)
    print(f'Fetched {destination.name}: {count} bytes', flush=True)
    return destination


def fetch_json(url, destination):
    return json.loads(download(url, destination, 20 * 1024 * 1024).read_bytes())


def digest_file(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def safe_name(name):
    parts = name.split('/')
    if (not name or len(name) > 1024 or any(p in ('', '.', '..') for p in parts)
            or any(ord(c) < 32 or c in '\\:<>"|?*' for c in name)):
        raise ValueError(f'Unsafe archive path: {name!r}')
    for p in parts:
        if p.endswith(('.', ' ')) or re.fullmatch(r'(?i)(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?', p):
            raise ValueError(f'Nonportable archive path: {name!r}')
    return name


def check_names(names):
    seen, dirs = set(), {}
    for name in names:
        safe_name(name)
        if name.lower() in seen:
            raise ValueError(f'Colliding archive path: {name}')
        seen.add(name.lower())
        for parent in list(PurePosixPath(name).parents)[:-1]:
            parent = str(parent)
            old = dirs.setdefault(parent.lower(), parent)
            if old != parent:
                raise ValueError('Case-colliding directory')
    if seen.intersection(dirs):
        raise ValueError('Archive leaf used as directory')


def check_archive_names(members):
    """Validate all metadata, including explicit directories, before reading."""
    seen, leaves, directories = {}, set(), {}
    for raw, directory in members:
        name = raw.rstrip('/') if directory else raw
        safe_name(name)
        if name.lower() in seen:
            raise ValueError(f'Duplicate/case-colliding archive entry: {raw}')
        seen[name.lower()] = name
        if directory:
            if directories.setdefault(name.lower(), name) != name:
                raise ValueError('Case-colliding explicit archive directory')
        else:
            leaves.add(name.lower())
        for parent in list(PurePosixPath(name).parents)[:-1]:
            parent = str(parent)
            old = directories.setdefault(parent.lower(), parent)
            if old != parent:
                raise ValueError('Case-colliding archive directory')
    if leaves.intersection(directories):
        raise ValueError('Archive leaf used as directory')


def file_entry(data, mode):
    executable = bool(mode & 0o111)
    return {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data), 'executable': executable}


def verify_node_sums(work):
    base = f'https://nodejs.org/dist/v{NODE}/'
    sums = download(base + 'SHASUMS256.txt', work / 'SHASUMS256.txt', 1024 * 1024)
    signed = download(base + 'SHASUMS256.txt.asc', work / 'SHASUMS256.txt.asc', 1024 * 1024)
    verification = {'status': 'TLS checksums only; GPG unavailable', 'allowed_primary_fingerprints': NODE_KEYS}
    if shutil.which('gpg'):
        home = work / 'gnupg'
        home.mkdir(mode=0o700, exist_ok=True)
        command = ['gpg', '--batch', '--yes', '--no-auto-key-retrieve', '--homedir', str(home)]
        for fingerprint in NODE_KEYS:
            key = download(f'https://raw.githubusercontent.com/nodejs/release-keys/main/keys/{fingerprint}.asc', work / (fingerprint + '.asc'), 1024 * 1024)
            listing = subprocess.run(command + ['--with-colons', '--show-keys', str(key)], check=True, capture_output=True, text=True)
            primary = next(line.split(':')[9] for line in listing.stdout.splitlines() if line.startswith('fpr:'))
            if primary != fingerprint:
                raise ValueError('Node release key fingerprint mismatch')
            subprocess.run(command + ['--import', str(key)], check=True, capture_output=True)
        verified = work / 'SHASUMS256.verified.txt'
        result = subprocess.run(command + ['--status-fd', '1', '--output', str(verified), '--decrypt', str(signed)], check=True, capture_output=True, text=True)
        valid = [line for line in result.stdout.splitlines() if line.startswith('[GNUPG:] VALIDSIG ')]
        if not valid or not any(line.split()[-1] in NODE_KEYS or line.split()[2] in NODE_KEYS for line in valid):
            raise ValueError('Node checksum signer outside pinned release keys')
        # Clear-sign decryption retains a publisher-added final blank line.
        # Compare all nonblank checksum records, not that framing newline.
        if [line for line in verified.read_bytes().splitlines() if line] != [line for line in sums.read_bytes().splitlines() if line]:
            raise ValueError('Signed Node checksum content differs')
        verification = {'status': 'GPG verified', 'validsig': valid, 'allowed_primary_fingerprints': NODE_KEYS}
    return dict(line.split()[::-1] for line in sums.read_text().splitlines()), verification


def verify_npm_signature(work, name, meta):
    if not shutil.which('openssl'):
        return 'Pinned SHA-512 SRI verified; npm registry signature not verified (OpenSSL unavailable)'
    keys = fetch_json('https://registry.npmjs.org/-/npm/v1/keys', work / 'npm-registry-keys.json')
    key = next(k for k in keys['keys'] if k['keyid'] == NPM_KEY_ID)
    if key['key'] != NPM_KEY or key['keytype'] != 'ecdsa-sha2-nistp256':
        raise ValueError('npm registry signing key mismatch')
    pem = work / 'npm-registry-key.pem'
    pem.write_text('-----BEGIN PUBLIC KEY-----\n' + NPM_KEY + '\n-----END PUBLIC KEY-----\n')
    message = work / (name + '-signed-message.txt')
    message.write_text(f'{name}@{PW}:{meta["dist"]["integrity"]}')
    for index, signature in enumerate(meta['dist'].get('signatures', [])):
        if signature['keyid'] != NPM_KEY_ID:
            continue
        sig = work / f'{name}-{index}.sig'
        sig.write_bytes(base64.b64decode(signature['sig'], validate=True))
        result = subprocess.run(['openssl', 'dgst', '-sha256', '-verify', str(pem), '-signature', str(sig), str(message)], capture_output=True)
        if result.returncode == 0:
            return 'Pinned SHA-512 SRI and npm registry ECDSA signature verified; no separate publisher/Sigstore attestation verification'
    raise ValueError(f'No valid npm registry signature for {name}')


def npm_packages(work):
    entries, metadata = {}, {}
    for name, integrity in SRI.items():
        meta = fetch_json(f'https://registry.npmjs.org/{name}/{PW}', work / f'{name}-metadata.json')
        url = f'https://registry.npmjs.org/{name}/-/{name}-{PW}.tgz'
        if meta['version'] != PW or meta['dist']['integrity'] != integrity or meta['dist']['tarball'] != url:
            raise ValueError('npm pin mismatch')
        archive = download(url, work / f'{name}-{PW}.tgz', 30 * 1024 * 1024)
        data = archive.read_bytes()
        if base64.b64encode(hashlib.sha512(data).digest()).decode() != integrity.removeprefix('sha512-'):
            raise ValueError('npm SHA-512 integrity mismatch')
        # No extraction and no install/lifecycle scripts. All published sources,
        # data, licenses and entry points are retained, not a hand-selected API.
        with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as tar:
            members = tar.getmembers()
            check_archive_names((m.name, m.isdir()) for m in members)
            for member in members:
                if member.isdir():
                    continue
                if not member.isfile() or not member.name.startswith('package/'):
                    raise ValueError('Unexpected npm member kind/root')
                path = 'worker/node_modules/' + name + '/' + member.name.removeprefix('package/')
                entries[path] = (tar.extractfile(member).read(), 0o755 if member.mode & 0o111 else 0o644)
        metadata[name] = {'metadata': meta, 'archive_sha256': hashlib.sha256(data).hexdigest(), 'archive_bytes': len(data), 'verification': verify_npm_signature(work, name, meta)}
    return entries, metadata


def worker_payload(entries):
    product = ROOT / 'worker/product'
    package = json.loads((product / 'package.json').read_text())
    if package['dependencies'] != {'playwright': PW}:
        raise ValueError('Worker dependency pin mismatch')
    for source in sorted((product / 'src').rglob('*')):
        if source.is_symlink():
            raise ValueError('Worker source symlink')
        if source.is_file():
            entries['worker/' + source.relative_to(product).as_posix()] = (source.read_bytes(), 0o644)
    for name in ('package.json', 'package-lock.json'):
        entries['worker/' + name] = ((product / name).read_bytes(), 0o644)
    entries['worker/LICENSE'] = ((ROOT / 'LICENSE').read_bytes(), 0o644)
    check_names(entries)
    output = OUT / 'bundled-worker.zip'
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, (data, mode) in sorted(entries.items()):
            info = zipfile.ZipInfo(name, (2000, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return digest_file(output)


def node_inventory(path, prefix, windows):
    only = 'node.exe' if windows else 'bin/node'
    if windows:
        with zipfile.ZipFile(path) as archive:
            check_archive_names((i.filename, i.is_dir()) for i in archive.infolist())
            member = archive.getinfo(prefix + '/' + only)
            if member.is_dir() or stat.S_IFMT(member.external_attr >> 16) not in (0, stat.S_IFREG):
                raise ValueError('Node ZIP entry is not regular')
            data = archive.read(member)
    else:
        with tarfile.open(path, 'r:gz') as archive:
            # Unselected vendor npm aliases remain inert, unmaterialized TAR
            # metadata. Only the uniquely named regular Node member is read.
            selected = [m for m in archive.getmembers() if m.name == prefix + '/' + only]
            if len(selected) != 1:
                raise ValueError('Node TAR entry missing or duplicated')
            member = selected[0]
            if not member.isfile():
                raise ValueError('Node entry is not regular')
            data = archive.extractfile(member).read()
    return 'node/' + only, file_entry(data, 0o755)


def resolve_link(name, target):
    if not target or target.startswith('/') or any(ord(c) < 32 or c in '\\:<>"|?*' for c in target):
        raise ValueError('Unsafe browser link target')
    parts = name.split('/')[:-1]
    for part in target.split('/'):
        if part == '..':
            if not parts:
                raise ValueError('Browser link escapes root')
            parts.pop()
        elif part == '.':
            continue
        elif part:
            parts.append(part)
        else:
            raise ValueError('Empty link component')
    return safe_name('/'.join(parts))


def browser_inventory(path, prefix, destination):
    files, modes, links = {}, {}, {}
    with zipfile.ZipFile(path) as archive:
        members = archive.infolist()
        check_archive_names((i.filename, i.is_dir()) for i in members)
        for member in members:
            original = member.external_attr >> 16
            kind = stat.S_IFMT(original)
            if member.is_dir():
                if member.file_size or kind not in (0, stat.S_IFDIR):
                    raise ValueError('Unsafe Chrome directory metadata')
            elif kind not in (0, stat.S_IFREG, stat.S_IFLNK):
                raise ValueError('Unexpected Chrome ZIP member kind')
            if member.flag_bits & (1 | 64 | 8192):
                raise ValueError('Encrypted Chrome ZIP unsupported')
        for member in members:
            if not member.filename.startswith(prefix + '/'):
                raise ValueError('Chrome ZIP root mismatch')
            if member.is_dir():
                continue
            name = destination + '/' + member.filename.removeprefix(prefix + '/')
            original = member.external_attr >> 16
            kind = stat.S_IFMT(original)
            if kind not in (0, stat.S_IFREG, stat.S_IFLNK):
                raise ValueError('Unexpected Chrome ZIP member kind')
            if stat.S_ISLNK(original):
                links[name] = archive.read(member).decode('utf-8')
            else:
                mode = 0o755 if original & 0o111 or name.endswith('.exe') else 0o644
                with archive.open(member) as stream:
                    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
                files[name] = {'sha256': digest, 'bytes': member.file_size, 'executable': bool(mode & 0o111)}
                modes[name] = mode
    check_names(list(files) + list(links))
    directories = {str(p) for name in files for p in PurePosixPath(name).parents}
    for name, target in links.items():
        boundary = next(('/'.join(name.split('/')[:i + 1]) + '/' for i, part in enumerate(name.split('/')) if part.endswith(('.app', '.framework'))), None)
        if boundary is None:
            raise ValueError('Link outside signed app/framework')
        resolved, visited = resolve_link(name, target), set()
        while True:
            if not resolved.startswith(boundary) or resolved in visited or len(visited) >= 32:
                raise ValueError('Browser link boundary/cycle/depth violation')
            visited.add(resolved)
            alias = next((p for p in sorted(links) if resolved == p or resolved.startswith(p + '/')), None)
            if alias is None:
                break
            resolved = resolve_link(alias, links[alias]) + resolved[len(alias):]
        if resolved not in files and resolved not in directories:
            raise ValueError('Dangling browser link')
    return files, modes, links


def catalog(platform, work, sums, browser, cft, entries, bundled_sha):
    os_name, arch, node_platform, chrome_platform = PLATFORMS[platform]
    node_prefix = f'node-v{NODE}-{node_platform}'
    suffix = '.zip' if os_name == 'windows' else '.tar.gz'
    node_name = node_prefix + suffix
    node_url = f'https://nodejs.org/dist/v{NODE}/{node_name}'
    chrome_prefix = f'chrome-{chrome_platform}'
    chrome_url = f'https://storage.googleapis.com/chrome-for-testing-public/{browser["browserVersion"]}/{chrome_platform}/{chrome_prefix}.zip'
    expected_url = next(item['url'] for item in cft['downloads']['chrome'] if item['platform'] == chrome_platform)
    if chrome_url != expected_url:
        raise ValueError('Chrome official metadata URL mismatch')
    node_path = download(node_url, work / node_name)
    if digest_file(node_path) != sums[node_name]:
        raise ValueError('Node archive SHA-256 mismatch')
    chrome_path = download(chrome_url, work / f'{chrome_prefix}.zip')
    node_entry, node_file = node_inventory(node_path, node_prefix, os_name == 'windows')
    destination = f'browsers/chromium-{browser["revision"]}/{chrome_prefix}'
    files, modes, links = browser_inventory(chrome_path, chrome_prefix, destination)
    files[node_entry], modes[node_entry] = node_file, 0o755
    for name, (data, mode) in entries.items():
        files[name], modes[name] = file_entry(data, mode), mode
    check_names(list(files) + list(links))
    identity = {'Pinned': {'node': NODE, 'playwright': PW, 'browser': browser['browserVersion'], 'worker': WORKER}}
    result = {
        'manifest': {'version': 3, 'id': f'official-{NODE}-{PW}-{WORKER}-{platform}', 'os': os_name, 'arch': arch, 'scope': 'OfficialPinned', 'support_until': SUPPORT_UNTIL, 'identity': identity, 'node': node_entry, 'worker': 'worker/src/index.mjs', 'browser_root': 'browsers', 'files': files, 'links': links, 'modes': modes},
        'components': [
            {'url': node_url, 'sha256': sums[node_name], 'bytes': node_path.stat().st_size, 'format': 'Zip' if os_name == 'windows' else 'TarGz', 'prefix': node_prefix, 'destination': 'node', 'only': 'node.exe' if os_name == 'windows' else 'bin/node'},
            {'url': chrome_url, 'sha256': digest_file(chrome_path), 'bytes': chrome_path.stat().st_size, 'format': 'Zip', 'prefix': chrome_prefix, 'destination': destination, 'only': None},
        ],
        'bundled_sha256': bundled_sha,
    }
    write_json(OUT / (platform + '.json'), result)
    print(f'Catalog {platform}: {len(files)} files, {len(links)} links', flush=True)
    return {'platform': platform, 'components': result['components'], 'catalog_sha256': digest_file(OUT / (platform + '.json'))}


def write_json(path, value):
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work-dir', type=Path, help='Reuse a private download directory from a previous attempt')
    parser.add_argument('--platform', action='append', choices=PLATFORMS, help='Default: all four platforms')
    args = parser.parse_args()
    os.umask(0o077)
    work = args.work_dir or Path(tempfile.mkdtemp(prefix='regressee-official-assets-', dir=TEMP))
    if not work.is_dir() or work.is_symlink() or work.parent != TEMP or not work.name.startswith('regressee-official-assets-') or stat.S_IMODE(work.stat().st_mode) != 0o700:
        raise ValueError('Downloads require the approved private 0700 directory')
    if shutil.disk_usage(work).free < 3 * 1024 ** 3:
        raise ValueError('At least 3 GiB free required')
    print(f'Private archive directory: {work}', flush=True)
    OUT.mkdir(parents=True, exist_ok=True)
    entries, npm = npm_packages(work)
    browser_data = json.loads(entries['worker/node_modules/playwright-core/browsers.json'][0])
    browser = next(b for b in browser_data['browsers'] if b['name'] == 'chromium')
    cft = fetch_json(f'https://googlechromelabs.github.io/chrome-for-testing/{browser["browserVersion"]}.json', work / 'chrome-for-testing.json')
    if cft['version'] != browser['browserVersion']:
        raise ValueError('Chrome metadata version mismatch')
    sums, verification = verify_node_sums(work)
    bundled_sha = worker_payload(entries)
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(catalog, p, work, sums, browser, cft, entries, bundled_sha) for p in (args.platform or PLATFORMS)]
        catalogs = [future.result() for future in futures]
    provenance = {'schema': 1, 'as_of': '2026-10-10', 'node': NODE, 'playwright': PW, 'worker': WORKER, 'chromium': browser, 'bundled_sha256': bundled_sha, 'npm': npm, 'node_checksum_verification': verification, 'node_checksums_sha256': digest_file(work / 'SHASUMS256.txt'), 'node_signed_checksums_sha256': digest_file(work / 'SHASUMS256.txt.asc'), 'chrome_for_testing': cft, 'browser_verification': 'Official Google HTTPS URL, cross-checked with Chrome for Testing metadata and Playwright browsers.json; computed SHA-256 pinned in app catalog, no independent publisher checksum/signature supplied', 'catalogs': catalogs}
    provenance['npm_registry_signing_key'] = {'keyid': NPM_KEY_ID, 'public_key_der_base64': NPM_KEY}
    write_json(OUT / 'provenance.json', provenance)
    licenses = OUT / 'LICENSES'
    licenses.mkdir(exist_ok=True)
    for package in SRI:
        for name in ('LICENSE', 'NOTICE'):
            key = f'worker/node_modules/{package}/{name}'
            if key in entries:
                (licenses / (package + '-' + name)).write_bytes(entries[key][0])
    # Node's license includes its bundled third-party notices. Read only from an
    # archive whose hash was checked against the authenticated release sums.
    first_platform = (args.platform or list(PLATFORMS))[0]
    node_platform = PLATFORMS[first_platform][2]
    prefix = f'node-v{NODE}-{node_platform}'
    if first_platform.startswith('windows-'):
        with zipfile.ZipFile(work / (prefix + '.zip')) as archive:
            node_license = archive.read(prefix + '/LICENSE')
    else:
        with tarfile.open(work / (prefix + '.tar.gz'), 'r:gz') as archive:
            node_license = archive.extractfile(prefix + '/LICENSE').read()
    (licenses / 'Node-LICENSE').write_bytes(node_license)
    (licenses / 'Regressee-MPL-2.0.txt').write_bytes((ROOT / 'LICENSE').read_bytes())
    # Source artifacts are readable in the checkout, archives remain private.
    for path in OUT.rglob('*'):
        if path.is_file():
            path.chmod(0o644)
        elif path.is_dir():
            path.chmod(0o755)
    OUT.chmod(0o755)
    print(f'Bundled worker SHA-256: {bundled_sha}', flush=True)


if __name__ == '__main__':
    main()
