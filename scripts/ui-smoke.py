#!/usr/bin/env python3
"""Linux X11 native GPUI interaction/capture harness; captures only its own window.
Uses installed X11 APIs via ctypes, not a web UI or a downloaded test framework.
"""
import argparse, ctypes as C, hashlib, json, os, pathlib, struct, subprocess, time, zlib
from qualify import scoped_audit
ROOT=pathlib.Path(__file__).resolve().parents[1]
ART=ROOT/'artifacts';ART.mkdir(exist_ok=True)
x=C.CDLL('libX11.so.6')
Display=C.c_void_p;Window=C.c_ulong
def bind(name,restype,args):
    f=getattr(x,name);f.restype=restype;f.argtypes=args;return f
bind('XOpenDisplay',Display,[C.c_char_p]);bind('XDefaultRootWindow',Window,[Display]);bind('XQueryTree',C.c_int,[Display,Window,C.POINTER(Window),C.POINTER(Window),C.POINTER(C.POINTER(Window)),C.POINTER(C.c_uint)])
bind('XFetchName',C.c_int,[Display,Window,C.POINTER(C.c_char_p)]);bind('XFree',C.c_int,[C.c_void_p]);bind('XFlush',C.c_int,[Display]);bind('XSetInputFocus',C.c_int,[Display,Window,C.c_int,C.c_ulong]);bind('XStringToKeysym',C.c_ulong,[C.c_char_p]);bind('XKeysymToKeycode',C.c_uint,[Display,C.c_ulong]);bind('XSendEvent',C.c_int,[Display,Window,C.c_int,C.c_long,C.c_void_p]);bind('XGetGeometry',C.c_int,[Display,Window,C.POINTER(Window),C.POINTER(C.c_int),C.POINTER(C.c_int),C.POINTER(C.c_uint),C.POINTER(C.c_uint),C.POINTER(C.c_uint),C.POINTER(C.c_uint)])
class XKey(C.Structure):
    _fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',Display),('window',Window),('root',Window),('subwindow',Window),('time',C.c_ulong),('x',C.c_int),('y',C.c_int),('x_root',C.c_int),('y_root',C.c_int),('state',C.c_uint),('keycode',C.c_uint),('same_screen',C.c_int)]
class XImage(C.Structure):
    _fields_=[('width',C.c_int),('height',C.c_int),('xoffset',C.c_int),('format',C.c_int),('data',C.c_void_p),('byte_order',C.c_int),('bitmap_unit',C.c_int),('bitmap_bit_order',C.c_int),('bitmap_pad',C.c_int),('depth',C.c_int),('bytes_per_line',C.c_int),('bits_per_pixel',C.c_int),('red_mask',C.c_ulong),('green_mask',C.c_ulong),('blue_mask',C.c_ulong)]
bind('XGetImage',C.POINTER(XImage),[Display,Window,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_ulong,C.c_int]);bind('XDestroyImage',C.c_int,[C.POINTER(XImage)])
class Attr(C.Structure):
    _fields_=[('x',C.c_int),('y',C.c_int),('width',C.c_int),('height',C.c_int),('border',C.c_int),('depth',C.c_int),('visual',C.c_void_p),('root',Window),('klass',C.c_int),('bit_gravity',C.c_int),('win_gravity',C.c_int),('backing_store',C.c_int),('backing_planes',C.c_ulong),('backing_pixel',C.c_ulong),('save_under',C.c_int),('colormap',Window),('map_installed',C.c_int),('map_state',C.c_int),('all_event_masks',C.c_long),('your_event_mask',C.c_long),('do_not_propagate_mask',C.c_long),('override_redirect',C.c_int),('screen',C.c_void_p)]
bind('XGetWindowAttributes',C.c_int,[Display,Window,C.POINTER(Attr)])
bind('XInternAtom',C.c_ulong,[Display,C.c_char_p,C.c_int])
bind('XGetWindowProperty',C.c_int,[Display,Window,C.c_ulong,C.c_long,C.c_long,C.c_int,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_int),C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ubyte))])
def window_pid(d,w):
    typ=C.c_ulong();fmt=C.c_int();n=C.c_ulong();left=C.c_ulong();data=C.POINTER(C.c_ubyte)()
    x.XGetWindowProperty(d,w,x.XInternAtom(d,b'_NET_WM_PID',0),0,1,0,0,C.byref(typ),C.byref(fmt),C.byref(n),C.byref(left),C.byref(data))
    value=C.cast(data,C.POINTER(C.c_ulong))[0] if n.value and fmt.value==32 else 0
    if data:x.XFree(data)
    return value
