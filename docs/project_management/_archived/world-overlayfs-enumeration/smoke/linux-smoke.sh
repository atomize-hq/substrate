#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: overlayfs enumeration smoke (not Linux)"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
RG_BIN="${RG_BIN:-rg}"
JQ_BIN="${JQ_BIN:-jq}"
MKTEMP_BIN="${MKTEMP_BIN:-mktemp}"

need_cmd() { command -v "$1" > /dev/null 2>&1 || {
  echo "FAIL: missing $1" >&2
  exit 3
}; }
need_cmd "$SUBSTRATE_BIN"
need_cmd "$RG_BIN"
need_cmd "$JQ_BIN"
need_cmd "$MKTEMP_BIN"

assert_jq() {
  local name="$1"
  local input="$2"
  local filter="$3"
  if ! printf '%s' "$input" | "$JQ_BIN" -e "$filter" >/dev/null; then
    echo "FAIL: ${name} (jq assertion failed)" >&2
    printf '%s\n' "$input" >&2
    exit 1
  fi
}

assert_jq_file_slurp() {
  local name="$1"
  local file="$2"
  local filter="$3"
  if ! "$JQ_BIN" -e -s "$filter" "$file" >/dev/null; then
    echo "FAIL: ${name} (jq assertion failed)" >&2
    echo "[FAIL] tail ${file}:" >&2
    tail -n 50 "$file" >&2 || true
    exit 1
  fi
}

tmp="$("$MKTEMP_BIN" -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

$SUBSTRATE_BIN workspace init . > /dev/null

world_doctor="$($SUBSTRATE_BIN world doctor --json)"
assert_jq "world doctor" "$world_doctor" '
  .schema_version == 1 and
  (.world.schema_version | IN(1,2)) and
  .world.world_fs_strategy.primary == "overlay" and
  .world.world_fs_strategy.fallback == "fuse" and
  .world.world_fs_strategy.probe.id == "enumeration_v1" and
  .world.world_fs_strategy.probe.probe_file == ".substrate_enum_probe" and
  (.world.world_fs_strategy.probe.result | IN("pass","fail"))
'

$SUBSTRATE_BIN --world -c 'touch a.txt; ls -a' | $RG_BIN -n -- '^a\.txt$' > /dev/null

trace="$tmp/trace.jsonl"
SHIM_TRACE_LOG="$trace" $SUBSTRATE_BIN --world -c 'touch a.txt; ls -a' > /dev/null
assert_jq_file_slurp "trace contract" "$trace" '
  ([.[] | select(.event_type == "command_complete")] | last) as $e
  | ($e.world_fs_strategy_primary | IN("overlay","fuse"))
  and ($e.world_fs_strategy_final | IN("overlay","fuse","host"))
  and ($e.world_fs_strategy_fallback_reason | IN(
    "none",
    "primary_unavailable",
    "primary_mount_failed",
    "primary_probe_failed",
    "fallback_unavailable",
    "fallback_mount_failed",
    "fallback_probe_failed",
    "world_optional_fallback_to_host"
  ))
'

echo "OK: overlayfs enumeration smoke"
