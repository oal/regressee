# SPDX-License-Identifier: MPL-2.0
"""Source-only tests of the owned protocol fixture; not Chromium qualification."""
import contextlib
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import uuid


class SyntheticWorkerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='regressee-cli-worker-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.state = self.root / 'state'
        self.state.write_text('base')
        self.worker = self.root / 'worker'
        self.worker.mkdir(mode=0o700)
        source = Path(__file__).with_name('synthetic_worker.py').read_text()
        self.script = self.worker / 'main.py'
        self.script.write_text(source.replace('__STATE__', json.dumps(str(self.state))))
        self.goldens = Path(__file__).resolve().parents[2] / 'core/tests/fixtures'
        for name in ('dom-v1.ndjson', 'dom-v1-html-names.ndjson'):
            shutil.copyfile(self.goldens / name, self.worker / name)
        self.runtime = {'Pinned': {'node': 'synthetic-python', 'worker': '6.0.0',
                                   'playwright': 'synthetic', 'browser': 'synthetic'}}
        self.profile = json.loads((self.goldens / 'legacy-capture-profile.json').read_text())
        self.profile['runtime'] = self.runtime
        self.profile['user_agent'] = 'Owned synthetic protocol fixture'
        self.namespace = {'__name__': 'synthetic_fixture', '__file__': str(self.script)}
        exec(compile(self.script.read_text(), str(self.script), 'exec'), self.namespace)

    def request(self, command='CaptureTarget', mode='ScreenshotsAndDomSnapshots'):
        request = {'version': 3, 'job': str(uuid.uuid4()), 'request_id': 1,
                   'command': {'type': command}}
        if command == 'CaptureTarget':
            request['command']['payload'] = {
                'key': {'page': str(uuid.uuid4()), 'profile': self.profile['id']},
                'address': 'http://127.0.0.1:43210/site', 'profile': copy.deepcopy(self.profile),
                'session': None,
                'plan': {'version': 1, 'mode': mode,
                         'dom_policy': None if mode == 'ScreenshotsOnly' else {'version': 1}}}
        return request

    def invoke(self, request, state='base'):
        self.state.write_text(state)
        output = self.root / str(uuid.uuid4())
        output.mkdir(mode=0o700)
        text = io.StringIO()
        environment = {'REGRESSEE_OUTPUT': str(output), 'REGRESSEE_SCOPE': 'loopback',
                       'REGRESSEE_RUNTIME': json.dumps(self.runtime)}
        with mock.patch.dict(os.environ, environment), contextlib.redirect_stdout(text):
            self.namespace['handle'](request)
        frames = text.getvalue().splitlines()
        events = [json.loads(frame) for frame in frames]
        for frame, event in zip(frames, events):
            self.assertLess(len(frame.encode()) + 1, 1024 * 1024)
            self.assertEqual(set(event), {'version', 'job', 'request_id', 'event'})
            self.assertEqual(event['version'], 3)
            self.assertEqual(event['job'], request['job'])
            self.assertEqual(event['request_id'], request['request_id'])
        self.assertEqual(events[0]['event']['type'], 'Acknowledged')
        self.assertNotIn('Private & ordinary', text.getvalue())
        self.assertNotIn('regressee-dom', text.getvalue())
        return output, [event['event'] for event in events]

    def originals(self, output, events, request):
        found = {}
        for event in events:
            if event['type'] != 'ChannelCaptured':
                continue
            self.assertEqual(set(event['payload']), {'key', 'capture'})
            capture = event['payload']['capture']
            kind = capture['kind']
            suffix, sidecar = ('.png', '.json') if kind == 'Screenshot' else ('.dom.jsonl', '.dom.json')
            body_path = output / (capture['output_token'] + suffix)
            meta_path = output / (capture['output_token'] + sidecar)
            body = body_path.read_bytes()
            metadata = json.loads(meta_path.read_text())
            self.assertEqual(capture['artifact'], {'id': hashlib.sha256(body).hexdigest(),
                                                  'bytes': len(body), 'kind': kind})
            for field in ('key', 'profile', 'requested', 'final_url', 'runtime'):
                expected = {'key': request['command']['payload']['key'],
                            'profile': request['command']['payload']['profile'],
                            'requested': request['command']['payload']['address'],
                            'final_url': request['command']['payload']['address'],
                            'runtime': self.runtime}[field]
                self.assertEqual(metadata[field], expected)
            self.assertEqual(metadata['sha256'], capture['artifact']['id'])
            self.assertEqual(metadata['bytes'], len(body))
            if kind == 'DomSnapshot':
                self.assertEqual(set(capture), {'kind', 'output_token', 'artifact', 'policy'})
                self.assertEqual(metadata['run'], request['job'])
                self.assertEqual(metadata['policy'], request['command']['payload']['plan']['dom_policy'])
                self.assertTrue(body.startswith(b'["regressee-dom",1]\n'))
                for fake_pixel_field in ('width', 'height', 'image', 'thumbnail', 'tiles'):
                    self.assertNotIn(fake_pixel_field, metadata)
                warnings = [event['payload']['code'] for event in events
                            if event['type'] == 'ChannelWarning'
                            and event['payload']['kind'] == kind]
                self.assertEqual(metadata['warnings'], warnings)
            else:
                self.assertEqual(set(capture), {'kind', 'output_token', 'artifact', 'width', 'height'})
                self.assertTrue(body.startswith(b'\x89PNG\r\n\x1a\n'))
                self.assertEqual((capture['width'], capture['height']), (64, 48))
            if os.name == 'posix':
                self.assertEqual(body_path.stat().st_mode & 0o7777, 0o600)
                self.assertEqual(meta_path.stat().st_mode & 0o7777, 0o600)
            found[kind] = (body, metadata)
        return found

    def test_handshake_has_no_pixel_path_and_install_smoke_is_explicit(self):
        original = self.namespace['make_png']
        self.namespace['make_png'] = mock.Mock(side_effect=AssertionError('pixel path forbidden'))
        output, events = self.invoke(self.request('Handshake'))
        self.namespace['make_png'].assert_not_called()
        self.assertEqual(events[-1], {'type': 'Ready', 'payload': {
            'worker_version': '6.0.0', 'browser_version': 'synthetic'}})
        self.assertEqual(list(output.iterdir()), [])
        self.namespace['make_png'] = mock.Mock(wraps=original)
        output, events = self.invoke(self.request('InstallSmoke'))
        self.namespace['make_png'].assert_called_once()
        self.assertEqual(events[-1], {'type': 'Completed'})
        self.assertEqual(list(output.iterdir()), [])

    def test_all_modes_publish_only_requested_real_channels(self):
        for mode, expected in [('ScreenshotsOnly', {'Screenshot'}),
                               ('ScreenshotsAndDomSnapshots', {'Screenshot', 'DomSnapshot'}),
                               ('DomSnapshotsOnly', {'DomSnapshot'})]:
            with self.subTest(mode=mode):
                request = self.request(mode=mode)
                original = self.namespace['make_png']
                if mode == 'DomSnapshotsOnly':
                    self.namespace['make_png'] = mock.Mock(side_effect=AssertionError('DOM-only called PNG'))
                output, events = self.invoke(request)
                found = self.originals(output, events, request)
                self.assertEqual(set(found), expected)
                self.assertEqual(events[-1], {'type': 'CaptureCompleted', 'payload': {
                    'key': request['command']['payload']['key']}})
                self.assertEqual(len(list(output.iterdir())), len(expected) * 2)
                if 'DomSnapshot' in found:
                    body, metadata = found['DomSnapshot']
                    self.assertEqual(body, (self.goldens / 'dom-v1.ndjson').read_bytes())
                    self.assertEqual(metadata['coverage'], {'iframe_hosts': 1, 'open_shadow_roots': 1,
                                                             'template_contents': 2})
                    self.assertEqual(metadata['max_depth'], 10)
                    self.assertEqual(metadata['node_count'], 28)
                if mode == 'DomSnapshotsOnly':
                    self.namespace['make_png'].assert_not_called()
                    self.assertFalse(list(output.glob('*.png')))
                self.namespace['make_png'] = original

    def test_independent_changes_and_failures_are_not_sibling_failures(self):
        request = self.request()
        output, events = self.invoke(request)
        base = self.originals(output, events, request)
        for state, changed_kind in [('dom-changed', 'DomSnapshot'), ('image-changed', 'Screenshot')]:
            output, events = self.invoke(request, state)
            found = self.originals(output, events, request)
            for kind in base:
                self.assertEqual(found[kind][0] != base[kind][0], kind == changed_kind)
        for state, failed_kind, code in [('dom-failed', 'DomSnapshot', 'DomSerialization'),
                                         ('dom-limit', 'DomSnapshot', 'DomLimit'),
                                         ('screenshot-failed', 'Screenshot', 'ImageLimit')]:
            output, events = self.invoke(request, state)
            found = self.originals(output, events, request)
            self.assertEqual(set(found), {'Screenshot', 'DomSnapshot'} - {failed_kind})
            failures = [event['payload'] for event in events if event['type'] == 'ChannelFailed']
            self.assertEqual(failures, [{'key': request['command']['payload']['key'],
                                          'kind': failed_kind, 'code': code, 'retryable': False}])
            self.assertEqual(events[-1]['type'], 'CaptureCompleted')
        output, events = self.invoke(request, 'failed')
        self.assertEqual(self.originals(output, events, request), {})
        self.assertEqual([event['payload']['kind'] for event in events if event['type'] == 'ChannelFailed'],
                         ['DomSnapshot', 'Screenshot'])
        self.assertEqual(list(output.iterdir()), [])

    def test_unchanged_but_unstable_dom_and_optional_real_prefix_goldens(self):
        request = self.request(mode='DomSnapshotsOnly')
        output, events = self.invoke(request, 'dom-unstable')
        body, metadata = self.originals(output, events, request)['DomSnapshot']
        self.assertEqual(body, (self.goldens / 'dom-v1.ndjson').read_bytes())
        self.assertFalse(metadata['stable'])
        self.assertIn('Unstable', metadata['warnings'])
        output, events = self.invoke(request, 'dom-prefix')
        body, metadata = self.originals(output, events, request)['DomSnapshot']
        self.assertEqual(body, (self.goldens / 'dom-v1-html-names.ndjson').read_bytes())
        records = [json.loads(line) for line in body.decode().split('\n')[:-1]]
        self.assertTrue(any(record[0] == 'element' and len(record) == 6 for record in records))
        self.assertEqual(metadata['node_count'], len(records) - 1)

    def test_strict_v3_frames_and_private_no_clobber_output(self):
        for mutation in ('old-version', 'unknown-plan-field', 'missing-plan'):
            request = self.request()
            if mutation == 'old-version':
                request['version'] = 2
            elif mutation == 'unknown-plan-field':
                request['command']['payload']['plan']['script'] = 'private-canary'
            else:
                del request['command']['payload']['plan']
            with self.assertRaises((ValueError, KeyError)):
                self.namespace['validate'](request)
        request = self.request('Handshake')
        request['version'] = 2
        result = subprocess.run([sys.executable, str(self.script)],
                                input=(json.dumps(request) + '\n').encode(), capture_output=True,
                                timeout=5)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (65, b'', b''))
        output = self.root / 'private-original'
        self.namespace['private_file'](output, b'original')
        with self.assertRaises(FileExistsError):
            self.namespace['private_file'](output, b'replacement')
        self.assertEqual(output.read_bytes(), b'original')


if __name__ == '__main__':
    unittest.main()
