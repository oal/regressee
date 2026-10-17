#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Compatibility entrypoint for the production preview build/package path.

Historical Phase 7 debug app.tar.gz receipts remain historical. New candidates
use release-preview.py's CLI/desktop products, selected sources and notices.
No signing, upload, publication or native/clean-machine approval is performed.
"""
import runpy
from pathlib import Path

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).with_name("release-preview.py")), run_name="__main__")
