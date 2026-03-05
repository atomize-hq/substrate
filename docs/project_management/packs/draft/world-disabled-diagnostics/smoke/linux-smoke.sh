#!/usr/bin/env bash
set -euo pipefail

# Exit codes:
# - 0: OK / SKIP
# - 1: assertion failed / unexpected error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: world-disabled-diagnostics linux smoke (not Linux)"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "FAIL: substrate not found (set SUBSTRATE_BIN=/path/to/substrate)" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "FAIL: python3 not found (required for JSON assertions)" >&2
  exit 1
fi

SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-WDD2}"
case "$SLICE_ID" in
  WDD0|WDD1|WDD2) ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$SLICE_ID (expected WDD0, WDD1, or WDD2)" >&2
    exit 2
    ;;
esac

WORKDIR="$(mktemp -d)"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT
cd "$WORKDIR"

require_contains() {
  local haystack="$1"
  local needle="$2"
  printf '%s\n' "$haystack" | grep -Fq "$needle" || {
    echo "FAIL: missing expected line: $needle" >&2
    return 1
  }
}

require_not_contains() {
  local haystack="$1"
  local needle="$2"
  if printf '%s\n' "$haystack" | grep -Fq "$needle"; then
    echo "FAIL: found forbidden substring: $needle" >&2
    return 1
  fi
}

run_json_capture() {
  local label="$1"
  shift

  local stdout_file stderr_file
  stdout_file="$(mktemp)"
  stderr_file="$(mktemp)"

  set +e
  "$@" 1>"$stdout_file" 2>"$stderr_file"
  local rc=$?
  set -e

  if [[ $rc -ne 0 ]]; then
    echo "FAIL: $label (exit=$rc)" >&2
    cat "$stderr_file" >&2 || true
    cat "$stdout_file" >&2 || true
    return "$rc"
  fi

  local json
  json="$(cat "$stdout_file")"
  rm -f "$stdout_file" "$stderr_file"
  printf '%s' "$json"
}

check_wdd0_config_resolution_error() {
  local home
  home="$(mktemp -d)"
  printf 'world: [\n' > "$home/config.yaml"

  local out rc

  set +e
  out="$(SUBSTRATE_HOME="$home" "$SUBSTRATE_BIN" shim doctor 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 2 ]] || { echo "FAIL: shim doctor invalid config expected exit=2, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }
  printf '%s\n' "$out" | grep -Fq "config.yaml" || { echo "FAIL: shim doctor invalid config stderr must mention config.yaml" >&2; printf '%s\n' "$out" >&2; return 1; }

  set +e
  out="$(SUBSTRATE_HOME="$home" "$SUBSTRATE_BIN" shim doctor --json 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 2 ]] || { echo "FAIL: shim doctor --json invalid config expected exit=2, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }
  printf '%s\n' "$out" | grep -Fq "config.yaml" || { echo "FAIL: shim doctor --json invalid config stderr must mention config.yaml" >&2; printf '%s\n' "$out" >&2; return 1; }

  set +e
  out="$(SUBSTRATE_HOME="$home" "$SUBSTRATE_BIN" health 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 2 ]] || { echo "FAIL: health invalid config expected exit=2, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }
  printf '%s\n' "$out" | grep -Fq "config.yaml" || { echo "FAIL: health invalid config stderr must mention config.yaml" >&2; printf '%s\n' "$out" >&2; return 1; }

  set +e
  out="$(SUBSTRATE_HOME="$home" "$SUBSTRATE_BIN" health --json 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 2 ]] || { echo "FAIL: health --json invalid config expected exit=2, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }
  printf '%s\n' "$out" | grep -Fq "config.yaml" || { echo "FAIL: health --json invalid config stderr must mention config.yaml" >&2; printf '%s\n' "$out" >&2; return 1; }

  rm -rf "$home"
}

check_wdd1_shim_doctor_disabled_and_broken() {
  local home
  home="$(mktemp -d)"

  local out rc
  set +e
  out="$(SUBSTRATE_HOME="$home" SUBSTRATE_OVERRIDE_WORLD=disabled "$SUBSTRATE_BIN" shim doctor 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 0 ]] || { echo "FAIL: shim doctor disabled expected exit=0, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }

  require_contains "$out" "World backend:"
  require_contains "$out" "  Status: disabled"
  require_contains "$out" "  Next: run \`substrate world enable\` to provision"
  require_contains "$out" "World deps:"
  require_contains "$out" "  Status: skipped (world disabled)"

  require_not_contains "$out" "  Error:"

  local json
  json="$(run_json_capture "shim doctor --json (disabled)" env SUBSTRATE_HOME="$home" SUBSTRATE_OVERRIDE_WORLD=disabled "$SUBSTRATE_BIN" shim doctor --json)"
  local json_file
  json_file="$(mktemp)"
  printf '%s' "$json" > "$json_file"
  python3 - "$json_file" <<'PY'
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as f:
    d = json.load(f)
assert d["world"]["status"] == "disabled"
assert d["world_deps"]["status"] == "skipped_disabled"
for k in ("error", "stderr", "exit_code", "details"):
    assert k not in d["world"], f"world.{k} must be omitted"
