#!/usr/bin/env bash
set -euo pipefail

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

tmp_root="${SUBSTRATE_SMOKE_ROOT:-}"
if [[ -z "${tmp_root}" ]]; then
  tmp_root="$(mktemp -d)"
fi

cleanup() {
  if [[ "${SUBSTRATE_SMOKE_KEEP:-0}" == "1" ]]; then
    return 0
  fi
  rm -rf "$tmp_root"
}
trap cleanup EXIT

export SUBSTRATE_HOME="${SUBSTRATE_HOME:-$tmp_root/substrate-home}"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

echo "== Setup: workspace + policy patch =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null
"$SUBSTRATE_BIN" policy init --force >/dev/null

expect_exit() {
  local want="$1"
  shift
  local out
  set +e
  out="$("$@" 2>&1)"
  got="$?"
  set -e
  if [[ "$got" -ne "$want" ]]; then
    echo "FAIL: expected exit $want, got $got: $*" >&2
    echo "$out" >&2
    exit 1
  fi
}

echo "== Case 1: legacy keys hard error =="
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.require_world=true'
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.isolation=full'
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.enforcement=strict'

echo "== Case 2: V3 schema accepts host_visible and routing flag =="
expect_exit 0 "$SUBSTRATE_BIN" policy set 'world_fs.host_visible=true'
expect_exit 0 "$SUBSTRATE_BIN" policy set 'world_fs.fail_closed.routing=false'

echo "OK: world-fs-granular-allow-deny-appendix smoke passed (schema-only)"

