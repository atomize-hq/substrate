#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: doctor_scopes smoke (not Linux)"
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

# Require world-enabled behavior for this feature’s Linux smoke.
$SUBSTRATE_BIN host doctor --json | $JQ_BIN -e '
  .schema_version == 1 and
  .platform == "linux" and
  .world_enabled == true and
  .ok == true and
  .host.platform == "linux" and
  .host.ok == true and
  (.host.world_fs_mode | IN("writable","read_only")) and
  (.host.world_fs_isolation | IN("workspace","full")) and
  (.host.world_socket.socket_path | type == "string") and
  (.host.world_socket.socket_exists == true) and
  (.host.world_socket.probe_ok == true)
' >/dev/null

$SUBSTRATE_BIN world doctor --json | $JQ_BIN -e '
  .schema_version == 1 and
  .platform == "linux" and
  .world_enabled == true and
  .ok == true and
  .host.ok == true and
  .world.schema_version == 1 and
  .world.ok == true and
  .world.landlock.supported == true and
  (.world.landlock.abi | type == "number") and
  .world.world_fs_strategy.primary == "overlay" and
  .world.world_fs_strategy.fallback == "fuse" and
  .world.world_fs_strategy.probe.id == "enumeration_v1" and
  .world.world_fs_strategy.probe.probe_file == ".substrate_enum_probe" and
  .world.world_fs_strategy.probe.result == "pass"
' >/dev/null

echo "OK: doctor_scopes smoke (linux)"

