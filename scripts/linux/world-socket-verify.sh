#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/linux/world-socket-verify.sh [--profile PROFILE] [--log-dir DIR] [--skip-cleanup]

Provision the systemd-managed substrate world-agent socket via scripts/linux/world-provision.sh,
run `substrate world doctor --json` to capture the `world_socket` block, run
`substrate --shim-status-json`, and optionally uninstall the units afterward. This script requires
sudo privileges and will write logs/artifacts under the specified log directory (defaults to
artifacts/linux/world-socket-verify-<timestamp>).

Options:
  --profile PROFILE   Cargo/systemd profile to pass to world-provision.sh (default: release)
  --log-dir DIR       Directory to store command output (default: artifacts/linux/world-socket-verify-<timestamp>)
  --skip-cleanup      Leave the provisioned services/socket in place (default: cleanup runs uninstall script)
  -h, --help          Show this help message
USAGE
}

PROFILE="release"
CLEANUP=1
LOG_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --log-dir)
      LOG_DIR="$2"
      shift 2
      ;;
    --skip-cleanup)
      CLEANUP=0
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

if [[ $(uname -s) != "Linux" ]]; then
  echo "This verification script must run on Linux." >&2
  exit 1
fi

if ! command -v sudo >/dev/null 2>&1; then
  echo "sudo is required to run provisioning commands." >&2
  exit 1
fi

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
  LOG_DIR="${ARTIFACT_ROOT}/world-socket-verify-$(date -u '+%Y%m%d-%H%M%S')"
fi
mkdir -p "${LOG_DIR}"

log() {
  printf '[world-socket-verify] %s\n' "$*"
}

run() {
  log "$*"
  "$@"
}

log "Using log directory: ${LOG_DIR}"
log "Provisioning profile: ${PROFILE}"

SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  log "Building substrate CLI (debug profile)"
  run cargo build -p substrate --bin substrate
fi

SYSTEMCTL_SOCKET_LOG="${LOG_DIR}/systemctl-socket.txt"
SYSTEMCTL_SERVICE_LOG="${LOG_DIR}/systemctl-service.txt"
DOCTOR_JSON="${LOG_DIR}/world-doctor.json"
DOCTOR_SOCKET_JSON="${LOG_DIR}/world-doctor-world_socket.json"
SHIM_STATUS_JSON="${LOG_DIR}/shim-status.json"

log "Running Linux world provisioner (requires sudo)"
run "${REPO_ROOT}/scripts/linux/world-provision.sh" --profile "${PROFILE}"

log "Capturing systemctl status for socket/service"
run sudo systemctl status substrate-world-agent.socket --no-pager --lines=20 >"${SYSTEMCTL_SOCKET_LOG}"
run sudo systemctl status substrate-world-agent.service --no-pager --lines=20 >"${SYSTEMCTL_SERVICE_LOG}"

log "Running substrate world doctor --json"
if ! run "${SUBSTRATE_BIN}" world doctor --json >"${DOCTOR_JSON}"; then
  echo "substrate world doctor failed; see ${DOCTOR_JSON}" >&2
  exit 1
fi
log "Extracting world_socket block"
jq '.world_socket // .agent_socket' "${DOCTOR_JSON}" >"${DOCTOR_SOCKET_JSON}"

log "Running substrate --shim-status-json"
run "${SUBSTRATE_BIN}" --shim-status-json >"${SHIM_STATUS_JSON}"

if [[ ${CLEANUP} -eq 1 ]]; then
  log "Cleaning up via uninstall-substrate.sh"
  run "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh"
else
  log "Skipping cleanup; services remain installed"
fi

log "Verification complete. Artifacts saved under ${LOG_DIR}:"
log "- systemctl socket log: ${SYSTEMCTL_SOCKET_LOG}"
log "- systemctl service log: ${SYSTEMCTL_SERVICE_LOG}"
log "- world doctor JSON: ${DOCTOR_JSON}"
log "- extracted world_socket/agent_socket: ${DOCTOR_SOCKET_JSON}"
log "- shim status JSON: ${SHIM_STATUS_JSON}"

cat <<SUMMARY
Next steps:
  * Review ${DOCTOR_SOCKET_JSON} to ensure the expected socket_activation mode/path are recorded.
  * Attach the logs above to the session log or PR as proof of verification.
SUMMARY
