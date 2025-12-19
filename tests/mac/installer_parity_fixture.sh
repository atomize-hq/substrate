#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FAKE_VERSION="${FAKE_VERSION:-0.0.1}"

usage() {
  cat <<'USAGE'
Usage: tests/mac/installer_parity_fixture.sh [--scenario <name>|--all] [--keep-root]

Scenarios:
  prod-copy         Production installer with bundled Linux agent (copy-first path).
  prod-build        Production installer fallback when Linux agent missing (build in Lima).
  dev-build         Dev installer (host cargo stub + in-guest build path).
  sync-deps         Production installer with --sync-deps (world deps sync wired).
  cleanup-guidance  Uninstaller cleanup-state guidance on mac hosts.
  all               Run every scenario (default).

This harness stubs mac tooling (limactl/envsubst/jq/file/cargo/uname/sysctl) so the
installers/uninstallers can run on non-mac hosts without touching host state.
USAGE
}

fatal() {
  printf '[mac-installer-fixture][ERROR] %s\n' "$*" >&2
  exit 1
}

info() {
  printf '[mac-installer-fixture] %s\n' "$*" >&2
}

SCENARIO="all"
KEEP_ROOT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      [[ $# -lt 2 ]] && fatal "missing value for --scenario"
      SCENARIO="$2"
      shift 2
      ;;
    --all)
      SCENARIO="all"
      shift
      ;;
    --keep-root)
      KEEP_ROOT=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fatal "unknown argument: $1"
      ;;
  esac
done

HOST_PATH="${PATH:-}"
declare -a WORK_ROOTS=()

cleanup_all() {
  if [[ "${KEEP_ROOT}" -eq 1 ]]; then
    for dir in "${WORK_ROOTS[@]}"; do
      info "preserving ${dir} (--keep-root set)"
    done
    return
  fi
  for dir in "${WORK_ROOTS[@]}"; do
    rm -rf "${dir}"
  done
}
trap cleanup_all EXIT

setup_workspace() {
  local label="$1"
  WORK_ROOT="$(mktemp -d "/tmp/substrate-mac-installer-${label}.XXXXXX")"
  WORK_ROOTS+=("${WORK_ROOT}")
  STUB_BIN="${WORK_ROOT}/stub-bin"
  mkdir -p "${STUB_BIN}"
  PATH="${STUB_BIN}:${HOST_PATH}"
  export PATH
  export SHIM_ORIGINAL_PATH="${PATH}"
  HOME="${WORK_ROOT}/home"
  mkdir -p "${HOME}"
  export HOME
  export SUBSTRATE_TEST_LIMACTL_LOG="${WORK_ROOT}/limactl-${label}.log"
  export SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR="${WORK_ROOT}/limactl-${label}-scripts"
  mkdir -p "${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"
  export SUBSTRATE_TEST_CARGO_LOG="${WORK_ROOT}/cargo-${label}.log"
  export SUBSTRATE_TEST_FILE_SENTINEL="ELF-STUB"
}

write_stub() {
  local name="$1"
  cat > "${STUB_BIN}/${name}"
  chmod +x "${STUB_BIN}/${name}"
}