def windows(d,parent,pid=None):
    root=Window();back=Window();children=C.POINTER(Window)();count=C.c_uint()
    if not x.XQueryTree(d,parent,C.byref(root),C.byref(back),C.byref(children),C.byref(count)):return
    values=[children[i] for i in range(count.value)]
    if children:x.XFree(children)
    for w in values:
        name=C.c_char_p()
        if x.XFetchName(d,w,C.byref(name)) and name.value:
            title=name.value.decode(errors='replace');x.XFree(name)
            if title=='Regressee — Phase 0' and (pid is None or window_pid(d,w)==pid):yield w
        yield from windows(d,w,pid)
def press(d,w,key,state=0):
    xt=C.CDLL('libXtst.so.6');xt.XTestFakeKeyEvent.argtypes=[Display,C.c_uint,C.c_int,C.c_ulong]
    code=lambda name:x.XKeysymToKeycode(d,x.XStringToKeysym(name.encode()))
    modifiers=[name for bit,name in [(1,'Shift_L'),(4,'Control_L')] if state&bit]
    for name in modifiers:xt.XTestFakeKeyEvent(d,code(name),1,0)
    xt.XTestFakeKeyEvent(d,code(key),1,0);xt.XTestFakeKeyEvent(d,code(key),0,0)
    for name in reversed(modifiers):xt.XTestFakeKeyEvent(d,code(name),0,0)
    x.XFlush(d);time.sleep(.08)
def wheel(d,w,button):
    xt=C.CDLL('libXtst.so.6')
    xt.XTestFakeMotionEvent.argtypes=[Display,C.c_int,C.c_int,C.c_int,C.c_ulong]
    xt.XTestFakeButtonEvent.argtypes=[Display,C.c_uint,C.c_int,C.c_ulong]
    bind('XTranslateCoordinates',C.c_int,[Display,Window,Window,C.c_int,C.c_int,C.POINTER(C.c_int),C.POINTER(C.c_int),C.POINTER(Window)])
    rx=C.c_int();ry=C.c_int();child=Window();x.XTranslateCoordinates(d,w,x.XDefaultRootWindow(d),350,500,C.byref(rx),C.byref(ry),C.byref(child))
    xt.XTestFakeMotionEvent(d,-1,rx.value,ry.value,0)
    xt.XTestFakeButtonEvent(d,button,1,0);xt.XTestFakeButtonEvent(d,button,0,0)
    x.XFlush(d);time.sleep(.5)
