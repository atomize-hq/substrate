#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -eq 0 ]]; then
  echo "Do not run this orchestration smoke script as root." >&2
  exit 1
fi

SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"

log() {
  printf '[mac-orchestration-smoke] %s\n' "$*"
}

require_cmd() {
  local cmd="$1"
  local hint="${2:-}"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    if [[ -n "${hint}" ]]; then
      echo "ERROR: ${cmd} not found on PATH. ${hint}" >&2
    else
      echo "ERROR: ${cmd} not found on PATH." >&2
    fi
    exit 1
  fi
}

ensure_host_prereqs() {
  if ! command -v limactl >/dev/null 2>&1; then
    PATH="/opt/homebrew/opt/lima/bin:/opt/homebrew/bin:$PATH"
  fi

  require_cmd limactl "Install Lima via Homebrew (brew install lima)."
  require_cmd cargo "Install Rust via rustup."
}

run_repo_cmd() {
  log "$*"
  (
    cd "${REPO_ROOT}"
    "$@"
  )
}

ensure_host_prereqs

log "Warming the Lima-backed world backend"
"${SCRIPTS_ROOT}/lima-warm.sh"

log "Running live Lima backend reachability smoke"
run_repo_cmd cargo run -p world-mac-lima --example mac_backend_smoke

log "Running macOS shared-owner bootstrap routing proof"
run_repo_cmd cargo test -p shell shared_owner_macos_allows_lima_backed_bootstrap -- --nocapture

log "Running shared-owner ready-proof acceptance and mismatch rejection checks"
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_accepts_shared_world_attach_create_ready_proof -- --nocapture
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_accepts_replacement_ready_when_generation_advances -- --nocapture
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_rejects_invalid_shared_world_ready_proof -- --nocapture

log "Running retained world-member launch, reuse, and cancel checks"
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_first_targeted_world_turn_uses_initial_prompt_in_member_dispatch -- --nocapture
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_first_world_backed_command_lazily_launches_member_runtime -- --nocapture
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_targeted_world_turn_uses_typed_submit_route_without_relaunching_member -- --nocapture

log "macOS/Lima orchestration smoke passed"