write_stub_uname() {
  write_stub uname <<'STUB'
#!/usr/bin/env bash
if [[ $# -eq 0 || "$1" == "-s" ]]; then
  printf 'Darwin\n'
  exit 0
fi
if [[ "$1" == "-m" ]]; then
  printf 'arm64\n'
  exit 0
fi
/usr/bin/uname "$@"
STUB
}

write_stub_sysctl() {
  write_stub sysctl <<'STUB'
#!/usr/bin/env bash
if [[ "$*" == "-n kern.hv_support" ]]; then
  printf '1\n'
  exit 0
fi
printf '0\n'
STUB
}

write_stub_sw_vers() {
  write_stub sw_vers <<'STUB'
#!/usr/bin/env bash
cat <<'OUT'
ProductName:    macOS
ProductVersion: 14.0
BuildVersion:   23A000
OUT
STUB
}

write_stub_envsubst() {
  write_stub envsubst <<'STUB'
#!/usr/bin/env python3
import os, sys
content = sys.stdin.read()
project = os.environ.get("PROJECT", "")
for token in ("${PROJECT}", "$PROJECT"):
    content = content.replace(token, project)
sys.stdout.write(content)
STUB
}

write_stub_jq() {
  write_stub jq <<'STUB'
#!/usr/bin/env python3
import json, sys
data = sys.stdin.read().strip()
try:
    obj = json.loads(data)
    status = obj.get("status", "unknown")
except Exception:
    status = "unknown"
sys.stdout.write(status + "\n")
STUB
}

write_stub_file() {
  write_stub file <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if grep -q "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" "$1" 2>/dev/null; then
  printf 'ELF 64-bit LSB executable\n'
else
  printf 'Mach-O 64-bit executable\n'
fi
STUB
}

write_stub_sudo() {
  write_stub sudo <<'STUB'
#!/usr/bin/env bash
if [[ "$1" == "-n" ]]; then
  shift
fi
"$@"
STUB
}

write_stub_limactl() {
  write_stub limactl <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_LIMACTL_LOG:-}"
capture_dir="${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR:-}"
record() {
  [[ -n "${log}" ]] && printf '%s\n' "$*" >>"${log}"
}
next_script() {
  [[ -z "${capture_dir}" ]] && { cat >/dev/null; return; }
  mkdir -p "${capture_dir}"
  local counter="${capture_dir}/.counter"
  local idx=0
  [[ -f "${counter}" ]] && idx="$(<"${counter}")"
  idx=$((idx + 1))
  printf '%s\n' "${idx}" >"${counter}"
  local target
  target=$(printf '%s/shell-%03d.sh' "${capture_dir}" "${idx}")
  cat >"${target}"
  record "shell-script:${target}"
}
if [[ $# -lt 1 ]]; then
  exit 0
fi
cmd="$1"
shift
case "${cmd}" in
  list)
    record "list $*"
    if [[ "$*" == *"--json"* ]]; then
      printf '{"status":"Running"}\n'
    else
      printf 'substrate\tRunning\n'
    fi
    ;;
  start|stop|delete)
    record "${cmd} $*"
    ;;
  copy)
    record "copy $*"
    ;;
  shell)
    vm="${1:-substrate}"
    shift
    record "shell ${vm} $*"
    if [[ "$1" == "id" && "$2" == "-un" ]]; then
      printf 'substrate\n'
      exit 0
    fi
    if [[ "$1" == "sudo" ]]; then
      shift
      if [[ "$1" == "-n" ]]; then
        shift
      fi
      if [[ "$1" == "cat" && "$2" == "/etc/substrate-lima-layout" ]]; then
        printf 'socket-parity-v1\n'
        exit 0
      fi
      if [[ "$1" == "stat" ]]; then
        printf 'root:substrate 660\n'
        exit 0
      fi
      if [[ "$1" == "loginctl" ]]; then
        printf 'Linger=yes\n'
        exit 0
      fi
      if [[ "$1" == "test" ]]; then
        exit 0
      fi
    fi
    if [[ "$1" == "env" ]]; then
      while [[ $# -gt 0 ]]; do
        if [[ "$1" == "bash" ]]; then
          break
        fi
        shift
      done
    fi
    if [[ "$1" == "bash" ]]; then
      shift
      next_script
      exit 0
    fi
    exit 0
    ;;
  *)
    record "${cmd} $*"
    ;;
 esac
STUB
}

write_stub_cargo() {
  write_stub cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_CARGO_LOG:-}"
[[ -n "${log}" ]] && printf '%s\n' "$*" >>"${log}"
target="debug"
for arg in "$@"; do
  if [[ "${arg}" == "--release" ]]; then
    target="release"
  fi
done
mkdir -p "target/${target}"
write_bin() {
  local path="$1"
  echo "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${path}"
  chmod +x "${path}"
}
case "$*" in
  *"--bin substrate"*)
    write_bin "target/${target}/substrate"
    write_bin "target/${target}/substrate-shim"
    ;;
 esac
case "$*" in
  *"-p world-agent"*)
    write_bin "target/${target}/world-agent"
    ;;
 esac
STUB
}

install_common_stubs() {
  write_stub_uname
  write_stub_sysctl
  write_stub_sw_vers
  write_stub_envsubst
  write_stub_jq
  write_stub_file
  write_stub_sudo
  write_stub_limactl
}

