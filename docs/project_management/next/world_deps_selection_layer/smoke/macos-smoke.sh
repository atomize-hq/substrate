#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: world-deps selection macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

if ! substrate world deps --help 2>/dev/null | grep -q "init"; then
  echo "SKIP: world-deps selection smoke (world deps init not available in this build)"
  exit 0
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq is required for world-deps selection smoke" >&2
  exit 1
fi

WDL_TEST_HOME="$(mktemp -d)"
WDL_TEST_WS="$(mktemp -d)"
cleanup() { rm -rf "$WDL_TEST_HOME" "$WDL_TEST_WS"; }
trap cleanup EXIT

cd "$WDL_TEST_WS"
mkdir -p .substrate

set +e
out="$(HOME="$WDL_TEST_HOME" substrate world deps status 2>&1)"
code=$?
set -e
test "$code" -eq 0
echo "$out" | grep -q "not configured"

HOME="$WDL_TEST_HOME" substrate world deps init --workspace --force >/dev/null
HOME="$WDL_TEST_HOME" substrate world deps status --json | jq -e '.selection.configured==true' >/dev/null

echo "OK: world-deps selection macOS smoke (gating)"
