#!/usr/bin/env bash
# Development convenience for THIS Linux x64 userspace, never host installation.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]]; then
  mkdir -p .runtime/link-libs
  # GPUI uses these dynamic libraries; this host lacks only their dev linker names.
  for spec in 'xcb:libxcb.so.1' 'xkbcommon-x11:libxkbcommon-x11.so.0'; do
    name="${spec%%:*}"
    soname="${spec#*:}"
    source="/usr/lib/x86_64-linux-gnu/$soname"
    test -f "$source" || { echo "Missing $soname. Use explicit supported-OS prerequisites." >&2; exit 1; }
    if [[ ! -e ".runtime/link-libs/lib$name.so" ]]; then
      ln -s "$source" ".runtime/link-libs/lib$name.so"
    fi
  done
  export LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
cargo build --locked --workspace --features regressee-gui/ui,regressee-spike/ui --bins -j 4