prepare_release_bundle() {
  local label="$1"
  local include_agent="$2"
  local stage="${WORK_ROOT}/release-stage"
  local artifact_dir="${WORK_ROOT}/artifacts-${label}"
  rm -rf "${stage}" "${artifact_dir}"
  mkdir -p "${stage}/bin/linux" "${stage}/scripts/mac" "${stage}/scripts/substrate" "${stage}/config" "${artifact_dir}"
  cp "${REPO_ROOT}/config/manager_hooks.yaml" "${stage}/config/manager_hooks.yaml"
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${stage}/scripts/substrate/world-deps.yaml"
  cat >"${stage}/scripts/mac/lima-warm.sh" <<'LIMA'
#!/usr/bin/env bash
set -euo pipefail
VM_NAME="${LIMA_VM_NAME:-substrate}"
PROJECT_PATH="${1:-$(pwd)}"
BUILD_PROFILE="${LIMA_BUILD_PROFILE:-release}"
log() { printf '[lima-warm-stub] %s\n' "$1"; }
host_cli="${PROJECT_PATH}/bin/linux/substrate"
host_agent="${PROJECT_PATH}/bin/linux/world-agent"
if [[ -f "${host_cli}" && -f "${host_agent}" ]]; then
  log "Installing Linux CLI/agent from host bundle"
  limactl copy "${host_cli}" "${VM_NAME}:/tmp/substrate-cli"
  limactl copy "${host_agent}" "${VM_NAME}:/tmp/world-agent"
  limactl shell "${VM_NAME}" sudo install -Dm0755 /tmp/substrate-cli /usr/local/bin/substrate
  limactl shell "${VM_NAME}" sudo install -Dm0755 /tmp/world-agent /usr/local/bin/substrate-world-agent
else
  log "Host Linux binaries missing; building inside Lima"
  limactl shell "${VM_NAME}" env BUILD_PROFILE="${BUILD_PROFILE}" bash <<'EOF'
set -euo pipefail
echo "[lima-warm-stub] building substrate"
cargo build --bin substrate
echo "[lima-warm-stub] building world-agent"
cargo build -p world-agent
EOF
fi
limactl shell "${VM_NAME}" sudo systemctl daemon-reload
limactl shell "${VM_NAME}" sudo systemctl restart substrate-world-agent.service
LIMA
  chmod +x "${stage}/scripts/mac/lima-warm.sh"
  cat >"${stage}/bin/substrate" <<'BIN'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${SUBSTRATE_TEST_SUBSTRATE_LOG:-}" ]]; then
  printf '%s\n' "$*" >>"${SUBSTRATE_TEST_SUBSTRATE_LOG}"
fi
if [[ "$1" == "--shim-deploy" ]]; then
  printf '[fake-substrate] shim deploy\n' >&2
  exit 0
fi
if [[ "$1" == "--version" ]]; then
  printf 'fake\n'
  exit 0
fi
exit 0
BIN
  chmod +x "${stage}/bin/substrate"
  printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${stage}/bin/linux/substrate"
  chmod +x "${stage}/bin/linux/substrate"
  if [[ "${include_agent}" -eq 1 ]]; then
    printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${stage}/bin/linux/world-agent"
    chmod +x "${stage}/bin/linux/world-agent"
  fi
  local archive="substrate-v${FAKE_VERSION}-macos_arm64.tar.gz"
  tar -C "${stage}" -czf "${artifact_dir}/${archive}" .
  printf '%s\n' "${artifact_dir}"
}

assert_contains() {
  local file="$1"
  local pattern="$2"
  local msg="$3"
  if ! grep -Eq -- "${pattern}" "${file}"; then
    fatal "${msg}: pattern '${pattern}' missing in ${file}"
  fi
}

assert_not_contains() {
  local file="$1"
  local pattern="$2"
  local msg="$3"
  if grep -Eq -- "${pattern}" "${file}"; then
    fatal "${msg}: unexpected pattern '${pattern}' in ${file}"
  fi
}

run_prod_scenario() {
  local label="$1"
  local include_agent="$2"
  info "Running scenario ${label} (include_agent=${include_agent})"
  setup_workspace "${label}"
  install_common_stubs
  local artifact_dir
  artifact_dir="$(prepare_release_bundle "${label}" "${include_agent}")"
  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --version "${FAKE_VERSION}" \
    --prefix "${prefix}" \
    --artifact-dir "${artifact_dir}" \
    --no-shims >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "install-substrate failed for ${label}"
  fi
  local limactl_log="${SUBSTRATE_TEST_LIMACTL_LOG}"
  local capture_dir="${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"
  if [[ "${include_agent}" -eq 1 ]]; then
    assert_contains "${limactl_log}" 'copy .*world-agent' "prod-copy should copy bundled agent"
  fi
  local has_build=0
  if [[ -d "${capture_dir}" ]] && grep -R "cargo build -p world-agent" "${capture_dir}" >/dev/null 2>&1; then
    has_build=1
  fi
  if [[ "${include_agent}" -eq 1 && "${has_build}" -eq 1 ]]; then
    fatal "prod-copy unexpectedly triggered in-guest build"
  fi
  if [[ "${include_agent}" -eq 0 && "${has_build}" -eq 0 ]]; then
    fatal "prod-build did not trigger in-guest build"
  fi
  info "Scenario ${label} complete:"
  info "  install log: ${log}"
  info "  limactl log: ${limactl_log}"
  info "  capture dir: ${capture_dir}"
}

