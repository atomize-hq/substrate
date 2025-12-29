#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: p0 platform stability linux smoke (not Linux)"
  exit 0
fi

ORIG_HOME="${HOME:-}"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || fail "$name not found on PATH"
}

need_cmd jq
need_cmd mktemp

json_allow_nonzero() {
  local out
  set +e
  out="$("$@" 2>/dev/null)"
  local status=$?
  set -e
  if [[ -z "${out:-}" ]]; then
    fail "expected JSON output, got empty output: $* (exit=$status)"
  fi
  printf '%s' "$out"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../../.." && pwd)"

SUBSTRATE_BIN="${SUBSTRATE_BIN:-}"
if [[ -z "$SUBSTRATE_BIN" ]]; then
  if [[ -x "$ROOT_DIR/target/debug/substrate" ]]; then
    SUBSTRATE_BIN="$ROOT_DIR/target/debug/substrate"
  elif command -v substrate >/dev/null 2>&1; then
    SUBSTRATE_BIN="$(command -v substrate)"
  else
    fail "substrate binary not found (set SUBSTRATE_BIN or build ./target/debug/substrate or install substrate)"
  fi
fi

TMP_HOME="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME"; }
trap cleanup EXIT

export HOME="$TMP_HOME"
export SUBSTRATE_HOME="$TMP_HOME"

echo "SMOKE: init isolated global config/policy"
"$SUBSTRATE_BIN" config global init --force >/dev/null
"$SUBSTRATE_BIN" policy global init --force >/dev/null

echo "SMOKE: world doctor (enabled + disabled) JSON parses"
SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 "$SUBSTRATE_BIN" world doctor --json | jq -e '.world_socket != null' >/dev/null
SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 "$SUBSTRATE_BIN" world doctor --json | jq -e '.' >/dev/null

echo "SMOKE: shim status + health JSON parses"
json_allow_nonzero "$SUBSTRATE_BIN" --shim-status-json | jq -e '.agent_socket != null' >/dev/null
"$SUBSTRATE_BIN" health --json | jq -e '.summary != null' >/dev/null

if command -v curl >/dev/null 2>&1; then
  SOCKET_PATH="$(
    SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 "$SUBSTRATE_BIN" world doctor --json \
      | jq -r '.world_socket.path // empty'
  )"
  if [[ -n "$SOCKET_PATH" && -S "$SOCKET_PATH" ]]; then
    echo "SMOKE: agent socket probe (${SOCKET_PATH})"
    curl --fail --unix-socket "$SOCKET_PATH" http://localhost/v1/capabilities | jq -e '.' >/dev/null
  else
    echo "SKIP: agent socket probe (no socket at .world_socket.path)"
  fi
fi

if command -v cargo >/dev/null 2>&1 && [[ -f "$ROOT_DIR/Cargo.toml" ]]; then
  echo "SMOKE: targeted regression tests (logging/replay/doctor/health)"
  (
    cd "$ROOT_DIR"
    HOME="$ORIG_HOME" cargo test -p substrate-shell --test logging --test replay_world --test shim_doctor --test shim_health
    HOME="$ORIG_HOME" cargo test -p substrate-replay -- --nocapture
  )
else
  echo "SKIP: cargo tests (cargo not found or not in repo checkout)"
fi

echo "OK: p0 platform stability linux smoke"
