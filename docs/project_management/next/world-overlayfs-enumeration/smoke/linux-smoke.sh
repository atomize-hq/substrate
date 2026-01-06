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

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "FAIL: missing $1" >&2; exit 3; }; }
need_cmd "$SUBSTRATE_BIN"
need_cmd "$RG_BIN"
need_cmd "$JQ_BIN"
need_cmd "$MKTEMP_BIN"

tmp="$("$MKTEMP_BIN" -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

$SUBSTRATE_BIN world doctor --json | $JQ_BIN -e '
  .world_fs_strategy_primary == "overlay" and
  .world_fs_strategy_fallback == "fuse" and
  .world_fs_strategy_probe.id == "enumeration_v1" and
  .world_fs_strategy_probe.probe_file == ".substrate_enum_probe" and
  (.world_fs_strategy_probe.result | IN("pass","fail"))
' >/dev/null

$SUBSTRATE_BIN --world -c 'touch a.txt; ls -a' | $RG_BIN -n -- '^a\.txt$' >/dev/null

trace="$tmp/trace.jsonl"
SHIM_TRACE_LOG="$trace" $SUBSTRATE_BIN --world -c 'touch a.txt; ls -a' >/dev/null
$JQ_BIN -e -s '
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
' "$trace" >/dev/null

echo "OK: overlayfs enumeration smoke"
