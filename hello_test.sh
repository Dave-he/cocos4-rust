#!/usr/bin/env bash
# Phase 1 — trivial test that the hello target prints the expected line.
set -euo pipefail
out=$(./hello)
expected="Bazel up (cocos4-rust phase-1 scaffold)"
if [[ "$out" != "$expected" ]]; then
    echo "FAIL: expected '$expected', got '$out'" >&2
    exit 1
fi
echo "OK: hello printed the expected line"
