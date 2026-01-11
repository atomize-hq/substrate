#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: doctor_scopes smoke (not macOS)"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
JQ_BIN="${JQ_BIN:-jq}"
MKTEMP_BIN="${MKTEMP_BIN:-mktemp}"

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "FAIL: missing $1" >&2; exit 3; }; }
need_cmd "$SUBSTRATE_BIN"
need_cmd "$JQ_BIN"
need_cmd "$MKTEMP_BIN"

tmp="$("$MKTEMP_BIN" -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

$SUBSTRATE_BIN host doctor --json | $JQ_BIN -e '
  .schema_version == 1 and
  .platform == "macos" and
  .world_enabled == true and
  .ok == true and
  .host.platform == "macos" and
  .host.ok == true and
  (.host.world_fs_mode | IN("writable","read_only")) and
  (.host.world_fs_isolation | IN("workspace","full")) and
  .host.lima.installed == true and
  .host.lima.virtualization == true and
  .host.lima.vm_status == "Running" and
  .host.lima.service_active == true and
  .host.lima.agent_caps_ok == true
' >/dev/null

$SUBSTRATE_BIN world doctor --json | $JQ_BIN -e '
  .schema_version == 1 and
  .platform == "macos" and
  .world_enabled == true and
  .ok == true and
  .host.ok == true and
  .world.schema_version == 1 and
  .world.ok == true and
  .world.landlock.supported == true and
  (.world.landlock.abi | type == "number") and
  .world.world_fs_strategy.probe.result == "pass"
' >/dev/null

echo "OK: doctor_scopes smoke (macos)"