def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
def screenshot(d,w,dest,assert_fixture=False):
    root=Window();a=C.c_int();b=C.c_int();width=C.c_uint();height=C.c_uint();border=C.c_uint();depth=C.c_uint()
    x.XGetGeometry(d,w,C.byref(root),C.byref(a),C.byref(b),C.byref(width),C.byref(height),C.byref(border),C.byref(depth))
    ptr=x.XGetImage(d,w,0,0,width.value,height.value,0xffffffff,2)
    assert ptr,'native screenshot unavailable'
    image=ptr.contents;assert image.bits_per_pixel==32 and image.byte_order==0
    raw=C.string_at(image.data,image.bytes_per_line*image.height);rows=[];raster_pixels=0
    for y in range(image.height):
        row=raw[y*image.bytes_per_line:y*image.bytes_per_line+image.width*4];rgb=bytearray()
        for i in range(0,len(row),4):
            pixel=(row[i+2],row[i+1],row[i]);rgb.extend(pixel)
            if y>330 and i//4<740 and pixel in [(255,0,0),(0,255,0)]:raster_pixels+=1
        rows.append(b'\0'+rgb)
    dest.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',image.width,image.height,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(rows)))+chunk(b'IEND',b''));x.XDestroyImage(ptr)
    if assert_fixture:assert raster_pixels>500,f'fixture raster was not rendered: {raster_pixels} pixels'
    return [width.value,height.value]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--test-map',action='store_true',help='Explicit conditional test mapping of own PID window; NOT ordinary launch qualification');args=parser.parse_args()
    env={**os.environ,'VK_ICD_FILENAMES':'/usr/share/vulkan/icd.d/lvp_icd.json','REGRESSEE_UI_TEST':'1'};env.pop('WAYLAND_DISPLAY',None)
    mode='conditional' if args.test_map else 'default'
    def record(result):
        (ART/f'corrections-ui-smoke-{mode}.json').write_text(json.dumps(result,indent=2))
        (ART/'corrections-ui-smoke.json').write_text(json.dumps(result,indent=2))
    logfile=(ART/f'corrections-ui-smoke-{mode}.log').open('w');p=subprocess.Popen([str(ROOT/'target/debug/regressee-ui')],cwd=ROOT,env=env,stdout=logfile,stderr=logfile)
    d=x.XOpenDisplay(None)
    try:
        assert d,'X11 display not available; native tests NOT RUN'
        w=None
        for _ in range(100):
            assert p.poll() is None,'GPUI exited; inspect artifacts/ui-smoke.log'
            w=next(windows(d,x.XDefaultRootWindow(d),p.pid),None)
            if w:break
            time.sleep(.1)
        assert w,'GPUI window not found'
        attr=Attr()
        for _ in range(50):
            x.XGetWindowAttributes(d,w,C.byref(attr))
            if attr.map_state==2:break
            time.sleep(.1)
        if attr.map_state!=2 and args.test_map:
            # Conditional verifier-only workaround. Never applied by the app.
            class SetAttr(C.Structure):
                _fields_=[('background_pixmap',Window),('background_pixel',C.c_ulong),('border_pixmap',Window),('border_pixel',C.c_ulong),('bit_gravity',C.c_int),('win_gravity',C.c_int),('backing_store',C.c_int),('backing_planes',C.c_ulong),('backing_pixel',C.c_ulong),('save_under',C.c_int),('event_mask',C.c_long),('do_not_propagate_mask',C.c_long),('override_redirect',C.c_int),('colormap',Window),('cursor',Window)]
            bind('XChangeWindowAttributes',C.c_int,[Display,Window,C.c_ulong,C.POINTER(SetAttr)]);bind('XMapWindow',C.c_int,[Display,Window])
            change=SetAttr(override_redirect=1);x.XChangeWindowAttributes(d,w,512,C.byref(change));x.XMapWindow(d,w);x.XFlush(d);time.sleep(2);x.XGetWindowAttributes(d,w,C.byref(attr))
        assert attr.map_state==2,f'Owned GPUI window is not viewable (map_state={attr.map_state}); native input tests NOT RUN'
        x.XSetInputFocus(d,w,1,0);x.XFlush(d)
        dimensions=screenshot(d,w,ART/'corrections-ui-initial.png')
        press(d,w,'a',4)
        for key in ['slash','p','u','b','l','i','c']:press(d,w,key)
        for _ in range(4):press(d,w,'Tab')
        state=ART/'ui-state.json'
        original=(ART/'public-before.png').read_bytes();protected=hashlib.sha256(original).hexdigest()
        activated=ART/'ui-activation.json'
        activation=time.monotonic();start=time.time();press(d,w,'Return')
        for _ in range(50):
            if activated.exists() and activated.stat().st_mtime>start:break
            time.sleep(.1)
        assert activated.exists() and activated.stat().st_mtime>start,'native XTest activation NOT PROVEN: app did not enqueue work'
        time.sleep(.2);press(d,w,'Escape')
        for _ in range(50):
            if state.exists() and state.stat().st_mtime>start and 'cancelled' in json.loads(state.read_text())['status'].lower():break
            time.sleep(.1)
        assert state.exists() and state.stat().st_mtime>start and 'cancelled' in json.loads(state.read_text())['status'].lower(),'native XTest keyboard delivery NOT PROVEN: no fresh cancellation completion'
        activation_cleanup=time.monotonic()-activation;assert activation_cleanup<5,'native activation-to-cleanup exceeded 5s'
        assert hashlib.sha256((ART/'public-before.png').read_bytes()).hexdigest()==protected,'cancel changed original evidence'
        start=time.time();press(d,w,'Return')
        capture=ART/'ui-capture.json'
        for _ in range(300):
            if capture.exists() and capture.stat().st_mtime>start:break
            assert p.poll() is None,'GPUI exited during capture';time.sleep(.1)
        assert capture.exists() and capture.stat().st_mtime>start,'keyboard capture did not complete'
        time.sleep(1);screenshot(d,w,ART/'corrections-ui-public.png',assert_fixture=True)
        # Next focus target is Capture tall page.
        press(d,w,'Tab');start=time.time();press(d,w,'Return')
        for _ in range(300):
            if capture.stat().st_mtime>start and json.loads(capture.read_text())['dimensions']['height']>20000:break
            time.sleep(.1)
        assert capture.stat().st_mtime>start and json.loads(capture.read_text())['dimensions']['height']>20000,'fresh tall control failed'
        state=ART/'ui-state.json'
        summary={}
        for _ in range(300):
            if state.exists() and state.stat().st_mtime>start:
                summary=json.loads(state.read_text())
                if not summary['busy'] and summary['height'] and summary['height']>20000:break
            time.sleep(.1)
        assert summary['height']>20000 and not summary['busy'],'UI did not finish tiled decode'
        time.sleep(1);screenshot(d,w,ART/'corrections-ui-tall.png',assert_fixture=True)
        wheel(d,w,5);screenshot(d,w,ART/'corrections-ui-wheel.png')
        wheel_changed=(ART/'corrections-ui-tall.png').read_bytes()!=(ART/'corrections-ui-wheel.png').read_bytes()
        # From tall -> ready -> open login; then exercise actual idle Escape.
        for _ in range(3):press(d,w,'Tab',1)
        start=time.time();press(d,w,'Return')
        for _ in range(600):
            if state.exists() and state.stat().st_mtime>start and json.loads(state.read_text())['status'].startswith('Sign in'):break
            time.sleep(.1)
        assert state.stat().st_mtime>start and json.loads(state.read_text())['status'].startswith('Sign in'),'login control failed'
        start=time.time();cancel=time.monotonic();press(d,w,'Escape')
        for _ in range(50):
            if state.stat().st_mtime>start and json.loads(state.read_text())['status'].startswith('Cancelled. Managed browser closed'):break
            time.sleep(.1)
        assert state.stat().st_mtime>start and json.loads(state.read_text())['status'].startswith('Cancelled. Managed browser closed'),'idle Escape did not complete cleanup'
        idle_cleanup=time.monotonic()-cancel
        press(d,w,'q',4);p.wait(timeout=5);assert p.returncode==0
        result={'outcome':'CONDITIONAL PASS' if args.test_map else 'PASS at scoped assertions','pid':p.pid,'map_state':attr.map_state,'test_mapping_requested':args.test_map,'native_dimensions':dimensions,'backend':'X11 Vulkan lavapipe software rendering','keyboard_capture_and_actual_raster_pixels':True,'activation_to_cancel_cleanup_seconds':round(activation_cleanup,3),'idle_login_escape_cleanup_seconds':round(idle_cleanup,3),'quit_exit':p.returncode,'wheel':'pixels changed; inspect tile identity' if wheel_changed else 'NOT PROVEN: valid XTest delivery produced no pixel change','not_tested':['human login/ready/private chain','file dialog completion','native menu interaction','actual screen-reader accessibility','Windows','macOS','hardware GPU']}
        record(result);print(json.dumps(result))
    except Exception as error:
        result={'outcome':'FAIL','reason':str(error),'pid':p.pid,'test_mapping_requested':args.test_map,'remaining_native_tests':'NOT RUN after failed prerequisite'}
        record(result);raise
    finally:
        if p.poll() is None:
            p.terminate()
            try:p.wait(timeout=5)
            except subprocess.TimeoutExpired:p.kill();p.wait()
        logfile.close()
        audit=scoped_audit();result['final_scoped_audit']=audit
        if any(audit.values()):result['outcome']='FAIL';result['cleanup_reason']='Final scoped process/scratch audit failed'
        record(result);assert not any(audit.values()),f'Native final cleanup audit: {audit}'
if __name__=='__main__':main()
