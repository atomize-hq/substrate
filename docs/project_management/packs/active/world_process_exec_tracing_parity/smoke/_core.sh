#!/usr/bin/env bash
set -euo pipefail

# Shared behavior smoke for world_process_exec_tracing_parity.
#
# Caller responsibilities:
# - Ensure `substrate` is available via `$SUBSTRATE_BIN` (default: `substrate`).
# - Ensure the world backend is healthy (`substrate world doctor`).
#
# Exit codes (aligned to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed
# - 1: smoke assertion failed / unexpected script error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)
# - 3: required dependency unavailable (e.g., substrate not found)
# - 4: not supported / missing prerequisites (e.g., world backend not healthy)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world_process_exec_tracing_parity: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

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

workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

trace="$tmp_root/trace.jsonl"
rm -f "$trace"
export SHIM_TRACE_LOG="$trace"

echo "== Setup: workspace =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "world_process_exec_tracing_parity: world backend not healthy; run 'substrate world enable' and retry" >&2
  exit 4
fi

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-WPEP3}"
case "$slice_id" in
  WPEP0|WPEP1|WPEP2|WPEP3) ;;
  *)
    echo "world_process_exec_tracing_parity: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice_id" >&2
    exit 2
    ;;
esac

platform=""
case "${OSTYPE:-}" in
  linux*) platform="linux" ;;
  darwin*) platform="macos" ;;
  *)
    echo "world_process_exec_tracing_parity: unsupported OSTYPE=${OSTYPE:-unknown}" >&2
    exit 4
    ;;
esac

expect_jq() {
  jq -s -e "$@" "$trace" >/dev/null
}

echo "== Case A: shell span_id joins to the command_complete record =="
marker="WPEP_SMOKE_MARKER_JOIN_${slice_id}_$RANDOM"
"$SUBSTRATE_BIN" --command "echo $marker" >/dev/null

expect_jq --arg m "$marker" '
  any(select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains($m))) | has("span_id"))
'

echo "== Case B: wrap-mode builtin trace omits command bodies (linux/macos only) =="
SUBSTRATE_OVERRIDE_WORLD=disabled "$SUBSTRATE_BIN" --command 'export SUBSTRATE_SMOKE_WRAP=1' >/dev/null

expect_jq '
  any(select(.event_type=="builtin_command" and .mode=="wrap") | (.command_omitted==true))
'
expect_jq '
  any(select(.event_type=="builtin_command") | (.command_omitted==true))
'
expect_jq '
  all(select(.event_type=="builtin_command") | (.command_omitted==true))
'
expect_jq '
  (any(select(.event_type=="builtin_command") | has("command"))) | not
'
expect_jq '
  (any(select(.event_type=="builtin_command_raw"))) | not
'

if [[ "$slice_id" == "WPEP0" ]]; then
  echo "OK: world_process_exec_tracing_parity smoke passed (slice=$slice_id platform=$platform)"
  exit 0
fi

echo "== Case C: world completion record carries the published process_events_status posture =="
world_marker="WPEP_SMOKE_MARKER_WORLD_${slice_id}_$RANDOM"
"$SUBSTRATE_BIN" --world --command "bash -lc \"echo $world_marker; sh -lc true; echo done\"" >/dev/null

status_expected=""
reason_expected=""
case "$platform:$slice_id" in
  linux:WPEP1)
    status_expected="unavailable"
    reason_expected="backend_disabled"
    ;;
  linux:WPEP2)
    status_expected="ok|truncated"
    ;;
  linux:WPEP3)
    status_expected="ok|truncated"
    ;;
  macos:WPEP1)
    status_expected="unavailable"
    reason_expected="backend_disabled"
    ;;
  macos:WPEP2)
    status_expected="ok|truncated"
    ;;
  macos:WPEP3)
    status_expected="ok|truncated"
    ;;
esac

if [[ "$status_expected" == "ok|truncated" ]]; then
  expect_jq --arg m "$world_marker" '
    any(
      select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains($m))) |
      (.process_events_status=="ok" or .process_events_status=="truncated")
    )
  '
else
  expect_jq --arg m "$world_marker" --arg st "$status_expected" --arg rs "$reason_expected" '
    any(
      select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains($m))) |
      (.process_events_status==$st and .process_events_reason==$rs)
    )
  '
fi

if [[ "$slice_id" == "WPEP1" ]]; then
  echo "OK: world_process_exec_tracing_parity smoke passed (slice=$slice_id platform=$platform)"
  exit 0
fi

if [[ "$platform" == "linux" ]]; then
  echo "== Case D: linux-backed world_process_* events are joinable by parent_span =="
  span_id="$(
    jq -r -s --arg m "$world_marker" '
      .[] | select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains($m))) | .span_id
    ' "$trace" | tail -n 1
  )"
  test -n "$span_id"

  expect_jq --arg sp "$span_id" '
    any(select(.component=="world-agent" and .event_type=="world_process_start" and .parent_span==$sp))
  '
  expect_jq --arg sp "$span_id" '
    any(select(.component=="world-agent" and .event_type=="world_process_exit" and .parent_span==$sp))
  '

  if [[ "$slice_id" == "WPEP2" ]]; then
    echo "== Case E: WPEP2 publishes argv_omitted = true =="
    expect_jq --arg sp "$span_id" '
      all(select(.component=="world-agent" and (.event_type=="world_process_start" or .event_type=="world_process_exit") and .parent_span==$sp) | (.argv_omitted==true))
    '
  fi

  if [[ "$slice_id" == "WPEP3" ]]; then
    echo "== Case E: WPEP3 publishes argv with redaction =="
    expect_jq --arg sp "$span_id" '
      any(select(.component=="world-agent" and .event_type=="world_process_start" and .parent_span==$sp) | (has("argv") and (.argv|type=="array")))
    '
    expect_jq --arg sp "$span_id" '
      (any(select(.component=="world-agent" and (.event_type=="world_process_start" or .event_type=="world_process_exit") and .parent_span==$sp) | has("argv_omitted"))) | not
    '
  fi
fi

echo "OK: world_process_exec_tracing_parity smoke passed (slice=$slice_id platform=$platform)"
