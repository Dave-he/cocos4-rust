#!/usr/bin/env bash
# Round 48 — build the WASM bridge for AGI-miniGame's scene_gen
# consumption. Produces an ES-module .js + .wasm pair under
# ../AGI-miniGame/wasm-pkg/ that the TS layer can `import()`
# at runtime.
#
# Usage (from cocos4-rust repo root):
#   ./scripts/build-wasm.sh           # release build
#   ./scripts/build-wasm.sh dev       # dev build (faster, no opt)

set -euo pipefail

MODE="${1:-release}"
OUT_DIR="../AGI-miniGame/wasm-pkg"

if [[ "$MODE" == "dev" ]]; then
    BUILD_FLAG="--dev"
elif [[ "$MODE" == "release" ]]; then
    BUILD_FLAG="--release"
else
    echo "Unknown mode '$MODE' (use 'release' or 'dev')." >&2
    exit 1
fi

echo "→ wasm-pack build ($MODE) into $OUT_DIR"
wasm-pack build \
    --target web \
    "$BUILD_FLAG" \
    --out-dir "$OUT_DIR" \
    -- \
    --no-default-features \
    --features wasm-bindings

echo "✓ wasm-pkg artifacts:"
ls -lh "$OUT_DIR" | tail -n +2 | awk '{ printf "    %s  %s\n", $5, $9 }'
