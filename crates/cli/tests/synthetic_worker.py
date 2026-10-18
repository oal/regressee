# SPDX-License-Identifier: MPL-2.0
# OWNED SYNTHETIC PROTOCOL FIXTURE. No Chromium, sandbox or public-site claim.
# DOM bytes come from the core cross-language goldens, never a PNG placeholder.
import hashlib
import json
import os
from pathlib import Path
import struct
import sys
import time
import uuid
import zlib

STATE = __STATE__
VERSION = 3
WORKER_VERSION = '6.0.0'
MAX_FRAME = 1024 * 1024
MODES = ('ScreenshotsOnly', 'ScreenshotsAndDomSnapshots', 'DomSnapshotsOnly')
RETRYABLE = ('Timeout', 'WorkerStopped', 'RateLimited', 'ServerError')


def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))


def make_png(changed=False):
    width, height = 64, 48
    color = b'\xd0\x20\x40' if changed else b'\x10\x80\x70'
    pixels = b''.join(b'\0' + color * width for _ in range(height))
    return (b'\x89PNG\r\n\x1a\n'
            + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(pixels)) + chunk(b'IEND', b''))


def emit(request, kind, payload=None):
    event = {'type': kind}
    if payload is not None:
        event['payload'] = payload
    print(json.dumps({'version': VERSION, 'job': request['job'],
                      'request_id': request['request_id'], 'event': event},
                     separators=(',', ':')), flush=True)


def private_file(path, body):
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'wb') as output:
        output.write(body)
        output.flush()
        os.fsync(output.fileno())


def publish(request, kind, body, metadata):
    token = str(uuid.uuid4())
    root = Path(os.environ['REGRESSEE_OUTPUT'])
    suffix = '.png' if kind == 'Screenshot' else '.dom.jsonl'
    sidecar = '.json' if kind == 'Screenshot' else '.dom.json'
    artifact = {'id': hashlib.sha256(body).hexdigest(), 'bytes': len(body), 'kind': kind}
    metadata.update(sha256=artifact['id'], bytes=artifact['bytes'])
    private_file(root / (token + suffix), body)
    private_file(root / (token + sidecar), json.dumps(metadata, separators=(',', ':')).encode())
    capture = {'kind': kind, 'output_token': token, 'artifact': artifact}
    if kind == 'Screenshot':
        capture.update(width=metadata['width'], height=metadata['height'])
    else:
        capture['policy'] = metadata['policy']
    emit(request, 'ChannelCaptured', {'key': metadata['key'], 'capture': capture})


def common_metadata(payload):
    return {'version': 1, 'key': payload['key'], 'requested': payload['address'],
            'final_url': payload['address'],
            'runtime': json.loads(os.environ['REGRESSEE_RUNTIME']),
            'profile': payload['profile'], 'warnings': [], 'stable': True,
            'duration_ms': 1,
            'readiness': {'font_failures': 0, 'image_failures': 0, 'resource_failures': 0}}


def dom_bytes(state):
    name = 'dom-v1-html-names.ndjson' if state == 'dom-prefix' else 'dom-v1.ndjson'
    body = Path(__file__).with_name(name).read_bytes()
    assert body.endswith(b'\n')
    records = [json.loads(line) for line in body.decode('utf-8').split('\n')[:-1]]
    assert records[0] == ['regressee-dom', 1]
    if state in ('changed', 'dom-changed', 'screenshot-failed', 'screenshot-timeout'):
        text = next(record for record in reversed(records) if record[0] == 'text')
        text[2] += ' [changed]'
        body = ''.join(json.dumps(record, ensure_ascii=False, separators=(',', ':')) + '\n'
                       for record in records).encode('utf-8')
    nodes = records[1:]
    coverage = {'iframe_hosts': sum(record[0] == 'element' and record[3] == 'iframe'
                                    for record in nodes),
                'open_shadow_roots': sum(record[0] == 'shadow' for record in nodes),
                'template_contents': sum(record[0] == 'template' for record in nodes)}
    return body, len(nodes), max(record[1] for record in nodes), coverage


def failed(request, payload, kind, code):
    emit(request, 'ChannelFailed', {'key': payload['key'], 'kind': kind,
                                   'code': code, 'retryable': code in RETRYABLE})


