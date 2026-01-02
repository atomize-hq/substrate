#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/linux/agent-hub-isolation-verify.sh [--log-dir DIR] [--substrate-bin PATH] [--keep-temp]

Verifies the policy-driven filesystem guarantees introduced by the Agent Hub Isolation Hardening track:
  1) world_fs.mode=read_only blocks both relative and absolute project writes
  2) world_fs.isolation=full prevents reading/writing arbitrary host paths outside the project

This script is intended for manual verification on a host with a working world backend.
On Linux, full-cage requires mount namespaces (root/CAP_SYS_ADMIN or unprivileged user namespaces).

Options:
  --log-dir DIR        Directory to store command output (default: artifacts/linux/agent-hub-isolation-verify-<timestamp>)
  --substrate-bin PATH Use an existing substrate binary (default: target/debug/substrate; auto-build if missing)
  --keep-temp          Keep temporary project directories (default: cleanup at end)
  -h, --help           Show this help message
USAGE
}

LOG_DIR=""
SUBSTRATE_BIN=""
KEEP_TEMP=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --log-dir)
      LOG_DIR="$2"
      shift 2
      ;;
    --substrate-bin)
      SUBSTRATE_BIN="$2"
      shift 2
      ;;
    --keep-temp)
      KEEP_TEMP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to inspect doctor output." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build the substrate CLI." >&2
  exit 1
fi

SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${SCRIPT_ROOT}/.." && pwd)"

ARTIFACT_ROOT="${REPO_ROOT}/artifacts/linux"
mkdir -p "${ARTIFACT_ROOT}"
if [[ -z "${LOG_DIR}" ]]; then
  LOG_DIR="${ARTIFACT_ROOT}/agent-hub-isolation-verify-$(date -u '+%Y%m%d-%H%M%S')"
fi
mkdir -p "${LOG_DIR}"

log() {
  printf '[agent-hub-isolation-verify] %s\n' "$*"
}

run() {
  log "$*"
  "$@"
}

if [[ -z "${SUBSTRATE_BIN}" ]]; then
  SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
fi

if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  log "Building substrate CLI (debug profile)"
  run cargo build -p substrate --bin substrate
fi

DOCTOR_JSON="${LOG_DIR}/world-doctor.json"
log "Running substrate world doctor --json"
run "${SUBSTRATE_BIN}" world doctor --json >"${DOCTOR_JSON}"

if ! jq -e '.ok == true' "${DOCTOR_JSON}" >/dev/null 2>&1; then
  log "World doctor reported ok=false; isolation verification requires a working world backend."
  log "See: ${DOCTOR_JSON}"
  jq '{ok, world_fs_mode, world_socket, agent_socket, lima: (.agent_socket.lima // null)}' "${DOCTOR_JSON}" \
    | tee "${LOG_DIR}/world-doctor-summary.json" >/dev/null || true
  cat <<'HINT' >&2
Hints:
  - Linux: run scripts/linux/world-provision.sh (installs systemd socket/service)
  - macOS: run scripts/mac/lima-warm.sh (provisions the Lima VM backend)
  - Re-run doctor: substrate world doctor --json
HINT
  exit 1
fi

HOST_HOME="$(cd "${HOME}" && pwd)"

readonly_project="${LOG_DIR}/readonly-project"
fullcage_project="${LOG_DIR}/fullcage-project"
mkdir -p "${readonly_project}" "${fullcage_project}"

readonly_substrate="${readonly_project}/.substrate"
fullcage_substrate="${fullcage_project}/.substrate"
mkdir -p "${readonly_substrate}" "${fullcage_substrate}"

cat >"${readonly_substrate}/workspace.yaml" <<'YAML'
world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: true
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
YAML

cat >"${readonly_substrate}/policy.yaml" <<'YAML'
id: i5-verify-readonly
name: I5 verify (read_only)
world_fs:
  mode: read_only
  isolation: workspace
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
YAML

cat >"${fullcage_substrate}/workspace.yaml" <<'YAML'
world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: true
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
YAML

cat >"${fullcage_substrate}/policy.yaml" <<'YAML'
id: i5-verify-full-cage
name: I5 verify (full cage)
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
YAML

