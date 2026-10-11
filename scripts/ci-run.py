#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Run hosted CI commands with files owned by the actual Windows token user.

Elevated Windows runners default new objects to the Administrators group. Set
only this process token's default owner to its user before spawning the command.
Do not take ownership of existing paths or weaken the product's owner/ACL checks.
"""
import ctypes
import os
import subprocess
import sys
from ctypes import wintypes


def windows_user_owner():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("Windows token setup is restricted to hosted CI")
    advapi = ctypes.WinDLL("advapi32", use_last_error=True)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    advapi.OpenProcessToken.argtypes = [
        wintypes.HANDLE, wintypes.DWORD, ctypes.POINTER(wintypes.HANDLE)
    ]
    advapi.OpenProcessToken.restype = wintypes.BOOL
    advapi.GetTokenInformation.argtypes = [
        wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD,
        ctypes.POINTER(wintypes.DWORD)
    ]
    advapi.GetTokenInformation.restype = wintypes.BOOL
    advapi.SetTokenInformation.argtypes = [
        wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD
    ]
    advapi.SetTokenInformation.restype = wintypes.BOOL
    advapi.EqualSid.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    advapi.EqualSid.restype = wintypes.BOOL

    class TokenOwner(ctypes.Structure):
        _fields_ = [("sid", ctypes.c_void_p)]

    def check(ok):
        if not ok:
            raise ctypes.WinError(ctypes.get_last_error())

    def information(token, kind):
        size = wintypes.DWORD()
        advapi.GetTokenInformation(token, kind, None, 0, ctypes.byref(size))
        if not size.value:
            raise ctypes.WinError(ctypes.get_last_error())
        buffer = ctypes.create_string_buffer(size.value)
        check(advapi.GetTokenInformation(token, kind, buffer, size, ctypes.byref(size)))
        return buffer

    token = wintypes.HANDLE()
    # TOKEN_QUERY | TOKEN_ADJUST_DEFAULT; no ownership/restore privileges.
    check(advapi.OpenProcessToken(kernel.GetCurrentProcess(), 0x0088, ctypes.byref(token)))
    try:
        user = information(token, 1)  # TokenUser starts with SID_AND_ATTRIBUTES.
        sid = ctypes.cast(user, ctypes.POINTER(ctypes.c_void_p)).contents.value
        owner = TokenOwner(sid)
        check(advapi.SetTokenInformation(token, 4, ctypes.byref(owner), ctypes.sizeof(owner)))
        actual = information(token, 4)
        actual_sid = ctypes.cast(actual, ctypes.POINTER(TokenOwner)).contents.sid
        check(advapi.EqualSid(sid, actual_sid))
    finally:
        kernel.CloseHandle(token)
    print("CI child processes create user-owned files; product ACL checks remain enabled.", flush=True)


def main():
    if len(sys.argv) < 2:
        raise SystemExit("Usage: ci-run.py COMMAND [ARGS...]")
    if os.name == "nt":
        windows_user_owner()
    return subprocess.call(sys.argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
