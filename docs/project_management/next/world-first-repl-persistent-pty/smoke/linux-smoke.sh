#!/usr/bin/env bash
set -euo pipefail

FEATURE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-}"

if ! command -v jq >/dev/null 2>&1; then
  echo "ERR: jq is required for smoke scripts" >&2
  exit 3
fi

echo "INFO: feature_dir=$FEATURE_DIR"
echo "INFO: slice_id=${SLICE_ID:-<full>}"

echo "INFO: checking world backend"
substrate world doctor --json | jq -e '.ok == true' >/dev/null
echo "OK: world backend reachable"

run_c0() {
  echo "INFO: C0 — world-agent persistent session bootstrap + preflight (v1)"
  cargo test -p world-agent --test repl_persistent_session_bootstrap_v1 -- --nocapture
  echo "OK: C0"
}

run_c1() {
  echo "INFO: C1 — world-agent exec + command_complete (v1)"
  cargo test -p world-agent --test repl_persistent_session_exec_v1 -- --nocapture
  echo "OK: C1"
}

run_c2() {
  echo "INFO: C2 — shell persistent session client core (v1)"
  cargo test -p shell --test persistent_session_client_v1 -- --nocapture
  echo "OK: C2"
}

run_c3() {
  echo "INFO: C3 — interactive REPL routing + lifecycle (v1)"
  cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
  echo "OK: C3"
}

run_c4() {
  echo "INFO: C4 — interactive REPL byte-safe rendering + buffering (v1)"
  cargo test -p shell --test repl_world_first_rendering_v1 -- --nocapture
  echo "OK: C4"
}

run_c5() {
  echo "INFO: C5 — non-interactive -c and pipe mode world-consistency (v1)"
  cargo test -p shell --test command_mode_world_consistency_v1 -- --nocapture

  rm -rf .wf_world_only_dir || true
  substrate -c 'mkdir -p .wf_world_only_dir'
  if [[ -d .wf_world_only_dir ]]; then
    echo "ERR: expected .wf_world_only_dir to be world-only (not present on host filesystem)" >&2
    exit 1
  fi
  substrate -c 'cd .wf_world_only_dir && pwd -P' >/dev/null
  substrate -c 'rm -rf .wf_world_only_dir' || true
  echo "OK: C5"
}

case "${SLICE_ID}" in
  "" )
    run_c0
    run_c1
    run_c2
    run_c3
    run_c4
    run_c5
    ;;
  "C0" )
    run_c0
    ;;
  "C1" )
    run_c0
    run_c1
    ;;
  "C2" )
    run_c0
    run_c1
    run_c2
    ;;
  "C3" )
    run_c0
    run_c1
    run_c2
    run_c3
    ;;
  "C4" )
    run_c0
    run_c1
    run_c2
    run_c3
    run_c4
    ;;
  "C5" )
    run_c0
    run_c1
    run_c2
    run_c3
    run_c4
    run_c5
    ;;
  * )
    echo "ERR: unsupported SUBSTRATE_SMOKE_SLICE_ID=${SLICE_ID}" >&2
    exit 2
    ;;
esac

echo "OK: linux smoke complete"
