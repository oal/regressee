# SPDX-License-Identifier: MPL-2.0
# OWNED SYNTHETIC PROTOCOL FIXTURE. No Chromium, sandbox or public-site claim.
import hashlib
import json
import os
import struct
import sys
import time
import uuid
import zlib

STATE = __STATE__


def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))


def emit(request, kind, payload=None):
    event = {'type': kind}
    if payload is not None:
        event['payload'] = payload
    print(json.dumps({'version': 2, 'job': request['job'], 'request_id': request['request_id'], 'event': event}), flush=True)


for line in sys.stdin:
    request = json.loads(line)
    command = request['command']
    kind = command['type']
    p = command.get('payload', {})
    emit(request, 'Acknowledged')
    if kind == 'Handshake':
        emit(request, 'Ready', {'worker_version': '5.0.1', 'browser_version': 'synthetic'})
    elif kind == 'FetchRobots':
        emit(request, 'RobotsFound', {'text': 'User-agent: *\nAllow: /', 'status': 200})
    elif kind == 'DiscoverPage':
        base = p['address'].split('/site')[0]
        emit(request, 'LinksFound', {'addresses': [base + '/site/second'], 'truncated': False, 'final_url': p['address']})
    elif kind == 'CaptureTarget':
        with open(STATE) as f:
            state = f.read().strip()
        if state == 'slow':
            time.sleep(1.2)
        if state == 'failed':
            emit(request, 'Failure', {'code': 'NotFound', 'retryable': False})
            continue
        w, h = 64, 48
        color = b'\x10\x80\x70' if state in ('base', 'slow') else b'\xd0\x20\x40'
        pixels = b''.join(b'\0' + color * w for _ in range(h))
        png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(pixels)) + chunk(b'IEND', b'')
        token = str(uuid.uuid4())
        sha = hashlib.sha256(png).hexdigest()
        root = os.environ['REGRESSEE_OUTPUT']
        with open(os.path.join(root, token + '.png'), 'wb') as f:
            f.write(png)
        meta = {'version': 1, 'key': p['key'], 'requested': p['address'], 'final_url': p['address'], 'runtime': json.loads(os.environ['REGRESSEE_RUNTIME']), 'profile': p['profile'], 'width': w, 'height': h, 'sha256': sha, 'bytes': len(png), 'warnings': [], 'stable': True, 'duration_ms': 1, 'readiness': {'font_failures': 0, 'image_failures': 0, 'resource_failures': 0}, 'capture_mode': 'chromium-new-headless'}
        with open(os.path.join(root, token + '.json'), 'w') as f:
            json.dump(meta, f)
        emit(request, 'TargetCaptured', {'key': p['key'], 'output_token': token, 'artifact': {'id': sha, 'bytes': len(png), 'kind': 'Screenshot'}, 'width': w, 'height': h})
    elif kind == 'Shutdown':
        emit(request, 'Completed')
        break
