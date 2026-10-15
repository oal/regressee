#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Local verification only. Does not install tools or provision browser runtimes.
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/build-ui.sh
if [[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]]; then
  export LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
cargo fmt --all -- --check
cargo test --locked --workspace --all-features -j 4
cargo clippy --locked --workspace --all-targets --all-features -j 4 -- -D warnings
