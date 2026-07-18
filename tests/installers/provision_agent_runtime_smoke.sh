#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

log() {
  printf '[provision-agent-runtime-smoke] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local context="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    fatal "${context}: expected to find '${needle}'"
  fi
}

assert_file_contains() {
  local file="$1"
  local needle="$2"
  local context="$3"
  if ! grep -Fq -- "${needle}" "${file}"; then
    fatal "${context}: expected ${file} to contain '${needle}'"
  fi
}

assert_in_order() {
  local file="$1"
  local first="$2"
  local second="$3"
  local context="$4"
  python3 - <<'PY' "${file}" "${first}" "${second}" "${context}"
import pathlib, sys

path = pathlib.Path(sys.argv[1])
first = sys.argv[2]
second = sys.argv[3]
context = sys.argv[4]
text = path.read_text()
first_idx = text.find(first)
second_idx = text.find(second)
if first_idx == -1 or second_idx == -1 or first_idx >= second_idx:
    raise SystemExit(f"{context}: expected '{first}' before '{second}' in {path}")
PY
}

write_stub_substrate() {
  local path="$1"
  cat >"${path}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

log_path="${SUBSTRATE_STUB_LOG:?}"
printf '%s\n' "$*" >>"${log_path}"
command_args=("$@")
expected_carrier="${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}"
if [[ -n "${expected_carrier}" ]]; then
  [[ "${command_args[0]:-}" == "--install-bootstrap-context-v1" ]] || exit 91
  [[ "${command_args[1]:-}" == "${expected_carrier}" ]] || exit 92
  command_args=("${command_args[@]:2}")
elif [[ "${command_args[0]:-}" == "--install-bootstrap-context-v1" ]]; then
  exit 93
fi
command="${command_args[*]}"

if [[ "${command}" == "world deps global add --json codex-runtime" ]]; then
  case "${STUB_ADD_MODE:-added}" in
    added)
      printf '{"changed":["codex-runtime"]}\n'
      ;;
    existing)
      printf '{"unchanged":["codex-runtime"]}\n'
      ;;
    *)
      printf '{"status":"ok"}\n'
      ;;
  esac
  exit 0
fi

if [[ "${command}" == "world deps current sync" ]]; then
  exit "${STUB_SYNC_EXIT:-0}"
fi

if [[ "${command}" == "world deps global remove codex-runtime" ]]; then
  exit "${STUB_REMOVE_EXIT:-0}"
fi

if [[ "${command}" == "world deps current list applied --json" ]]; then
  printf '{"items":[]}\n'
  exit 0
fi

exit 0
EOF
  chmod +x "${path}"
}

source_dev_runtime_provision_helpers() {
  # shellcheck disable=SC1090
  source <(awk '/^run_privileged\(\)/{exit} {print}' "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh")
}

run_prod_install_then_sync_scenario() {
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-provision-runtime-prod.XXXXXX")"
  trap 'rm -rf "${work_root}"' RETURN

  local stub="${work_root}/substrate"
  local stub_log="${work_root}/substrate.log"
  write_stub_substrate "${stub}"
  : >"${stub_log}"

  (
    export SUBSTRATE_STUB_LOG="${stub_log}"
    export STUB_ADD_MODE="added"
    export STUB_SYNC_EXIT=0
    source "${REPO_ROOT}/scripts/substrate/install-substrate.sh"
    PROVISION_AGENT_RUNTIME="codex"
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
    SYNC_DEPS=1
    NO_WORLD=0
    DRY_RUN=0
    PREFIX="${work_root}/prefix"
    ORIGINAL_PATH="${PATH}"
    resolve_install_bootstrap_context 1 "${PREFIX}" "" 0
    provision_agent_runtime_world_deps "${stub}"
    sync_world_deps "${stub}"
  )

  assert_file_contains "${stub_log}" "world deps global add --json codex-runtime" "prod install should enable the runtime item"
  assert_file_contains "${stub_log}" "world deps current sync" "prod install should run sync immediately after enable"
  assert_in_order "${stub_log}" "world deps global add --json codex-runtime" "world deps current sync" "prod install should enable before sync"
  log "Verified prod install-time runtime provisioning performs add-then-sync."
}

