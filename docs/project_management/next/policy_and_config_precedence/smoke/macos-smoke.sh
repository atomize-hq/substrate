#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: policy/config precedence macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 3
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq not found on PATH" >&2
  exit 3
fi

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
TMP_NOWS="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS" "$TMP_NOWS"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$TMP_HOME"
export HOME="$TMP_HOME"

substrate workspace init "$TMP_WS" >/dev/null
cd "$TMP_WS"

substrate config set world.caged=false >/dev/null
out="$(SUBSTRATE_CAGED=1 substrate config show --json)"
if ! jq -e '.world.caged==false' <<<"$out" >/dev/null; then
  got="$(jq -r '.world.caged' <<<"$out" 2>/dev/null || true)"
  echo "FAIL: expected world.caged=false from workspace config even when SUBSTRATE_CAGED=1; got world.caged=$got" >&2
  exit 1
fi

cd "$TMP_NOWS"
set +e
substrate config show --json >/dev/null 2>&1
code=$?
set -e
if [[ "$code" -ne 2 ]]; then
  echo "FAIL: expected exit code 2 for workspace-scoped config show without a workspace; got $code" >&2
  exit 1
fi

echo "OK: policy/config precedence macOS smoke"
