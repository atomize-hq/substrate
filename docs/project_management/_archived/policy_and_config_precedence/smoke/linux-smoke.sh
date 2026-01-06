#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: policy/config precedence linux smoke (not Linux)"
  exit 0
fi

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "MISSING: $name not found on PATH" >&2
    exit 3
  fi
}

need_cmd substrate
need_cmd jq
need_cmd mktemp

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
  fail "expected world.caged=false from workspace config even when SUBSTRATE_CAGED=1; got world.caged=$got"
fi

cd "$TMP_NOWS"
set +e
substrate config show --json >/dev/null 2>&1
code=$?
set -e
if [[ "$code" -ne 2 ]]; then
  fail "expected exit code 2 for workspace-scoped config show without a workspace; got $code"
fi

echo "OK: policy/config precedence linux smoke"
