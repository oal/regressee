#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Explicit developer-tool boundaries; no fixed host paths or application launch."""
import json
import os
from pathlib import Path
import stat
import subprocess


def private_directory(path):
    path = Path(path)
    metadata = path.lstat()
    if (not path.is_absolute() or path.resolve(strict=True) != path
            or not stat.S_ISDIR(metadata.st_mode)
            or getattr(metadata, "st_file_attributes", 0) & 0x400):
        raise ValueError("An absolute physical private directory is required")
    if os.name == "posix" and (metadata.st_uid != os.geteuid()
                               or stat.S_IMODE(metadata.st_mode) != 0o700):
        raise ValueError("Directory must be current-user-owned mode 0700")
    return path


def require_developer_binary(binary, isolated_root):
    # Current binaries return metadata before graphics/storage startup. Keep an
    # isolated override even for an accidentally supplied older executable.
    result = subprocess.run(
        [str(binary), "--build-info"], check=True, capture_output=True, text=True,
        timeout=10, env={**os.environ, "REGRESSEE_DEV_DATA": str(isolated_root)},
    )
    info = json.loads(result.stdout)
    if info.get("features", {}).get("developer-tools") is not True:
        raise ValueError("Build this diagnostic executable with --features ui,developer-tools")
