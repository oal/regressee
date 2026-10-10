#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""One owned-window X11/lavapipe inspection, never desktop qualification.

Fresh output only. No test mapping, virtual display, unrelated window capture,
global input outside the identified owned window, or retries after blank pixels.
"""
import argparse
import ctypes as C
import importlib.util
import json
import os
import pathlib
import sqlite3
import subprocess
import time
import zlib
import struct

ROOT = pathlib.Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location('bench_native_helpers', ROOT / 'scripts/ui-smoke.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    x = helper.x
    helper.bind('XCloseDisplay', C.c_int, [helper.Display])
    helper.bind('XGetInputFocus', C.c_int, [helper.Display, C.POINTER(helper.Window), C.POINTER(C.c_int)])
    def owned_windows(display, parent, pid):
        root = helper.Window(); back = helper.Window(); children = C.POINTER(helper.Window)(); count = C.c_uint()
        if not x.XQueryTree(display, parent, C.byref(root), C.byref(back), C.byref(children), C.byref(count)): return
        values = [children[i] for i in range(count.value)]
        if children: x.XFree(children)
        for window in values:
            name = C.c_char_p()
            if x.XFetchName(display, window, C.byref(name)) and name.value:
                title = name.value.decode(errors='replace'); x.XFree(name)
                if title == 'Regressee — Foundation' and helper.window_pid(display, window) == pid: yield window
            yield from owned_windows(display, window, pid)
    def pixels(path):
        data = path.read_bytes(); position = 8; compressed = bytearray(); width = height = 0
        while position < len(data):
            size = struct.unpack('>I', data[position:position+4])[0]
            kind = data[position+4:position+8]; payload = data[position+8:position+8+size]
            assert zlib.crc32(kind+payload) & 0xffffffff == struct.unpack('>I', data[position+8+size:position+12+size])[0]
            if kind == b'IHDR': width, height = struct.unpack('>II', payload[:8])
            if kind == b'IDAT': compressed.extend(payload)
            position += 12 + size
        raw = zlib.decompress(compressed); colors = set()
        for row in range(height):
            start = row * (1 + width * 3); assert raw[start] == 0
            for at in range(start+1, start+1+width*3, 3): colors.add(bytes(raw[at:at+3]))
        return {'width':width, 'height':height, 'colors':len(colors), 'meaningful':len(colors)>8}
    result = {'outcome':'FAIL / native evidence open', 'backend':'X11 / Vulkan lavapipe developer probe', 'forced_mapping':False,
              'not_run':['Windows', 'macOS', 'default Wayland / hardware graphics', 'screen reader', 'native OS dialogs/menu'],
              'native_input':'NOT RUN unless meaningful rendering and owned focus are established'}
    env = dict(os.environ, REGRESSEE_DEV_DATA=str(output/'native-data'), VK_ICD_FILENAMES='/usr/share/vulkan/icd.d/lvp_icd.json')
    env.pop('WAYLAND_DISPLAY', None)
    display = None; process = None
    try:
        with (output/'app.log').open('w') as log:
            process = subprocess.Popen([str(ROOT/'target/debug/regressee')], cwd=ROOT, env=env, stdout=log, stderr=log)
            result['pid'] = process.pid
            display = x.XOpenDisplay(None); assert display, 'X11 display unavailable'
            window = None
            for _ in range(100):
                assert process.poll() is None, 'native application exited before window discovery'
                window = next(owned_windows(display, x.XDefaultRootWindow(display), process.pid), None)
                if window: break
                time.sleep(.1)
            assert window, 'owned product window not found'
            result['window'] = int(window); attr = helper.Attr()
            for _ in range(50):
                x.XGetWindowAttributes(display, window, C.byref(attr))
                if attr.map_state == 2: break
                time.sleep(.1)
            result['map_state'] = attr.map_state
            assert attr.map_state == 2, 'owned window not viewable; no forced mapping attempted'
            time.sleep(.5)
            helper.screenshot(display, window, output/'projects-linux-x11.png')
            result['initial_pixels'] = pixels(output/'projects-linux-x11.png')
            assert result['initial_pixels']['meaningful'], 'owned screenshot is black/blank, not visual evidence'
            x.XSetInputFocus(display, window, 1, 0); x.XFlush(display)
            focus = helper.Window(); revert = C.c_int(); x.XGetInputFocus(display, C.byref(focus), C.byref(revert))
            result['focus_readback'] = int(focus.value); assert focus.value == window, 'owned native focus not established'
            helper.press(display, window, 'n', 4)
            for key in ['p','h','a','s','e','space','o','n','e']: helper.press(display, window, key)
            helper.press(display, window, 'Tab')
            for key in ['h','t','t','p','s']: helper.press(display, window, key)
            helper.press(display, window, 'semicolon', 1)
            for key in ['slash','slash','e','x','a','m','p','l','e','period','t','e','s','t']: helper.press(display, window, key)
            helper.press(display, window, 's', 4)
            database = output/'native-data/data/regressee.sqlite3'
            saved = False
            for _ in range(50):
                if database.exists():
                    with sqlite3.connect(database) as connection:
                        rows = connection.execute('SELECT name,config FROM projects').fetchall()
                        saved = len(rows) == 1 and rows[0][0] == 'phase one' and json.loads(rows[0][1])['sources'] == ['https://example.test/']
                if saved: break
                time.sleep(.1)
            assert saved, 'native create/save not proven by stored project'
            helper.screenshot(display, window, output/'setup-linux-x11.png')
            assert pixels(output/'setup-linux-x11.png')['meaningful'], 'setup screenshot lacks meaningful pixels'
            helper.press(display, window, '1', 4); time.sleep(.5)
            for _ in range(6): helper.press(display, window, 'Tab')
            helper.press(display, window, 'Return'); time.sleep(.5)
            helper.screenshot(display, window, output/'reopened-setup-linux-x11.png')
            assert pixels(output/'reopened-setup-linux-x11.png')['meaningful'], 'reopened setup screenshot lacks meaningful pixels'
            for key, name in [('3','results'), ('4','history'), ('5','settings')]:
                helper.press(display, window, key, 4); time.sleep(.3)
                helper.screenshot(display, window, output/f'{name}-linux-x11.png')
                assert pixels(output/f'{name}-linux-x11.png')['meaningful'], f'{name} screenshot lacks meaningful pixels'
            helper.press(display, window, 'q', 4); process.wait(timeout=5)
            assert process.returncode == 0, 'native keyboard quit failed'
            result['native_input'] = 'scoped create/save/navigation/quit exercised; visual content needs independent review'
            result['outcome'] = 'Scoped probe completed; NOT ordinary desktop qualification'
    except Exception as error:
        result['reason'] = str(error)
    finally:
        if process and process.poll() is None:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired: process.kill(); process.wait()
        if display: x.XCloseDisplay(display)
        result['own_process_reaped'] = process is not None and process.poll() is not None
        (output/'native-report.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result, indent=2))
    return 0 if result['outcome'].startswith('Scoped probe completed') else 1

if __name__ == '__main__':
    raise SystemExit(main())