cleanup() {
  if [[ ${KEEP_TEMP} -eq 1 ]]; then
    log "Keeping temp projects: ${readonly_project}, ${fullcage_project}"
    return 0
  fi
  rm -rf "${readonly_project}" "${fullcage_project}" 2>/dev/null || true
  rm -f "${outside_read:-}" "${outside_write:-}" 2>/dev/null || true
}
trap cleanup EXIT

expect_failure() {
  local label="$1"
  local out="$2"
  local err="$3"
  shift 3
  if "$@" >"${out}" 2>"${err}"; then
    log "FAILED: expected failure (${label}) but command succeeded"
    log "stdout: ${out}"
    log "stderr: ${err}"
    exit 1
  fi
}

expect_success() {
  local label="$1"
  local out="$2"
  local err="$3"
  shift 3
  if ! "$@" >"${out}" 2>"${err}"; then
    log "FAILED: expected success (${label}) but command failed"
    log "stdout: ${out}"
    log "stderr: ${err}"
    exit 1
  fi
}

log "Test 1: world_fs.mode=read_only blocks project writes (relative + absolute)"
pushd "${readonly_project}" >/dev/null
expect_failure \
  "read_only relative write" \
  "${LOG_DIR}/readonly-rel.stdout" \
  "${LOG_DIR}/readonly-rel.stderr" \
  "${SUBSTRATE_BIN}" --ci -c 'echo deny > rel.txt'
if [[ -e "${readonly_project}/rel.txt" ]]; then
  log "FAILED: rel.txt was created on host (read_only should prevent project writes)"
  exit 1
fi

expect_failure \
  "read_only absolute write" \
  "${LOG_DIR}/readonly-abs.stdout" \
  "${LOG_DIR}/readonly-abs.stderr" \
  env I5_ABS_WRITE="${readonly_project}/abs.txt" \
  "${SUBSTRATE_BIN}" --ci -c 'echo deny > "$I5_ABS_WRITE"'
if [[ -e "${readonly_project}/abs.txt" ]]; then
  log "FAILED: abs.txt was created on host (read_only should prevent absolute-path project writes)"
  exit 1
fi
popd >/dev/null
log "PASS: read_only blocked relative + absolute project writes"

log "Test 2: world_fs.isolation=full blocks arbitrary host paths outside the project"
outside_read="${HOST_HOME}/substrate-i5-outside-read.$$.txt"
outside_write="${HOST_HOME}/substrate-i5-outside-write.$$.txt"
printf 'host-visible\n' >"${outside_read}"

pushd "${fullcage_project}" >/dev/null
expect_success \
  "full_cage host path isolation checks" \
  "${LOG_DIR}/fullcage.stdout" \
  "${LOG_DIR}/fullcage.stderr" \
  env I5_OUTSIDE_READ="${outside_read}" I5_OUTSIDE_WRITE="${outside_write}" \
  "${SUBSTRATE_BIN}" --ci -c '
set -eu

test -r /etc/passwd

mkdir -p writable
echo ok > writable/ok.txt
test -s writable/ok.txt

if echo nope > not-allowlisted.txt; then
  echo "unexpected: wrote to non-allowlisted project path" >&2
  exit 1
fi

if cat "$I5_OUTSIDE_READ" >/dev/null 2>&1; then
  echo "unexpected: could read host path outside project ($I5_OUTSIDE_READ)" >&2
  exit 1
fi

if echo deny > "$I5_OUTSIDE_WRITE" 2>/dev/null; then
  echo "unexpected: could write host path outside project ($I5_OUTSIDE_WRITE)" >&2
  exit 1
fi
'
popd >/dev/null

rm -f "${outside_read}" "${outside_write}" 2>/dev/null || true
log "PASS: full cage blocked host paths outside the project (and enforced project write allowlist)"

log "Verification complete. Logs saved under: ${LOG_DIR}"
cat <<'SUMMARY'
Expected failure modes:
  - If full-cage fails with an unshare/mount-namespace error, the host/guest needs CAP_SYS_ADMIN
    (root) or unprivileged user namespaces enabled (kernel.unprivileged_userns_clone=1 on Linux).
  - If doctor reports ok=false, provision/repair the world backend and re-run this script.
SUMMARY
