#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: policy/config precedence macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq not found on PATH" >&2
  exit 3
fi

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$TMP_HOME"
export HOME="$TMP_HOME"

substrate workspace init "$TMP_WS" >/dev/null
cd "$TMP_WS"

substrate config set world.caged=false >/dev/null
SUBSTRATE_CAGED=1 substrate config show --json | jq -e '.world.caged==false' >/dev/null

echo "OK: policy/config precedence macOS smoke"

