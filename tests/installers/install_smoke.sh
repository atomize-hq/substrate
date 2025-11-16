#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
ORIGINAL_PATH="${PATH}"
ORIGINAL_HOME="${HOME:-}"
DEFAULT_VERSION="0.0.0-test"
RC_SENTINEL="# substrate installer rc sentinel"
TEMP_DIRS=()
SCENARIO=""
KEEP_TEMP=0

usage() {
  cat <<'USAGE'
Usage: tests/installers/install_smoke.sh --scenario <default|no-world|uninstall|all> [--keep-temp]

Scenarios:
  default   Run installer with world provisioning + --sync-deps expectations
  no-world  Run installer with --no-world to verify metadata + guidance
  uninstall Install into a temp prefix then run the uninstaller to verify cleanup
  all       Execute every scenario sequentially

Flags:
  --keep-temp  Preserve the temporary fixture directories for debugging
USAGE
}

log_section() {
  printf '\n[%s] %s\n' "install-smoke" "$1"
}

fail() {
  printf '[install-smoke][ERROR] %s\n' "$1" >&2
  exit 1
}

cleanup() {
  if [[ ${KEEP_TEMP} -eq 1 ]]; then
    return
  fi
  for dir in "${TEMP_DIRS[@]:-}"; do
    [[ -n "${dir:-}" && -d "${dir}" ]] && rm -rf "${dir}"
  done
}

trap cleanup EXIT

parse_args() {
  if [[ $# -eq 0 ]]; then
    usage
    exit 1
  fi
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --scenario)
        [[ $# -ge 2 ]] || fail "--scenario requires a value"
        SCENARIO="$2"
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
        fail "Unknown argument: $1"
        ;;
    esac
  done
}

register_temp_dir() {
  local dir="$1"
  TEMP_DIRS+=("${dir}")
}

setup_install_fixture() {
  local scenario_name="$1"
  CURRENT_SCENARIO="${scenario_name}"
  WORKDIR="$(mktemp -d -t substrate-installer-${scenario_name}.XXXXXX)"
  register_temp_dir "${WORKDIR}"
  export HOME="${WORKDIR}/home"
  mkdir -p "${HOME}"
  STUB_BIN="${WORKDIR}/stub-bin"
  mkdir -p "${STUB_BIN}"
  export PATH="${STUB_BIN}:${ORIGINAL_PATH}"
  SCENARIO_LOG="${WORKDIR}/${scenario_name}.stubs.log"
  : >"${SCENARIO_LOG}"
  export SUBSTRATE_INSTALLER_TEST_LOG="${SCENARIO_LOG}"
  FAKE_SYSTEM_ROOT="${WORKDIR}/fake-system"
  mkdir -p "${FAKE_SYSTEM_ROOT}/usr/local/bin" \
           "${FAKE_SYSTEM_ROOT}/etc/systemd/system" \
           "${FAKE_SYSTEM_ROOT}/var/lib/substrate" \
           "${FAKE_SYSTEM_ROOT}/run/substrate"
  export SUBSTRATE_TEST_SYSTEM_ROOT="${FAKE_SYSTEM_ROOT}"
  ARTIFACT_DIR="${WORKDIR}/artifacts"
  mkdir -p "${ARTIFACT_DIR}"
  export SUBSTRATE_TEST_ARTIFACT_DIR="${ARTIFACT_DIR}"
  create_rc_sentinels
  create_stub_commands
}

restore_host_context() {
  export PATH="${ORIGINAL_PATH}"
  if [[ -n "${ORIGINAL_HOME}" ]]; then
    export HOME="${ORIGINAL_HOME}"
  fi
}

create_rc_sentinels() {
  for rc in .bashrc .zshrc .bash_profile; do
    printf '%s\n' "${RC_SENTINEL}" >"${HOME}/${rc}"
  done
}

create_stub_commands() {
  cat >"${STUB_BIN}/envsubst" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cat
EOF
  chmod +x "${STUB_BIN}/envsubst"

  for cmd in fuse-overlayfs nft ip limactl pkill pgrep; do
    cat >"${STUB_BIN}/${cmd}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${SUBSTRATE_INSTALLER_TEST_LOG:-}" ]]; then
  printf '[stub:%s] %s\n' "${0##*/}" "$*" >>"${SUBSTRATE_INSTALLER_TEST_LOG}"
fi
exit 0
EOF
    chmod +x "${STUB_BIN}/${cmd}"
  done

  cat >"${STUB_BIN}/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${SUBSTRATE_INSTALLER_TEST_LOG:-}" ]]; then
  printf '[stub:systemctl] %s\n' "$*" >>"${SUBSTRATE_INSTALLER_TEST_LOG}"
fi
case "$1" in
  status)
    printf 'systemctl status stub for %s\n' "${2:-substrate-world-agent}"
    ;;
  *)
    :
    ;;