def capture(request, payload):
    state = Path(STATE).read_text().strip()
    if state == 'slow':
        time.sleep(1.2)
    plan = payload['plan']
    kinds = ([] if plan['mode'] == 'ScreenshotsOnly' else ['DomSnapshot'])
    if plan['mode'] != 'DomSnapshotsOnly':
        kinds.append('Screenshot')
    for kind in kinds:
        if state == 'failed':
            failed(request, payload, kind, 'NotFound')
            continue
        if kind == 'Screenshot':
            if state in ('screenshot-failed', 'screenshot-timeout'):
                failed(request, payload, kind,
                       'ImageLimit' if state == 'screenshot-failed' else 'Timeout')
                continue
            metadata = common_metadata(payload)
            metadata.update(width=64, height=48, capture_mode='chromium-new-headless')
            publish(request, kind, make_png(state in ('changed', 'image-changed')), metadata)
        else:
            if state in ('dom-failed', 'dom-limit'):
                failed(request, payload, kind,
                       'DomSerialization' if state == 'dom-failed' else 'DomLimit')
                continue
            body, count, depth, coverage = dom_bytes(state)
            metadata = common_metadata(payload)
            warnings = ['ClosedShadowRootsExcluded']
            if coverage['iframe_hosts']:
                warnings.insert(0, 'IframeDocumentsExcluded')
            if plan['mode'] == 'ScreenshotsAndDomSnapshots':
                warnings.append('NearbySamplesNotAtomic')
            if state == 'dom-unstable':
                warnings.append('Unstable')
                metadata['stable'] = False
            for warning in warnings:
                emit(request, 'ChannelWarning', {'key': payload['key'], 'kind': kind,
                                                'code': warning})
            metadata.update(run=request['job'], policy=plan['dom_policy'],
                            node_count=count, visited_nodes=count, max_depth=depth,
                            coverage=coverage, warnings=warnings)
            publish(request, kind, body, metadata)
    emit(request, 'CaptureCompleted', {'key': payload['key']})


def validate(request):
    if (not isinstance(request, dict) or set(request) != {'version', 'job', 'request_id', 'command'}
            or request['version'] != VERSION or type(request['request_id']) is not int
            or not 1 <= request['request_id'] <= 9007199254740991
            or str(uuid.UUID(request['job'])) != request['job'] or uuid.UUID(request['job']).int == 0):
        raise ValueError('invalid fixture envelope')
    command = request['command']
    if not isinstance(command, dict) or 'type' not in command:
        raise ValueError('invalid fixture command')
    kind = command['type']
    if kind in ('Handshake', 'InstallSmoke', 'Cancel', 'Shutdown'):
        if set(command) != {'type'}:
            raise ValueError('unexpected fixture arguments')
    else:
        if set(command) != {'type', 'payload'} or not isinstance(command['payload'], dict):
            raise ValueError('invalid fixture arguments')
        payload = command['payload']
        if kind == 'CaptureTarget':
            plan = payload.get('plan', {})
            if (set(payload) != {'key', 'address', 'profile', 'session', 'plan'}
                    or set(plan) != {'version', 'mode', 'dom_policy'}
                    or type(plan['version']) is not int or plan['version'] != 1
                    or plan['mode'] not in MODES
                    or (plan['dom_policy'] is not None if plan['mode'] == 'ScreenshotsOnly'
                        else plan['dom_policy'] != {'version': 1})
                    or payload['key']['profile'] != payload['profile']['id']
                    or payload['profile']['runtime'] != json.loads(os.environ['REGRESSEE_RUNTIME'])):
                raise ValueError('invalid fixture capture plan')
        elif kind == 'FetchRobots':
            if set(payload) != {'address'}:
                raise ValueError('invalid fixture robots request')
        elif kind == 'DiscoverPage':
            if set(payload) != {'address', 'section'}:
                raise ValueError('invalid fixture discovery request')
        else:
            raise ValueError('unsupported fixture command')


def handle(request):
    validate(request)
    command = request['command']
    kind = command['type']
    payload = command.get('payload', {})
    emit(request, 'Acknowledged')
    if kind == 'Handshake':
        # Deliberately does not call make_png: handshake is pixel-independent.
        emit(request, 'Ready', {'worker_version': WORKER_VERSION, 'browser_version': 'synthetic'})
    elif kind == 'InstallSmoke':
        assert make_png().startswith(b'\x89PNG\r\n\x1a\n')
        emit(request, 'Completed')
    elif kind == 'FetchRobots':
        emit(request, 'RobotsFound', {'text': 'User-agent: *\nAllow: /', 'status': 200})
    elif kind == 'DiscoverPage':
        base = payload['address'].split('/site')[0]
        emit(request, 'LinksFound', {'addresses': [base + '/site/second'],
                                     'truncated': False, 'final_url': payload['address']})
    elif kind == 'CaptureTarget':
        capture(request, payload)
    elif kind == 'Cancel':
        emit(request, 'Cancelled')
    elif kind == 'Shutdown':
        emit(request, 'Completed')
        return False
    return True


def main():
    while True:
        line = sys.stdin.buffer.readline(MAX_FRAME + 1)
        if not line:
            return 0
        if len(line) > MAX_FRAME or not line.endswith(b'\n'):
            return 65
        try:
            if not handle(json.loads(line)):
                return 0
        except (ValueError, KeyError, TypeError, AssertionError):
            # Never echo source/state/request contents into logs.
            return 65


if __name__ == '__main__':
    sys.exit(main())
