#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Permit sandbox user namespaces for only this hosted job's pinned Chromium.

Ubuntu 24.04 restricts unconfined user namespaces through AppArmor. An executable
profile grants userns only to the downloaded browser in the three Phase 0 pack
locations. Never disable AppArmor, change sysctls or pass --no-sandbox.
"""
import os
from pathlib import Path
import subprocess
import sys


def main():
    if sys.platform != "linux" or os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("Browser policy setup is restricted to Linux hosted CI")
    restriction = Path("/proc/sys/kernel/apparmor_restrict_unprivileged_userns")
    if not restriction.exists() or restriction.read_text().strip() == "0":
        print("Runner user namespaces are already available; no policy changes.")
        return
    root = Path(os.environ["GITHUB_WORKSPACE"]).resolve()
    source = root / ".runtime/source"
    browsers = list(source.glob("browsers/chromium-*/chrome-linux*/chrome"))
    if len(browsers) != 1:
        raise RuntimeError("Expected exactly one pinned Linux Chromium executable")
    relative = browsers[0].relative_to(source)
    profiles = ["abi <abi/4.0>,", "include <tunables/global>"]
    for index, location in enumerate(("source", "installed.stage", "installed")):
        executable = str(root / ".runtime" / location / relative)
        if any(char in executable for char in '\n\r"\\*?[]{}'):
            raise RuntimeError("Unsafe AppArmor executable path")
        profiles.append(
            f'profile regressee-ci-chromium-{index} "{executable}" flags=(unconfined) {{\n'
            '  userns,\n}'
        )
    subprocess.run(
        ["sudo", "apparmor_parser", "--replace"],
        input="\n".join(profiles) + "\n", text=True, check=True
    )
    print("Allowed sandbox user namespaces for only the pinned CI browser executables.")


if __name__ == "__main__":
    main()