for k in ("error", "report"):
    assert k not in d["world_deps"], f"world_deps.{k} must be omitted"
PY
  rm -f "$json_file"

  # Force-enabled, intentionally broken connectivity via nonexistent world socket.
  local sock
  sock="$home/does-not-exist.sock"
  rm -f "$sock" || true

  set +e
  out="$(SUBSTRATE_HOME="$home" SUBSTRATE_WORLD_SOCKET="$sock" "$SUBSTRATE_BIN" --world shim doctor 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 0 ]] || { echo "FAIL: shim doctor enabled-but-broken expected exit=0, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }

  require_contains "$out" "World backend:"
  require_contains "$out" "  Status: needs attention"
  require_contains "$out" "  Error:"
  require_not_contains "$out" "  Status: disabled"

  json="$(run_json_capture "shim doctor --json (broken)" env SUBSTRATE_HOME="$home" SUBSTRATE_WORLD_SOCKET="$sock" "$SUBSTRATE_BIN" --world shim doctor --json)"
  json_file="$(mktemp)"
  printf '%s' "$json" > "$json_file"
  python3 - "$json_file" <<'PY'
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as f:
    d = json.load(f)
assert d["world"]["status"] == "needs_attention"
assert isinstance(d["world"].get("error"), str) and d["world"]["error"].strip()
assert d["world_deps"]["status"] == "error"
assert isinstance(d["world_deps"].get("error"), str) and d["world_deps"]["error"].strip()
PY
  rm -f "$json_file"

  rm -rf "$home"
}

check_wdd2_health_disabled_and_broken() {
  local home
  home="$(mktemp -d)"

  local out rc
  set +e
  out="$(SUBSTRATE_HOME="$home" SUBSTRATE_OVERRIDE_WORLD=disabled "$SUBSTRATE_BIN" health 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 0 ]] || { echo "FAIL: health disabled expected exit=0, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }

  require_contains "$out" "World backend: disabled"
  require_contains "$out" "  Next: run \`substrate world enable\` to provision"
  require_contains "$out" "World deps: skipped (world disabled)"
  require_not_contains "$out" "substrate world deps current"

  local json
  json="$(run_json_capture "health --json (disabled)" env SUBSTRATE_HOME="$home" SUBSTRATE_OVERRIDE_WORLD=disabled "$SUBSTRATE_BIN" health --json)"
  local json_file
  json_file="$(mktemp)"
  printf '%s' "$json" > "$json_file"
  python3 - "$json_file" <<'PY'
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as f:
    h = json.load(f)
assert h["shim"]["world"]["status"] == "disabled"
assert h["shim"]["world_deps"]["status"] == "skipped_disabled"
summary = h["summary"]
assert summary["world_ok"] is None
assert "world_error" not in summary
assert "world_deps_error" not in summary
assert summary["world_deps_missing"] == []
assert summary["world_deps_blocked"] == []
PY
  rm -f "$json_file"

  local sock
  sock="$home/does-not-exist.sock"
  rm -f "$sock" || true

  set +e
  out="$(SUBSTRATE_HOME="$home" SUBSTRATE_WORLD_SOCKET="$sock" "$SUBSTRATE_BIN" --world health 2>&1)"
  rc=$?
  set -e
  [[ $rc -eq 0 ]] || { echo "FAIL: health enabled-but-broken expected exit=0, got=$rc" >&2; printf '%s\n' "$out" >&2; return 1; }

  require_contains "$out" "World backend: needs attention"
  require_contains "$out" "  Error:"
  require_contains "$out" "World deps: unavailable"
  require_contains "$out" "Overall status: attention required"
  require_not_contains "$out" "World backend: disabled"

  json="$(run_json_capture "health --json (broken)" env SUBSTRATE_HOME="$home" SUBSTRATE_WORLD_SOCKET="$sock" "$SUBSTRATE_BIN" --world health --json)"
  json_file="$(mktemp)"
  printf '%s' "$json" > "$json_file"
  python3 - "$json_file" <<'PY'
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as f:
    h = json.load(f)
assert h["shim"]["world"]["status"] == "needs_attention"
assert h["summary"]["world_ok"] is False
assert isinstance(h["summary"].get("world_error"), str) and h["summary"]["world_error"].strip()
assert h["shim"]["world_deps"]["status"] == "error"
assert isinstance(h["summary"].get("world_deps_error"), str) and h["summary"]["world_deps_error"].strip()
PY
  rm -f "$json_file"

  rm -rf "$home"
}

echo "INFO: world-disabled-diagnostics linux smoke slice=$SLICE_ID"

check_wdd0_config_resolution_error
if [[ "$SLICE_ID" == "WDD0" ]]; then
  echo "OK: world-disabled-diagnostics linux smoke ($SLICE_ID)"
  exit 0
fi

check_wdd1_shim_doctor_disabled_and_broken
if [[ "$SLICE_ID" == "WDD1" ]]; then
  echo "OK: world-disabled-diagnostics linux smoke ($SLICE_ID)"
  exit 0
fi

check_wdd2_health_disabled_and_broken
echo "OK: world-disabled-diagnostics linux smoke ($SLICE_ID)"
