#!/usr/bin/env bash
# Existing SDK probe only; never installs/publishes a Flatpak or disables sandbox.
set -euo pipefail
cd "$(dirname "$0")/.."
expected=0cc82216a407cc993941b5ddabd446becc3c9a6219d9bcf50125c60101dcad46
actual="$(flatpak info --show-commit org.freedesktop.Sdk//25.08)"
[[ "$actual" == "$expected" ]] || { echo "Pinned SDK commit is unavailable; probe not run." >&2; exit 2; }
timeout --signal=TERM 35s flatpak run --die-with-parent --filesystem="$PWD:ro" \
  --env="REGRESSEE_PROBE_ROOT=$PWD" \
  --env="PLAYWRIGHT_BROWSERS_PATH=$PWD/.runtime/source/browsers" \
  --env=TMPDIR=/tmp/opencode \
  --command="$PWD/.runtime/source/node/bin/node" org.freedesktop.Sdk//25.08 --input-type=module <<'JS'
import fs from 'node:fs/promises';
await fs.mkdir('/tmp/opencode',{recursive:true,mode:0o700});
const root=process.env.REGRESSEE_PROBE_ROOT;
const {launch,replay,capture}=await import('file://'+root+'/.runtime/source/worker/src/capture.mjs');
const {fixtureServer}=await import('file://'+root+'/.runtime/source/worker/src/fixtures.mjs');
let b,s;
try{s=await fixtureServer();b=await launch(false);const c=await replay(b);const result=await capture(c,s.origin+'/public');console.log(JSON.stringify({probe:'SDK only; not a packaged app',sandbox:true,bytes:result.image.length,browser:b.version()}));}
catch(e){console.error(e.message.includes('Chromium sandboxing failed')?'FAIL: Chromium sandboxing failed inside this pinned Flatpak SDK. No sandbox override attempted.':'FAIL: private browser launch/capture failed inside this SDK.');process.exitCode=1;}
finally{await b?.close();await s?.close();}
JS