esac
exit 0
EOF
  chmod +x "${STUB_BIN}/systemctl"

  cat >"${STUB_BIN}/sudo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log_stub() {
  if [[ -n "${SUBSTRATE_INSTALLER_TEST_LOG:-}" ]]; then
    printf '[stub:sudo] %s\n' "$*" >>"${SUBSTRATE_INSTALLER_TEST_LOG}"
  fi
}
rewrite_arg() {
  local arg="$1"
  local root="${SUBSTRATE_TEST_SYSTEM_ROOT:-}"
  if [[ -z "${root}" ]]; then
    printf '%s\n' "$arg"
    return
  fi
  case "$arg" in
    /usr/local/*|/etc/systemd/*|/var/lib/substrate*|/run/substrate*)
      printf '%s%s\n' "${root}" "$arg"
      ;;
    *)
      printf '%s\n' "$arg"
      ;;
  esac
}
args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -n|-E)
      shift
      ;;
    *)
      break
      ;;
  esac
done
if [[ $# -eq 0 ]]; then
  exit 0
fi
while [[ $# -gt 0 ]]; do
  args+=("$(rewrite_arg "$1")")
  shift
done
log_stub "${args[*]}"
exec "${args[@]}"
EOF
  chmod +x "${STUB_BIN}/sudo"
}

host_label() {
  local arch
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64)
      printf 'linux_x86_64'
      ;;
    aarch64|arm64)
      printf 'linux_aarch64'
      ;;
    *)
      fail "Unsupported architecture for installer tests: ${arch}"
      ;;
  esac
}

create_fake_release() {
  local version="$1"
  local label
  label="$(host_label)"
  local build_dir="${WORKDIR}/release-${label}"
  local root_dir="${build_dir}/substrate-v${version}-${label}"
  rm -rf "${build_dir}"
  mkdir -p "${root_dir}/bin"
  cat >"${root_dir}/bin/substrate" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log() {
  if [[ -n "${SUBSTRATE_INSTALLER_TEST_LOG:-}" ]]; then
    printf '[stub:substrate] %s\n' "$*" >>"${SUBSTRATE_INSTALLER_TEST_LOG}"
  fi
}
if [[ "$1" == "--shim-deploy" ]]; then
  log "shim deploy"
  exit 0
fi
case "$1" in
  world)
    shift
    case "$1" in
      doctor)
        shift
        printf '{"status":"ok","summary":{"healthy":true}}\n'
        exit 0
        ;;
      deps)
        shift
        if [[ "$1" == "sync" ]]; then
          log "world deps sync $*"
          exit 0
        fi
        ;;
    esac
    ;;
  --version)
    printf 'substrate-stub 0.0.0-test\n'
    exit 0
    ;;
  *)
    log "unknown command $*"
    exit 0
    ;;
esac
EOF
  chmod +x "${root_dir}/bin/substrate"
  cat >"${root_dir}/bin/world-agent" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${root_dir}/bin/world-agent"
  local archive="${ARTIFACT_DIR}/substrate-v${version}-${label}.tar.gz"
  tar -C "${build_dir}" -czf "${archive}" .
  (cd "${ARTIFACT_DIR}" && sha256sum "$(basename "${archive}")" >SHA256SUMS)
}

run_installer() {
  local log_path="$1"
  shift
  local args=("$@")
  if ! (cd "${REPO_ROOT}" && ./scripts/substrate/install-substrate.sh "${args[@]}") >"${log_path}" 2>&1; then
    cat "${log_path}" >&2
    fail "Installer failed for scenario '${CURRENT_SCENARIO}'. See ${log_path}"
  fi
}

run_uninstaller() {
  local log_path="$1"
  if ! (cd "${REPO_ROOT}" && ./scripts/substrate/uninstall-substrate.sh) >"${log_path}" 2>&1; then
    cat "${log_path}" >&2
    fail "Uninstaller failed for scenario '${CURRENT_SCENARIO}'. See ${log_path}"
  fi
}

assert_file_equals() {
  local path="$1"
  local expected="$2"
  if [[ ! -f "${path}" ]]; then
    fail "Expected file ${path} to exist"
  fi
  local contents
  contents="$(cat "${path}")"
  if [[ "${contents}" != "${expected}" ]]; then
    fail "File ${path} was modified unexpectedly"
  fi
}

assert_rc_pristine() {
  for rc in .bashrc .zshrc .bash_profile; do
    assert_file_equals "${HOME}/${rc}" "${RC_SENTINEL}"
  done
}

assert_file_contains() {
  local path="$1"
  local needle="$2"
  if ! grep -Fq "${needle}" "${path}"; then
    fail "Expected '${needle}' in ${path}"
  fi
}

assert_file_not_contains() {
  local path="$1"
  local needle="$2"
  if grep -Fq "${needle}" "${path}"; then
    fail "Did not expect '${needle}' in ${path}"
  fi
}

assert_manager_artifacts() {
  local expected_world_flag="$1"
  local expected_world_env="$2"
  local manager_root="${HOME}/.substrate"
  local env_path="${manager_root}/manager_env.sh"
  local init_path="${manager_root}/manager_init.sh"
  [[ -f "${env_path}" ]] || fail "Missing manager_env.sh"
  [[ -f "${init_path}" ]] || fail "Missing manager_init.sh"
  assert_file_contains "${env_path}" "manager_init.sh"
  assert_file_contains "${env_path}" ".substrate_bashenv"
  assert_file_contains "${env_path}" "SUBSTRATE_WORLD=${expected_world_env}"
  assert_file_contains "${env_path}" "SUBSTRATE_WORLD_ENABLED=${expected_world_flag}"
}

assert_config_world_flag() {
  local expected="$1"
  local config_path="${HOME}/.substrate/config.json"
  [[ -f "${config_path}" ]] || fail "Missing config.json"
  python3 - "${config_path}" "${expected}" <<'PY'
import json
import pathlib
import sys

config_path = pathlib.Path(sys.argv[1])
expected_raw = sys.argv[2].strip().lower()
expected = expected_raw == "true"
config = json.loads(config_path.read_text())
val = config.get("world_enabled")
if val is None:
    raise SystemExit("config.json missing world_enabled key")
if bool(val) != expected:
    raise SystemExit(f"world_enabled={val!r} (expected {expected})")
PY
}

assert_log_contains() {
  local log="$1"
  local needle="$2"
  if ! grep -Fq "${needle}" "${log}"; then
    fail "Expected log ${log} to contain '${needle}'"
  fi
}

assert_log_lacks() {
  local log="$1"
  local needle="$2"
  if grep -Fq "${needle}" "${log}"; then
    fail "Log ${log} unexpectedly contains '${needle}'"
  fi
}

assert_file_missing() {
  local path="$1"
  if [[ -e "${path}" ]]; then
    fail "Expected ${path} to be removed"
  fi
}

run_default_scenario() {
  log_section "Running default installer scenario"
  setup_install_fixture "default"
  create_fake_release "${DEFAULT_VERSION}"
  local dry_log="${WORKDIR}/default.dry-run.log"
  run_installer "${dry_log}" --version "${DEFAULT_VERSION}" --artifact-dir "${ARTIFACT_DIR}" --sync-deps --no-world --dry-run
  assert_log_contains "${dry_log}" "manager_env.sh"
  assert_log_contains "${dry_log}" "manager_init.sh"
  assert_log_contains "${dry_log}" "config.json"
  local install_log="${WORKDIR}/default.install.log"
  run_installer "${install_log}" --version "${DEFAULT_VERSION}" --artifact-dir "${ARTIFACT_DIR}" --sync-deps
  assert_rc_pristine
  assert_manager_artifacts "1" "enabled"
  assert_config_world_flag "True"
  assert_log_contains "${install_log}" "substrate world deps sync --all"
  printf '[install-smoke] default scenario artifacts: %s\n' "${WORKDIR}"
  restore_host_context
}

run_no_world_scenario() {
  log_section "Running --no-world installer scenario"
  setup_install_fixture "no-world"
  create_fake_release "${DEFAULT_VERSION}"
  local install_log="${WORKDIR}/noworld.install.log"
  run_installer "${install_log}" --version "${DEFAULT_VERSION}" --artifact-dir "${ARTIFACT_DIR}" --no-world
  assert_rc_pristine
  assert_manager_artifacts "0" "disabled"
  assert_config_world_flag "False"
  assert_log_contains "${install_log}" "substrate world enable"
  assert_log_contains "${install_log}" "Skipping world provisioning"
  assert_log_lacks "${install_log}" "substrate world deps sync"
  printf '[install-smoke] no-world scenario artifacts: %s\n' "${WORKDIR}"
  restore_host_context
}

run_uninstall_scenario() {
  log_section "Running uninstall scenario"
  setup_install_fixture "uninstall"
  create_fake_release "${DEFAULT_VERSION}"
  local install_log="${WORKDIR}/uninstall.install.log"
  run_installer "${install_log}" --version "${DEFAULT_VERSION}" --artifact-dir "${ARTIFACT_DIR}" --no-world
  local uninstall_log="${WORKDIR}/uninstall.uninstall.log"
  run_uninstaller "${uninstall_log}"
  local manager_root="${HOME}/.substrate"
  assert_file_missing "${manager_root}"
  assert_rc_pristine
  if [[ -n "${SUBSTRATE_TEST_SYSTEM_ROOT:-}" ]]; then
    assert_file_missing "${SUBSTRATE_TEST_SYSTEM_ROOT}/usr/local/bin/substrate-world-agent"
  fi
  printf '[install-smoke] uninstall scenario artifacts: %s\n' "${WORKDIR}"
  restore_host_context
}

run_scenarios() {
  case "${SCENARIO}" in
    default)
      run_default_scenario
      ;;
    no-world)
      run_no_world_scenario
      ;;
    uninstall)
      run_uninstall_scenario
      ;;
    all)
      run_default_scenario
      run_no_world_scenario
      run_uninstall_scenario
      ;;
    *)
      fail "Unknown scenario '${SCENARIO}'"
      ;;
  esac
}

parse_args "$@"
run_scenarios
