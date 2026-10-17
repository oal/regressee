#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""One owned-window inspection; stop on unmapped/blank, never force mapping."""
import argparse
import ctypes as C
import importlib.util
import json
import os
import pathlib
import subprocess
import time
from development_tools import require_developer_binary

ROOT=pathlib.Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
    output=a.output.resolve();output.mkdir(parents=True,exist_ok=False)
    require_developer_binary(ROOT/'target/debug/regressee-desktop', output/'native-data')
    spec=importlib.util.spec_from_file_location('owned_x11',ROOT/'scripts/ui-smoke.py');h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.bind('XCloseDisplay',C.c_int,[h.Display]);x=h.x
    result={'scope':'One X11/lavapipe developer inspection; not ordinary native qualification','outcome':'FAIL / native evidence open','forced_mapping':False,'input':'NOT RUN','human_visual_approval':'PENDING'}
    display=None;process=None
    def owned(parent,pid):
        root=h.Window();back=h.Window();children=C.POINTER(h.Window)();count=C.c_uint()
        if not x.XQueryTree(display,parent,C.byref(root),C.byref(back),C.byref(children),C.byref(count)):return
        values=[children[i] for i in range(count.value)]
        if children:x.XFree(children)
        for window in values:
            name=C.c_char_p()
            if x.XFetchName(display,window,C.byref(name)) and name.value:
                title=name.value.decode(errors='replace');x.XFree(name)
                if title=='Regressee — Single-page preview' and h.window_pid(display,window)==pid:yield window
            yield from owned(window,pid)
    env=dict(os.environ,REGRESSEE_DEV_DATA=str(output/'native-data'),VK_ICD_FILENAMES='/usr/share/vulkan/icd.d/lvp_icd.json');env.pop('WAYLAND_DISPLAY',None);env.pop('REGRESSEE_DISTRIBUTION',None)
    try:
        with (output/'app.log').open('w') as log:
            process=subprocess.Popen([str(ROOT/'target/debug/regressee-desktop')],cwd=ROOT,env=env,stdout=log,stderr=log)
            result['pid']=process.pid;display=x.XOpenDisplay(None);assert display,'X11 unavailable'
            window=None
            for _ in range(100):
                assert process.poll() is None,'product exited before window inspection'
                window=next(owned(x.XDefaultRootWindow(display),process.pid),None)
                if window:break
                time.sleep(.1)
            assert window,'owned product window not found';result['window']=int(window);attr=h.Attr()
            for _ in range(50):
                x.XGetWindowAttributes(display,window,C.byref(attr))
                if attr.map_state==2:break
                time.sleep(.1)
            result['map_state']=attr.map_state;assert attr.map_state==2,'owned window unmapped; stopped without forced mapping or input'
            time.sleep(.5);h.screenshot(display,window,output/'projects.png')
            # Read image pixels through the existing bounded helper's PNG output.
            import struct,zlib
            data=(output/'projects.png').read_bytes();at=8;compressed=bytearray()
            while at<len(data):
                n=struct.unpack('>I',data[at:at+4])[0]
                if data[at+4:at+8]==b'IDAT':compressed.extend(data[at+8:at+8+n])
                at+=n+12
            raw=zlib.decompress(compressed);assert len(set(raw))>8,'blank pixels; stop without input/retry'
            result['pixels']='Meaningful pixels acquired; human review pending'
            # Restrict every event to the PID-owned window. This is navigation
            # only, not product capture or desktop accessibility qualification.
            x.XSetInputFocus(display,window,1,0);x.XFlush(display)
            for key,name in [('2','setup'),('3','results'),('5','settings')]:
                h.press(display,window,key,4);time.sleep(.3);h.screenshot(display,window,output/f'{name}.png')
            result['input']='Owned navigation events sent; semantic/human review pending'
            result['outcome']='Scoped screenshots acquired; NOT release/native acceptance'
    except Exception as e:result['reason']=str(e)
    finally:
        if process and process.poll() is None:
            process.terminate()
            try:process.wait(timeout=5)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        if display:x.XCloseDisplay(display)
        result['own_process_reaped']=bool(process and process.poll() is not None)
        (output/'native-report.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
    return int(result['outcome'].startswith('FAIL'))

if __name__=='__main__':raise SystemExit(main())