run_dev_scenario() {
  local label="dev-build"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  write_stub_cargo
  local build_log="${WORK_ROOT}/host-build.log"
  if ! cargo build --bin substrate --bin substrate-shim >"${build_log}" 2>&1; then
    cat "${build_log}" >&2 || true
    fatal "host cargo build simulation failed"
  fi
  assert_contains "${SUBSTRATE_TEST_CARGO_LOG}" '--bin substrate' "dev build should invoke host cargo"
  local artifact_dir
  artifact_dir="$(prepare_release_bundle "dev" 0)"
  local stage_dir="${WORK_ROOT}/release-stage"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${stage_dir}/scripts/mac/lima-warm.sh" "${stage_dir}" >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "lima warm stub failed for dev scenario"
  fi
  local capture_dir="${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"
  if [[ ! -d "${capture_dir}" ]] || ! grep -R "cargo build -p world-agent" "${capture_dir}" >/dev/null 2>&1; then
    fatal "dev scenario did not trigger in-guest world-agent build"
  fi
  info "Scenario ${label} complete:"
  info "  host build log: ${build_log}"
  info "  lima stub log: ${log}"
  info "  limactl log: ${SUBSTRATE_TEST_LIMACTL_LOG}"
  info "  capture dir: ${capture_dir}"
}

run_sync_deps_scenario() {
  local label="sync-deps"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  local artifact_dir
  artifact_dir="$(prepare_release_bundle "${label}" 1)"
  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  local substrate_log="${WORK_ROOT}/${label}-substrate.log"
  export SUBSTRATE_TEST_SUBSTRATE_LOG="${substrate_log}"
  if ! "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --version "${FAKE_VERSION}" \
    --prefix "${prefix}" \
    --artifact-dir "${artifact_dir}" \
    --no-shims \
    --sync-deps >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "install-substrate failed for ${label}"
  fi
  assert_contains "${log}" "Syncing guest dependencies via 'substrate world deps sync'" \
    "sync-deps should announce world deps sync"
  assert_contains "${substrate_log}" "world deps sync" \
    "sync-deps should invoke world deps sync"
  unset SUBSTRATE_TEST_SUBSTRATE_LOG
  info "Scenario ${label} complete:"
  info "  install log: ${log}"
  info "  substrate log: ${substrate_log}"
}

run_cleanup_guidance() {
  local label="cleanup-guidance"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  local metadata_root="${HOME}/.substrate"
  mkdir -p "${metadata_root}"
  local metadata="${metadata_root}/install_state.json"
  cat >"${metadata}" <<'JSON'
{
  "schema_version": 1,
  "host_state": {
    "group": {
      "existed_before": false,
      "created_by_installer": true,
      "members_added": ["tester"]
    },
    "linger": {
      "users": {
        "tester": { "enabled_by_substrate": true }
      }
    }
  }
}
JSON
  local log="${WORK_ROOT}/${label}.log"
  if ! "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh" --cleanup-state >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "uninstall-substrate failed"
  fi
  assert_contains "${log}" 'Host-state cleanup is only supported on Linux' "expected mac cleanup guidance"
  info "Scenario ${label} complete:"
  info "  uninstall log: ${log}"
  info "  metadata fixture: ${metadata}"
}

run_selected() {
  case "${SCENARIO}" in
    prod-copy)
      run_prod_scenario "prod-copy" 1
      ;;
    prod-build)
      run_prod_scenario "prod-build" 0
      ;;
    dev-build)
      run_dev_scenario
      ;;
    sync-deps)
      run_sync_deps_scenario
      ;;
    cleanup-guidance)
      run_cleanup_guidance
      ;;
    all)
      run_prod_scenario "prod-copy" 1
      run_prod_scenario "prod-build" 0
      run_dev_scenario
      run_sync_deps_scenario
      run_cleanup_guidance
      ;;
    *)
      fatal "unsupported scenario: ${SCENARIO}"
      ;;
  esac
}

run_selected
