#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Verify the 2026-10 UX plan's final native batch; emit a scrubbed receipt.

Read-only: original PNGs, metadata, state matrix and current sources are required.
This checks recorded assertions/integrity, not visual quality or native OS input.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import zlib


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inside(bounds, clip):
    return (bounds['width'] > 0 and bounds['height'] > 0
            and bounds['x'] >= clip['x'] - 0.1
            and bounds['y'] >= clip['y'] - 0.1
            and bounds['x'] + bounds['width'] <= clip['x'] + clip['width'] + 0.1
            and bounds['y'] + bounds['height'] <= clip['y'] + clip['height'] + 0.1)


def verify(root, repo):
    assert root.is_absolute() and root.stat().st_mode & 0o777 == 0o700
    matrix = json.loads((root / 'phase5-matrix.json').read_text())
    assert not matrix['preflight_only']
    assert matrix['baseline_unchanged'] and matrix['no_import_or_deletion']
    assert matrix['originals_unchanged'] and matrix['synthetic_archive_constructed_locally']
    proof = json.loads((root / 'proof-reviewed.json').read_text())
    assert proof['personally_reviewed'] and proof['png_sha256'] == sha(root / 'renderer-proof.png')
    fixtures = json.loads((root / 'website-fixture-provenance.json').read_text())
    assert len(fixtures) == 4
    for fixture in fixtures:
        assert fixture['synthetic_offline_website_only']
        assert fixture['sha256'] == sha(root / 'website-fixtures' / fixture['name'])
    harness = repo / 'crates/gui/src/ui_renderer_capture.rs'
    bindings = {
        key: (harness.parent / path).resolve()
        for key, path in re.findall(r'"(\w+_source_sha256)"[^"\n]*include_bytes!\("([^"]+)"\)', harness.read_text())
    }
    assert len(bindings) >= 34
    sources = {str(path.relative_to(repo)): sha(path) for path in bindings.values()}
    scenes = set('''wizard-website access-public access-optional-mapping private-login-loading
        mobile-revealed-selected tablet-selected custom-width custom-height custom-added-actions
        image-limitations-tail capture-limitations-tail visible-screen-selected replacement-selected
        fast-selected invalid-advanced-revealed discovery-excluded discovery-included
        discovery-matches-excluded discovery-matches-restored review-compare results-hidden
        navigation-toggled full-before full-after difference overlay heatmap-with-ignored view-menu
        zoom-menu multiline-note dirty-note-blocker analysis-sensitivity analysis-ignored-coordinates
        dirty-area-blocker analysis-advanced-tail filter-changed filter-gaps filter-no-significant-change
        filter-grouped baseline-selection history-last-run history-selected-actions history-page-two
        history-provenance delete-consequences delete-exact export-consequences export-exact
        import-consequences import-exact network-limits-closed network-limits-open'''.split())
    frames = matrix['frames']
    assert len(frames) == len(scenes) * 2
    assert {p.stem for p in root.glob('phase5-*.png')} == {f['label'] for f in frames}
    receipt_frames = []
    for size in [(1280, 800), (820, 600)]:
        selected = {f['scene']: f for f in frames if tuple(f['size']) == size}
        assert set(selected) == scenes
        state = lambda scene: selected[scene]['state']
        assert state('private-login-loading')['private_access'] == 'InteractiveLogin'
        assert state('private-login-loading')['login_status_loading']
        assert state('private-login-loading')['ordinary_pending'] is not None
        assert not state('private-login-loading')['access_can_continue']
        assert 'Guided(31)' not in state('private-login-loading')['mounted_enabled']
        assert any(p['name'] == 'Mobile' for p in state('mobile-revealed-selected')['profiles'])
        assert any(p['name'] == 'Tablet' for p in state('tablet-selected')['profiles'])
        assert any(p['name'] == 'Custom' and p['width'] == 1024 and p['height'] == 768
                   for p in state('custom-added-actions')['profiles'])
        invalid = state('invalid-advanced-revealed')
        assert invalid['advanced'] and invalid['parallel_field'] == '9' and invalid['parallel_error']
        assert invalid['focused'] == 'CrawlField(4)' and invalid['ordinary_pending'] is None
        assert not invalid['task_present']
        assert state('fast-selected')['speed'] == 'Fast'
        assert all(p['extent'] == 'VisibleScreen' and p['images'] != 'Normal'
                   for p in state('fast-selected')['profiles'])
        for scene, included in [('discovery-excluded', 74), ('discovery-included', 75),
                                ('discovery-matches-excluded', 74), ('discovery-matches-restored', 75)]:
            assert state(scene)['included'] == included and state(scene)['discovered'] == 128
        assert state('discovery-excluded')['selected_page_included'] is False
        assert state('discovery-included')['selected_page_included'] is True
        for scene, index, classification in [('filter-changed', 1, 'Changed — review the evidence'),
                                             ('filter-gaps', 2, 'Could not capture — not verified'),
                                             ('filter-no-significant-change', 3, 'No significant change')]:
            s = state(scene)
            assert s['filter'] == index and s['shown_results']
            assert set(s['shown_classifications']) == {classification}
            assert s['selected_key'] == state('filter-grouped')['selected_key']
        assert state('filter-grouped')['group_profiles']
        assert state('multiline-note')['note_open'] and state('multiline-note')['note_dirty']
        assert state('dirty-note-blocker')['note_dirty'] and state('dirty-note-blocker')['blocker']
        assert state('dirty-area-blocker')['mask_dirty'] and state('dirty-area-blocker')['blocker']
        assert state('analysis-sensitivity')['analysis_open']
        assert state('analysis-ignored-coordinates')['ignored_open']
        assert state('analysis-advanced-tail')['analysis_advanced']
        assert state('heatmap-with-ignored')['pair']['mask']
        assert state('baseline-selection')['selection_count'] == 1
        assert state('history-last-run')['loaded_runs'] == 64
        assert state('history-last-run')['selected_run_index'] == 63
        assert state('history-page-two')['history_page'] == 1
        assert not state('history-page-two')['history_more']
        assert state('history-provenance')['provenance_open']
        for action in ('export', 'import', 'delete'):
            assert state(action + '-exact')['portability_exact']
            preview = action + '_preview'
            assert state(action + '-exact')[preview]
        assert not state('network-limits-closed')['network_limits']
        assert state('network-limits-open')['network_limits']
        def pane_height(scene):
            return next(g['bounds']['height'] for g in selected[scene]['geometry']
                        if g['selector'] == 'phase5-before')
        assert pane_height('results-hidden') >= pane_height('review-compare')
        assert pane_height('full-before') >= pane_height('review-compare')
        assert state('full-before')['navigation_hidden'] == state('review-compare')['navigation_hidden']
        for frame in selected.values():
            label = frame['label']
            png, sidecar = root / (label + '.png'), root / (label + '.json')
            data = png.read_bytes()
            assert data[:8] == b'\x89PNG\r\n\x1a\n' and data[12:16] == b'IHDR'
            assert struct.unpack('>II', data[16:24]) == size
            cursor = 8
            while cursor < len(data):
                length = struct.unpack('>I', data[cursor:cursor + 4])[0]
                chunk = data[cursor + 4:cursor + 8 + length]
                crc = struct.unpack('>I', data[cursor + 8 + length:cursor + 12 + length])[0]
                assert zlib.crc32(chunk) == crc, f'{label}: PNG chunk integrity'
                cursor += length + 12
            assert cursor == len(data)
            metadata = json.loads(sidecar.read_text())
            assert metadata['png_sha256'] == sha(png)
            assert metadata['opaque'] and metadata['distinct_rgb_colors'] > 16
            assert metadata['real_gpui_blade_offscreen'] and not metadata['os_input_verified']
            assert not frame['website_visited'] and frame['geometry']
            for key, path in bindings.items():
                assert metadata[key] == sha(path), f'{label}: stale {key}'
            for geometry in frame['geometry']:
                assert geometry['fully_visible'] and inside(geometry['bounds'], geometry['clip'])
                assert inside(geometry['bounds'], dict(x=0, y=0, width=size[0], height=size[1]))
            receipt_frames.append({'label': label, 'png_sha256': sha(png),
                                   'sidecar_sha256': sha(sidecar),
                                   'state_geometry_sha256': hashlib.sha256(json.dumps(frame, sort_keys=True).encode()).hexdigest(),
                                   'state': {k: frame['state'][k] for k in (
                                       'screen', 'focused', 'last_action', 'advanced', 'private_access',
                                       'login_status_loading', 'access_can_continue', 'profiles', 'speed',
                                       'filter', 'group_profiles', 'included', 'matching_pages',
                                       'shown_classifications', 'view', 'results_hidden', 'navigation_hidden',
                                       'note_bytes', 'note_dirty', 'mask_dirty', 'blocker', 'selection_count',
                                       'analysis_open', 'ignored_open', 'analysis_advanced',
                                       'loaded_runs', 'history_page', 'selected_run_index',
                                       'portability_exact', 'network_limits')},
                                   'geometry': frame['geometry']})
    return {'scope': 'UX plan 2026-10-13 Phase 5 implementer integrity/state check; not independent verdict',
            'root': str(root), 'frames': receipt_frames, 'sizes': [[1280,800],[820,600]],
            'embedded_source_bindings_per_png': len(bindings), 'current_sources': sources,
            'matrix_sha256': sha(root / 'phase5-matrix.json'),
            'proof_sha256': sha(root / 'renderer-proof.png'),
            'proof_receipt_sha256': sha(root / 'proof-reviewed.json'),
            'fixture_provenance_sha256': sha(root / 'website-fixture-provenance.json'),
            'assertions_passed': True, 'baseline_originals_retained': True,
            'os_input_at_platform_release_qualified': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--repo', type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    print(json.dumps(verify(args.root.resolve(), args.repo.resolve()), indent=2))
