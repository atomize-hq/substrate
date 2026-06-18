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

if [[ "$*" == "world deps global add --json codex-runtime" ]]; then
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

if [[ "$*" == "world deps current sync" ]]; then
  exit "${STUB_SYNC_EXIT:-0}"
fi

if [[ "$*" == "world deps global remove codex-runtime" ]]; then
  exit "${STUB_REMOVE_EXIT:-0}"
fi

if [[ "$*" == "world deps current list applied --json" ]]; then
  printf '{"items":[]}\n'
  exit 0
fi

exit 0
EOF
  chmod +x "${path}"
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
    provision_agent_runtime_world_deps "${stub}"
    sync_world_deps "${stub}"
  2>&1)"
  local status=$?
  set -e

  [[ "${status}" -eq 7 ]] || fatal "prod rollback scenario should exit 7, got ${status}"
  assert_contains "${output}" "the installer removed the global enable" "prod rollback remediation"
  assert_contains "${output}" "re-run the install to re-add 'codex-runtime' and retry the sync" "prod rollback remediation"
  assert_file_contains "${stub_log}" "world deps global remove codex-runtime" "prod rollback should remove the newly-added enable"
  log "Verified prod rollback remediation explains how to re-add and retry."
}

run_dev_rollback_remediation_scenario() {
  assert_file_contains \
    "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" \
    "Re-run the dev install to re-add '%s' and retry the sync." \
    "dev rollback helper should keep the retry wording committed"
  assert_file_contains \
    "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" \
    "Then \$(dev_install_retry_after_sync_failure)" \
    "dev rollback path should append the retry guidance after rollback"
  log "Verified dev rollback remediation text stays explicit about re-adding and retrying."
}

run_prod_install_then_sync_scenario
run_prod_rollback_remediation_scenario
run_dev_rollback_remediation_scenario

log "All provision-agent-runtime Packet 3 smoke checks passed."