run_prod_rollback_remediation_scenario() {
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-provision-runtime-prod-rollback.XXXXXX")"
  trap 'rm -rf "${work_root}"' RETURN

  local stub="${work_root}/substrate"
  local stub_log="${work_root}/substrate.log"
  write_stub_substrate "${stub}"
  : >"${stub_log}"

  local output
  set +e
  output="$(
    exec 2>&1
    export SUBSTRATE_STUB_LOG="${stub_log}"
    export STUB_ADD_MODE="added"
    export STUB_SYNC_EXIT=7
    export STUB_REMOVE_EXIT=0
    source "${REPO_ROOT}/scripts/substrate/install-substrate.sh"
    PROVISION_AGENT_RUNTIME="codex"
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
    SYNC_DEPS=1
    NO_WORLD=0
    DRY_RUN=0
    PREFIX="${work_root}/prefix"
    ORIGINAL_PATH="${PATH}"
    resolve_install_bootstrap_context 1 "${PREFIX}" "" 0
    provision_agent_runtime_world_deps "${stub}"
    sync_world_deps "${stub}"
  )"
  local status=$?
  set -e

  [[ "${status}" -eq 7 ]] || fatal "prod rollback scenario should exit 7, got ${status}"
  assert_contains "${output}" "the installer removed the global enable" "prod rollback remediation"
  assert_contains "${output}" "re-run the installer with '--provision-agent-runtime codex' to re-add 'codex-runtime' and retry the sync" "prod rollback remediation"
  assert_file_contains "${stub_log}" "world deps global remove codex-runtime" "prod rollback should remove the newly-added enable"
  log "Verified prod rollback remediation explains how to re-add and retry."
}

run_world_enable_helper_rollback_remediation_scenario() {
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-provision-runtime-helper-rollback.XXXXXX")"
  trap 'rm -rf "${work_root}"' RETURN

  local stub="${work_root}/substrate"
  local stub_log="${work_root}/substrate.log"
  write_stub_substrate "${stub}"
  : >"${stub_log}"

  local output
  set +e
  output="$(
    exec 2>&1
    export SUBSTRATE_STUB_LOG="${stub_log}"
    export STUB_ADD_MODE="added"
    export STUB_SYNC_EXIT=7
    export STUB_REMOVE_EXIT=0
    INSTALLER_NAME="substrate-world-enable"
    source "${REPO_ROOT}/scripts/substrate/install-substrate.sh"
    PROVISION_AGENT_RUNTIME="codex"
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
    SYNC_DEPS=1
    NO_WORLD=0
    DRY_RUN=0
    PREFIX="${work_root}/prefix"
    ORIGINAL_PATH="${PATH}"
    resolve_install_bootstrap_context 1 "${PREFIX}" "" 0
    provision_agent_runtime_world_deps "${stub}"
    sync_world_deps "${stub}"
  )"
  local status=$?
  set -e

  [[ "${status}" -eq 7 ]] || fatal "world-enable rollback scenario should exit 7, got ${status}"
  assert_contains "${output}" "the installer removed the global enable" "world-enable rollback remediation"
  assert_contains "${output}" "re-run the world-enable helper with '--provision-agent-runtime codex' to re-add 'codex-runtime' and retry the sync" "world-enable rollback remediation"
  assert_file_contains "${stub_log}" "world deps global remove codex-runtime" "world-enable rollback should remove the newly-added enable"
  log "Verified world-enable rollback remediation explains how to re-add and retry."
}

run_dev_rollback_remediation_scenario() {
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-provision-runtime-dev-rollback.XXXXXX")"
  trap 'rm -rf "${work_root}"' RETURN

  local stub="${work_root}/substrate"
  local stub_log="${work_root}/substrate.log"
  write_stub_substrate "${stub}"
  : >"${stub_log}"

  local output
  set +e
  output="$(
    exec 2>&1
    export SUBSTRATE_STUB_LOG="${stub_log}"
    export STUB_ADD_MODE="added"
    export STUB_SYNC_EXIT=7
    export STUB_REMOVE_EXIT=0
    source_dev_runtime_provision_helpers
    PROVISION_AGENT_RUNTIME="codex"
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
    PREFIX="${work_root}/prefix"
    BIN_DIR="${work_root}/bin"
    mkdir -p "${BIN_DIR}"
    provision_agent_runtime_with_sync "${stub}"
  )"
  local status=$?
  set -e

  [[ "${status}" -eq 7 ]] || fatal "dev rollback scenario should exit 7, got ${status}"
  assert_contains "${output}" "the dev installer removed the global enable" "dev rollback remediation"
  assert_contains "${output}" "Re-run the dev install with '--provision-agent-runtime codex' to re-add 'codex-runtime' and retry the sync." "dev rollback remediation"
  assert_file_contains "${stub_log}" "world deps global remove codex-runtime" "dev rollback should remove the newly-added enable"
  log "Verified dev rollback remediation explains how to re-add and retry."
}

run_prod_install_then_sync_scenario
run_prod_rollback_remediation_scenario
run_world_enable_helper_rollback_remediation_scenario
run_dev_rollback_remediation_scenario

log "All provision-agent-runtime Packet 3 smoke checks passed."
