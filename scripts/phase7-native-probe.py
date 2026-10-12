#!/usr/bin/env python3
"""Owned X11 process only. OS XTEST input, never GPUI TestAppContext.

Does not visit sites, install anything or manipulate another application's window.
Creates a fresh approved temporary data root; preserves every artifact on exit.
OS screenshots can be black: input delivery alone is NOT visible-task acceptance.
"""
import argparse
import ctypes as C
import ctypes.util
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import time
import zlib


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--parent", required=True, type=Path)
    parser.add_argument("--backend", choices=["x11", "wayland"], default="x11")
    args = parser.parse_args()
    parent = args.parent.resolve(strict=True)
    assert parent == Path("/var/home/olav/Data/opencode/tmp/opencode")
    binary = args.binary.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="regressee-phase7-native-", dir=parent))
    report = {"root": str(root), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "kind": "real X11/XTEST OS input on an isolated owned process; not human qualification",
              "events": [], "native_acceptance": "NOT TESTED until painted outcomes inspected"}
    report["environment"] = {"backend":args.backend,"DISPLAY":os.getenv("DISPLAY"),
                             "WAYLAND_DISPLAY":os.getenv("WAYLAND_DISPLAY"),"VK_DRIVER_FILES":os.getenv("VK_DRIVER_FILES")}
    x = C.CDLL(ctypes.util.find_library("X11"))
    xt = C.CDLL(ctypes.util.find_library("Xtst"))
    def bind(lib, name, restype, *types):
        f = getattr(lib, name); f.restype = restype; f.argtypes = list(types); return f
    ptr, ul, ui, integer = C.c_void_p, C.c_ulong, C.c_uint, C.c_int
    open_display = bind(x, "XOpenDisplay", ptr, C.c_char_p)
    d = open_display(None)
    if not d:
        report["blocker"] = "X11 display inaccessible"
        (root / "native.json").write_text(json.dumps(report, indent=2)); print(root); return
    default_root = bind(x, "XDefaultRootWindow", ul, ptr)(d)
    # Keep X protocol failures checkable and allow the process-owner finally block
    # to run. Xlib's default handler exits immediately and leaks child ownership.
    errors = []
    class XError(C.Structure):
        _fields_ = [("type",integer),("display",ptr),("resource",ul),("serial",ul),
                    ("error",C.c_ubyte),("request",C.c_ubyte),("minor",C.c_ubyte)]
    handler_type = C.CFUNCTYPE(integer, ptr, ptr)
    def on_error(_display, error):
        e=C.cast(error,C.POINTER(XError)).contents
        errors.append({"error":e.error,"request":e.request,"minor":e.minor,"resource":e.resource})
        return 0
    handler = handler_type(on_error)
    bind(x, "XSetErrorHandler", ptr, handler_type)(handler)
    report["x_errors"] = errors
    atom = bind(x, "XInternAtom", ul, ptr, C.c_char_p, integer)
    prop = bind(x, "XGetWindowProperty", integer, ptr, ul, ul, C.c_long, C.c_long, integer, ul,
                C.POINTER(ul), C.POINTER(integer), C.POINTER(ul), C.POINTER(ul), C.POINTER(ptr))
    free = bind(x, "XFree", integer, ptr)
    query = bind(x, "XQueryTree", integer, ptr, ul, C.POINTER(ul), C.POINTER(ul), C.POINTER(C.POINTER(ul)), C.POINTER(ui))
    sync = bind(x, "XSync", integer, ptr, integer)
    def property(w, name):
        actual, fmt, count, left, value = ul(), integer(), ul(), ul(), ptr()
        prop(d, w, atom(d, name.encode(), 0), 0, 1024, 0, 0,
             C.byref(actual), C.byref(fmt), C.byref(count), C.byref(left), C.byref(value))
        if not value: return None
        try:
            if fmt.value == 32: return list(C.cast(value, C.POINTER(ul))[:count.value])
            return C.string_at(value, count.value)
        finally: free(value)
    def children(w):
        r, p, c, n = ul(), ul(), C.POINTER(ul)(), ui()
        query(d, w, C.byref(r), C.byref(p), C.byref(c), C.byref(n))
        result = list(c[:n.value]) if c else []
        if c: free(c)
        return result
    env = os.environ.copy()
    if args.backend == "x11": env.pop("WAYLAND_DISPLAY", None)
    # Never accept user/development runtime override in a qualification process.
    for name in list(env):
        if name.startswith("REGRESSEE_"): env.pop(name)
    env["REGRESSEE_DEV_DATA"] = str(root / "app")
    env["REGRESSEE_REDUCED_MOTION"] = "1"
    env["TMPDIR"] = str(parent)
    output = (root / "launch.log").open("wb")
    process = subprocess.Popen([str(binary)], env=env, stdout=output, stderr=subprocess.STDOUT)
    try:
        def accessibility():
            a=C.CDLL(ctypes.util.find_library("atspi"))
            init=bind(a,"atspi_init",integer)
            desktops=bind(a,"atspi_get_desktop_count",integer)
            desktop=bind(a,"atspi_get_desktop",ptr,integer)
            count=bind(a,"atspi_accessible_get_child_count",integer,ptr,ptr)
            child=bind(a,"atspi_accessible_get_child_at_index",ptr,ptr,integer,ptr)
            pid=bind(a,"atspi_accessible_get_process_id",ui,ptr,ptr)
            result={"init":init(),"desktop_count":desktops(),"owned_pid_registered":False,
                    "qualification":"actual read-only AT-SPI registry during owned process; not a screen-reader/user pass"}
            for i in range(result["desktop_count"]):
                desk=desktop(i)
                for j in range(count(desk,None)):
                    if pid(child(desk,j,None),None)==process.pid:result["owned_pid_registered"]=True
            return result
        if args.backend == "wayland":
            time.sleep(8)
            report["wayland_process_alive_after_8s"] = process.poll() is None
            report["accessibility"] = accessibility()
            report["blocker"] = "Wayland OS-input/screenshot/IME automation unavailable; no nested compositor or physical input permission. No input pass."
            return
        window = None
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline and process.poll() is None:
            pending = children(default_root)
            while pending:
                w = pending.pop()
                title = property(w, "_NET_WM_NAME") or property(w, "WM_NAME") or b""
                if property(w, "_NET_WM_PID") == [process.pid] and isinstance(title, bytes) and b"Regressee" in title:
                    window = w; break
                pending.extend(children(w))
            if window: break
            time.sleep(.2)
        if not window:
            report["blocker"] = "owned app window unavailable; see launch.log"
            return
        report["window"] = window; report["pid"] = process.pid
        report["accessibility"] = accessibility()
        class Attributes(C.Structure):
            _fields_=[("x",integer),("y",integer),("width",integer),("height",integer),("border_width",integer),("depth",integer),
                      ("visual",ptr),("root",ul),("window_class",integer),("bit_gravity",integer),("win_gravity",integer),
                      ("backing_store",integer),("backing_planes",ul),("backing_pixel",ul),("save_under",integer),
                      ("colormap",ul),("map_installed",integer),("map_state",integer),("all_event_masks",C.c_long),
                      ("your_event_mask",C.c_long),("do_not_propagate_mask",C.c_long),("override_redirect",integer),("screen",ptr)]
        attributes=Attributes()
        get_attributes=bind(x,"XGetWindowAttributes",integer,ptr,ul,C.POINTER(Attributes))
        bind(x, "XMapWindow", integer, ptr, ul)(d, window)
        sync(d, 0); time.sleep(1)
        get_attributes(d,window,C.byref(attributes))
        report["window_attributes"]={"width":attributes.width,"height":attributes.height,"depth":attributes.depth,"class":attributes.window_class,"map_state":attributes.map_state}
        if attributes.map_state != 2:
            report["blocker"]="owned native window is not viewable; refusing OS input into an unpainted/unmapped window"
            return
        focus = bind(x, "XSetInputFocus", integer, ptr, ul, integer, ul)
        get_focus = bind(x, "XGetInputFocus", integer, ptr, C.POINTER(ul), C.POINTER(integer))
        bind(x, "XRaiseWindow", integer, ptr, ul)(d, window)
        focus(d, window, 2, 0); sync(d, 0); time.sleep(.5)
        def guard():
            w, revert = ul(), integer()
            get_focus(d, C.byref(w), C.byref(revert))
            assert process.poll() is None and property(window, "_NET_WM_PID") == [process.pid]
            assert w.value == window, "focus left owned process; stop rather than type into another app"
        keycode = bind(x, "XKeysymToKeycode", ui, ptr, ul)
        keysym = bind(x, "XStringToKeysym", ul, C.c_char_p)
        fake_key = bind(xt, "XTestFakeKeyEvent", integer, ptr, ui, integer, ul)
        def key(name, control=False):
            guard()
            ctrl = keycode(d, keysym(b"Control_L"))
            shift = keycode(d, keysym(b"Shift_L"))
            code = keycode(d, keysym(name.encode()))
            assert code, name
            if control: fake_key(d, ctrl, 1, 0)
            if name == "colon": fake_key(d, shift, 1, 0)
            fake_key(d, code, 1, 0); fake_key(d, code, 0, 0)
            if name == "colon": fake_key(d, shift, 0, 0)
            if control: fake_key(d, ctrl, 0, 0)
            sync(d, 0); time.sleep(.07)
            report["events"].append(("ctrl-" if control else "") + name)
        def text(value):
            for c in value:
                key({" ": "space", ":": "colon", "/": "slash", ".": "period", "-": "minus"}.get(c, c))
        # OS image read, not a GPU/offscreen substitute.
        class Image(C.Structure):
            _fields_ = [("width", integer), ("height", integer), ("xoffset", integer), ("format", integer),
                        ("data", ptr), ("byte_order", integer), ("bitmap_unit", integer), ("bitmap_bit_order", integer),
                        ("bitmap_pad", integer), ("depth", integer), ("bytes_per_line", integer), ("bits_per_pixel", integer),
                        ("red_mask", ul), ("green_mask", ul), ("blue_mask", ul)]
        get_geometry = bind(x, "XGetGeometry", integer, ptr, ul, C.POINTER(ul), C.POINTER(integer), C.POINTER(integer), C.POINTER(ui), C.POINTER(ui), C.POINTER(ui), C.POINTER(ui))
        get_image = bind(x, "XGetImage", C.POINTER(Image), ptr, ul, integer, integer, ui, ui, ul, integer)
        destroy_image = bind(x, "XDestroyImage", integer, C.POINTER(Image))
        report["screenshots"] = []
        def screenshot(name):
            time.sleep(.5); sync(d, 0)
            r, gx, gy, w, h, border, depth = ul(), integer(), integer(), ui(), ui(), ui(), ui()
            get_geometry(d, window, C.byref(r), C.byref(gx), C.byref(gy), C.byref(w), C.byref(h), C.byref(border), C.byref(depth))
            image = get_image(d, window, 0, 0, w.value, h.value, (1 << 64)-1, 2)
            assert image
            try:
                im = image.contents
                raw = C.string_at(im.data, im.bytes_per_line * im.height)
                stride = im.bits_per_pixel // 8
                rows = []
                for y in range(im.height):
                    row = bytearray([0])
                    for xx in range(im.width):
                        offset = y * im.bytes_per_line + xx * stride
                        pixel = int.from_bytes(raw[offset:offset+stride], "little" if im.byte_order == 0 else "big")
                        for mask in [im.red_mask, im.green_mask, im.blue_mask]:
                            shift = (mask & -mask).bit_length()-1
                            row.append(((pixel & mask) >> shift) * 255 // (mask >> shift))
                    rows.append(bytes(row))
                def chunk(kind, data):
                    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind+data))
                png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", im.width, im.height, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(b"".join(rows))) + chunk(b"IEND", b"")
                (root / (name+".png")).write_bytes(png)
                report["screenshots"].append({"name": name, "width": im.width, "height": im.height,
                    "sha256": hashlib.sha256(png).hexdigest(), "distinct_source_bytes": len(set(raw))})
            finally: destroy_image(image)
        screenshot("projects")
        key("n", True); text("phase7-owned-native"); key("Tab"); text("http://127.0.0.1:43210/")
        screenshot("website-draft")
        key("s", True); time.sleep(1); screenshot("saved-access")
        key("F1"); screenshot("help"); key("Escape"); screenshot("help-dismissed")
        # Actual host resize; source Fit/manual assertions are qualified separately.
        bind(x, "XResizeWindow", integer, ptr, ul, ui, ui)(d, window, 820, 600)
        sync(d, 0); screenshot("minimum")
        guard()
        # One pointer click inside the owned window; do not activate a browser visit.
        translate = bind(x, "XTranslateCoordinates", integer, ptr, ul, ul, integer, integer, C.POINTER(integer), C.POINTER(integer), C.POINTER(ul))
        rx, ry, child = integer(), integer(), ul()
        translate(d, window, default_root, 50, 30, C.byref(rx), C.byref(ry), C.byref(child))
        bind(xt, "XTestFakeMotionEvent", integer, ptr, integer, integer, integer, ul)(d, -1, rx, ry, 0)
        button = bind(xt, "XTestFakeButtonEvent", integer, ptr, ui, integer, ul)
        guard(); button(d, 1, 1, 0); button(d, 1, 0, 0); sync(d, 0)
        report["events"].append("owned-pointer-click")
        screenshot("pointer")
        report["delivery"] = "completed OS event sequence; inspect screenshots and persisted settings before acceptance"
    except Exception as e:
        report["blocker"] = repr(e)
    finally:
        if process.poll() is None:
            process.terminate()
            try: process.wait(timeout=8)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=3)
        report["exit"] = process.returncode
        output.close()
        (root / "native.json").write_text(json.dumps(report, indent=2))
        bind(x, "XCloseDisplay", integer, ptr)(d)
        print(root)


if __name__ == "__main__":
    main()
