#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FAKE_VERSION="${FAKE_VERSION:-0.0.1}"
ACCOUNT_HOME="$(python3 - <<'PY'
import os
import pwd

print(pwd.getpwuid(os.getuid()).pw_dir)
PY
)"
ACCOUNT_LIMA_HOME="${ACCOUNT_HOME%/}/.lima"

usage() {
  cat <<'USAGE'
Usage: tests/mac/installer_parity_fixture.sh [--scenario <name>|--all] [--keep-root]

Scenarios:
  prod-copy         Production installer with bundled Linux agent (copy-first path).
  prod-build        Production installer fallback when Linux agent missing (build in Lima).
  dev-build         Dev installer (host cargo stub + in-guest build path).
  dev-call-section  Dev installer forwards the validated prefix/carrier into lima-warm.sh.
  dev-runtime-bundle  Dev installer stages the stable world-enable runtime bundle under SUBSTRATE_HOME.
  dev-runtime-bundle-self-contained  Dev installer persists Linux guest binaries into the prefix bundle on macOS.
  dev-runtime-bundle-protected-path-conflicts  Dev uninstall refuses explicit managed target conflicts with exit 5.
  sync-deps         Production installer with --sync-deps (world deps current sync wired).
  sync-deps-remediation  Production installer handles sync exit 4 with remediation guidance.
  sync-deps-generic-failure  Production installer handles non-4 sync failures with a generic warning only.
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
  export SUBSTRATE_TEST_CARGO_MARKER="MACHO-STUB"
  export SUBSTRATE_INSTALL_NO_PATH=1
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
import os
import re
import sys

content = sys.stdin.read()
pattern = re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)")


def replace(match):
    name = match.group(1) or match.group(2)
    return os.environ.get(name, "")


content = pattern.sub(replace, content)
sys.stdout.write(content)
STUB
}

write_stub_jq() {
  write_stub jq <<'STUB'
#!/usr/bin/env python3
import json
import sys

args = sys.argv[1:]
raw = sys.stdin.read()
try:
    payload = json.loads(raw or "null")
except Exception:
    payload = None

if not args:
    sys.exit(2)

if args == ["."]:
    sys.stdout.write(raw)
    sys.exit(0)

if args == ["-r"]:
    sys.exit(2)

if len(args) >= 2 and args[0] == "-r":
    expr = args[1]
else:
    expr = args[0]

if "(.items | length) == 0" in expr:
    items = payload.get("items", []) if isinstance(payload, dict) else []
    if not items:
        sys.stdout.write("  (no enabled deps items)\n")
        sys.exit(0)
    for item in items:
        name = item.get("name", "")
        kind = item.get("kind", "")
        enabled = item.get("enabled", False)
        world = item.get("world", "unknown")
        remediation = item.get("remediation")
        line = f"- {name}: kind={kind} enabled={str(enabled).lower()} world={world}"
        if remediation:
            line += f" remediation={remediation}"
        sys.stdout.write(line + "\n")
    sys.exit(0)

status = payload.get("status", "unknown") if isinstance(payload, dict) else "unknown"
sys.stdout.write(status + "\n")
STUB
}

write_stub_file() {
  write_stub file <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "-b" ]]; then
  shift
fi
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
  [[ -n "${log}" ]] && printf 'HOME=%s\tLIMA_HOME=%s\t%s\n' "${HOME:-}" "${LIMA_HOME:-}" "$*" >>"${log}"
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
next_capture_path() {
  local basename="$1"
  [[ -z "${capture_dir}" ]] && return 1
  mkdir -p "${capture_dir}"
  local counter="${capture_dir}/.counter"
  local idx=0
  [[ -f "${counter}" ]] && idx="$(<"${counter}")"
  idx=$((idx + 1))
  printf '%s\n' "${idx}" >"${counter}"
  printf '%s/copy-%03d-%s\n' "${capture_dir}" "${idx}" "${basename}"
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
    src="${1:-}"
    dest="${2:-}"
    if [[ -n "${src}" && -n "${dest}" && "${src}" != *:* && "${dest}" == *:* && -f "${src}" ]]; then
      captured_copy="$(next_capture_path "$(basename "${src}")")" || captured_copy=""
      if [[ -n "${captured_copy}" ]]; then
        cp "${src}" "${captured_copy}"
        record "copy-capture:${captured_copy}"
      fi
    fi
    if [[ -n "${src}" && -n "${dest}" && "${src}" == *:* && "${dest}" != *:* ]]; then
      mkdir -p "$(dirname "${dest}")"
      printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${dest}"
      chmod +x "${dest}"
    fi
    ;;
  shell)
    vm="${1:-substrate}"
    shift
    record "shell ${vm} $*"
    guest_account="${SUBSTRATE_TEST_GUEST_ACCOUNT:-substrate}"
    guest_uid="${SUBSTRATE_TEST_GUEST_UID:-2000}"
    guest_home="${SUBSTRATE_TEST_GUEST_HOME:-/home/substrate}"
    guest_machine_id="${SUBSTRATE_TEST_MACHINE_ID:-17171717171717171717171717171717}"
    if [[ "$1" == "cat" && "${2:-}" == "/etc/machine-id" ]]; then
      printf '%s\n' "${guest_machine_id}"
      exit 0
    fi
    if [[ "$1" == "id" && "$2" == "-un" ]]; then
      printf '%s\n' "${guest_account}"
      exit 0
    fi
    if [[ "$1" == "id" && "$2" == "-u" ]]; then
      printf '%s\n' "${guest_uid}"
      exit 0
    fi
    if [[ "$1" == "getent" && "$2" == "passwd" ]]; then
      query="${3:-}"
      if [[ "${query}" == "${guest_account}" || "${query}" == "${guest_uid}" ]]; then
        printf '%s:x:%s:%s:Substrate:%s:/bin/bash\n' "${guest_account}" "${guest_uid}" "${guest_uid}" "${guest_home}"
      fi
      exit 0
    fi
    if [[ "$1" == "sudo" ]]; then
      shift
      if [[ "$1" == "-n" ]]; then
        shift
      fi
      if [[ "$1" == "cat" && "$2" == "/etc/substrate-lima-layout" ]]; then
        printf 'socket-parity-v2-staged-workspace-v1\n'
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
  cat >"${path}" <<'BIN'
#!/usr/bin/env bash
set -euo pipefail
# MACHO-STUB
if [[ "${1:-}" == "--shim-deploy" ]]; then
  printf '[fake-substrate] shim deploy\n' >&2
  exit 0
fi
if [[ "${1:-}" == "--version" ]]; then
  printf 'fake\n'
  exit 0
fi
if [[ "$*" == *"--install-bootstrap-home-v1"* ]]; then
  exit 0
fi
if [[ "$*" == *"world doctor --json"* ]]; then
  printf '{"ok":true,"host":{"ok":true},"world":{"ok":true,"status":"ok"}}\n'
  exit 0
fi
if [[ "$*" == *"world deps current list applied --json"* ]]; then
  printf '{"items":[]}\n'
  exit 0
fi
if [[ "$*" == *"world deps global add --json"* ]]; then
  deps_item="${@: -1}"
  printf '{"items":["%s"]}\n' "${deps_item}"
  exit 0
fi
if [[ "$*" == *"world deps global remove"* ]]; then
  exit 0
fi
if [[ "$*" == *"world deps current sync"* ]]; then
  sync_deps_exit_code="${SUBSTRATE_TEST_SYNC_DEPS_EXIT_CODE:-0}"
  if [[ "${SUBSTRATE_TEST_SYNC_DEPS_EXIT_4:-0}" == "1" ]]; then
    sync_deps_exit_code=4
  fi
  if [[ "${sync_deps_exit_code}" != "0" ]]; then
    if [[ "${sync_deps_exit_code}" == "4" ]]; then
      printf 'substrate world enable --provision-deps\n' >&2
    else
      printf 'sync failed\n' >&2
    fi
    exit "${sync_deps_exit_code}"
  fi
  exit 0
fi
exit 0
BIN
  chmod +x "${path}"
}
case "$*" in
  *"--bin substrate"*)
    write_bin "target/${target}/substrate"
    write_bin "target/${target}/substrate-shim"
    ;;
 esac
case "$*" in
  *"-p world-service"*)
    write_bin "target/${target}/world-service"
    ;;
  *"-p substrate-gateway"*)
    write_bin "target/${target}/substrate-gateway"
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
  mkdir -p \
    "${stage}/bin/linux" \
    "${stage}/scripts/mac/lima/units" \
    "${stage}/scripts/substrate" \
    "${stage}/config" \
    "${artifact_dir}"
  if [[ -f "${REPO_ROOT}/config/manager_hooks.yaml" ]]; then
    cp "${REPO_ROOT}/config/manager_hooks.yaml" "${stage}/config/manager_hooks.yaml"
  else
    cat >"${stage}/config/manager_hooks.yaml" <<'MANIFEST'
version: 1
tools: []
MANIFEST
  fi
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${stage}/scripts/substrate/world-deps.yaml"
  cp "${REPO_ROOT}/scripts/mac/lima/units/substrate-world-service.service.tmpl" \
    "${stage}/scripts/mac/lima/units/substrate-world-service.service.tmpl"
  cp "${REPO_ROOT}/scripts/mac/lima/units/substrate-world-service.socket" \
    "${stage}/scripts/mac/lima/units/substrate-world-service.socket"
cat >"${stage}/scripts/mac/lima-warm.sh" <<'LIMA'
#!/usr/bin/env bash
set -euo pipefail
VM_NAME="${SUBSTRATE_LIMA_VM_NAME:-substrate}"
BUILD_PROFILE="${LIMA_BUILD_PROFILE:-release}"
INSTALL_PREFIX=""
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PROJECT_PATH=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-prefix)
      INSTALL_PREFIX="${2:?missing install prefix}"
      shift 2
      ;;
    --install-bootstrap-context-v1)
      INSTALL_BOOTSTRAP_CONTEXT_V1="${2:?missing install bootstrap context}"
      shift 2
      ;;
    --)
      shift
      break
      ;;
    -*)
      printf '[lima-warm-stub][ERROR] unknown arg: %s\n' "$1" >&2
      exit 2
      ;;
    *)
      PROJECT_PATH="$1"
      shift
      break
      ;;
  esac
done
PROJECT_PATH="${PROJECT_PATH:-${1:-$(pwd)}}"
[[ -n "${INSTALL_PREFIX}" ]] || exit 21
[[ -n "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]] || exit 22
[[ "${INSTALL_PREFIX}" == "${SUBSTRATE_HOME:-}" ]] || exit 23
[[ "${INSTALL_PREFIX}" == "${SUBSTRATE_ROOT:-}" ]] || exit 24
[[ "${INSTALL_BOOTSTRAP_CONTEXT_V1}" == "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" ]] || exit 25
log() { printf '[lima-warm-stub] %s\n' "$1"; }
host_cli="${PROJECT_PATH}/bin/linux/substrate"
host_world_service="${PROJECT_PATH}/bin/linux/world-service"
host_gateway="${PROJECT_PATH}/bin/linux/substrate-gateway"
if [[ -f "${host_cli}" && -f "${host_world_service}" && -f "${host_gateway}" ]]; then
  log "Installing Linux CLI/world-service/gateway from host bundle"
  limactl copy "${host_cli}" "${VM_NAME}:/tmp/substrate-cli"
  limactl copy "${host_world_service}" "${VM_NAME}:/tmp/world-service"
  limactl copy "${host_gateway}" "${VM_NAME}:/tmp/substrate-gateway"
  limactl shell "${VM_NAME}" sudo install -Dm0755 /tmp/substrate-cli /usr/local/bin/substrate
  limactl shell "${VM_NAME}" sudo install -Dm0755 /tmp/world-service /usr/local/bin/substrate-world-service
  limactl shell "${VM_NAME}" sudo install -Dm0755 /tmp/substrate-gateway /usr/local/bin/substrate-gateway
else
  log "Host Linux binaries missing; building inside Lima"
  limactl shell "${VM_NAME}" env BUILD_PROFILE="${BUILD_PROFILE}" bash <<'EOF'
set -euo pipefail
echo "[lima-warm-stub] building substrate"
cargo build --bin substrate
echo "[lima-warm-stub] building world-service"
cargo build -p world-service
echo "[lima-warm-stub] building substrate-gateway"
cargo build -p substrate-gateway
EOF
fi
service_template="${PROJECT_PATH}/scripts/mac/lima/units/substrate-world-service.service.tmpl"
socket_template="${PROJECT_PATH}/scripts/mac/lima/units/substrate-world-service.socket"
rendered_units_dir="$(mktemp -d)"
SUBSTRATE_GUEST_HOME="/home/substrate/.substrate" \
SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" \
SUBSTRATE_LIMA_INSTANCE_NAME="${VM_NAME}" \
SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT="${LIMA_HOME:-${HOME%/}/.lima}" \
SUBSTRATE_LIMA_HOST_SOCKET="${INSTALL_PREFIX}/sock/agent.sock" \
SUBSTRATE_LIMA_GUEST_SOCKET="/run/substrate.sock" \
WORLD_NETFILTER_ENV="" \
  envsubst < "${service_template}" > "${rendered_units_dir}/substrate-world-service.service"
envsubst < "${socket_template}" > "${rendered_units_dir}/substrate-world-service.socket"
limactl copy "${rendered_units_dir}/substrate-world-service.service" "${VM_NAME}:/tmp/substrate-world-service.service"
limactl copy "${rendered_units_dir}/substrate-world-service.socket" "${VM_NAME}:/tmp/substrate-world-service.socket"
rm -rf "${rendered_units_dir}"
limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0644 /tmp/substrate-world-service.service /etc/systemd/system/substrate-world-service.service
sudo install -Dm0644 /tmp/substrate-world-service.socket /etc/systemd/system/substrate-world-service.socket
sudo rm -f /tmp/substrate-world-service.service /tmp/substrate-world-service.socket
EOF
limactl shell "${VM_NAME}" sudo systemctl daemon-reload
limactl shell "${VM_NAME}" sudo systemctl restart substrate-world-service.service
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
  if [[ "$*" == "world doctor --json" ]]; then
    printf '{"ok":true,"host":{"ok":true},"world":{"ok":true,"status":"ok"}}\n'
    exit 0
  fi
  if [[ "$*" == "world deps current list applied --json" ]]; then
    printf '{"items":[]}\n'
    exit 0
  fi
  if [[ "$*" == "world deps current sync" ]]; then
    sync_deps_exit_code="${SUBSTRATE_TEST_SYNC_DEPS_EXIT_CODE:-0}"
    if [[ "${SUBSTRATE_TEST_SYNC_DEPS_EXIT_4:-0}" == "1" ]]; then
      sync_deps_exit_code=4
    fi
    if [[ "${sync_deps_exit_code}" != "0" ]]; then
      if [[ "${sync_deps_exit_code}" == "4" ]]; then
        printf 'substrate world enable --provision-deps\n' >&2
      else
        printf 'sync failed\n' >&2
      fi
      exit "${sync_deps_exit_code}"
    fi
  fi
  exit 0
BIN
  chmod +x "${stage}/bin/substrate"
  printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${stage}/bin/linux/substrate"
  chmod +x "${stage}/bin/linux/substrate"
  if [[ "${include_agent}" -eq 1 ]]; then
    printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${stage}/bin/linux/world-service"
    chmod +x "${stage}/bin/linux/world-service"
    printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${stage}/bin/linux/substrate-gateway"
    chmod +x "${stage}/bin/linux/substrate-gateway"
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

assert_contains_literal() {
  local file="$1"
  local pattern="$2"
  local msg="$3"
  if ! grep -Fq -- "${pattern}" "${file}"; then
    fatal "${msg}: literal '${pattern}' missing in ${file}"
  fi
}

authoritative_install_commitment() {
  local prefix="$1"
  python3 - "${prefix}" <<'PY'
import base64
import hashlib
import os
import pwd
import sys

DOMAIN = "substrate.install_bootstrap_context"


def b64_encode(value: bytes) -> str:
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


prefix = sys.argv[1]
entry = pwd.getpwuid(os.getuid())
uid = os.getuid()
encoded_prefix = b64_encode(prefix.encode("utf-8"))
frame = (
    f"domain={DOMAIN}\nversion=1\nselected_host_prefix={encoded_prefix}\n"
    f"host_substrate_home={encoded_prefix}\nhost_substrate_root={encoded_prefix}\n"
    f"principal_kind=unix\nprincipal_account={b64_encode(entry.pw_name.encode('utf-8'))}\n"
    f"principal_uid={uid}\n"
).encode("ascii")
print(hashlib.sha256(frame).hexdigest())
PY
}

latest_captured_copy_path() {
  local capture_dir="$1"
  local basename="$2"
  local file
  file="$(find "${capture_dir}" -maxdepth 1 -type f -name "copy-*-$(basename "${basename}")" | LC_ALL=C sort | tail -n 1)"
  [[ -n "${file}" ]] || fatal "expected captured copy for ${basename}"
  printf '%s\n' "${file}"
}

assert_captured_copy_contains_literal() {
  local capture_dir="$1"
  local basename="$2"
  local pattern="$3"
  local msg="$4"
  local file
  file="$(latest_captured_copy_path "${capture_dir}" "${basename}")"
  assert_contains_literal "${file}" "${pattern}" "${msg}"
}

assert_limactl_fragment_env() {
  local log_path="$1"
  local expected_home="$2"
  local expected_lima_home="$3"
  local fragment="$4"
  local msg="$5"
  awk -F'\t' \
    -v home="HOME=${expected_home}" \
    -v lima="LIMA_HOME=${expected_lima_home}" \
    -v fragment="${fragment}" '
    $1 == home && $2 == lima && index($3, fragment) { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "${log_path}" || fatal "${msg}: expected HOME=${expected_home} LIMA_HOME=${expected_lima_home} for '${fragment}'"
}

assert_limactl_fragment_not_ambient() {
  local log_path="$1"
  local ambient_home="$2"
  local ambient_lima_home="$3"
  local fragment="$4"
  local msg="$5"
  if awk -F'\t' \
    -v home="HOME=${ambient_home}" \
    -v lima="LIMA_HOME=${ambient_lima_home}" \
    -v fragment="${fragment}" '
    $1 == home && $2 == lima && index($3, fragment) { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "${log_path}"; then
    fatal "${msg}: fragment '${fragment}' used ambient HOME/LIMA_HOME"
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
  local ambient_lima_home="${WORK_ROOT}/ambient-lima"
  mkdir -p "${prefix}"
  mkdir -p "${ambient_lima_home}"
  local log="${WORK_ROOT}/${label}.log"
  if ! env \
    SUBSTRATE_LIMA_VM_NAME="named-vm" \
    LIMA_VM_NAME="ambient-vm" \
    LIMA_HOME="${ambient_lima_home}" \
    "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
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
    assert_contains "${limactl_log}" 'copy .*world-service' "prod-copy should copy bundled world-service"
    assert_contains "${limactl_log}" 'copy .*substrate-gateway' "prod-copy should copy bundled substrate-gateway"
    assert_limactl_fragment_env \
      "${limactl_log}" \
      "${ACCOUNT_HOME}" \
      "${ACCOUNT_LIMA_HOME}" \
      "named-vm:/tmp/substrate-cli" \
      "prod-copy should launch lima-warm with the account-database HOME/LIMA_HOME"
    assert_limactl_fragment_not_ambient \
      "${limactl_log}" \
      "${HOME}" \
      "${ambient_lima_home}" \
      "named-vm:/tmp/substrate-cli" \
      "prod-copy warm launch"
  fi
  local built_world_service=0
  local built_gateway=0
  if [[ -d "${capture_dir}" ]] && grep -R "cargo build -p world-service" "${capture_dir}" >/dev/null 2>&1; then
    built_world_service=1
  fi
  if [[ -d "${capture_dir}" ]] && grep -R "cargo build -p substrate-gateway" "${capture_dir}" >/dev/null 2>&1; then
    built_gateway=1
  fi
  if [[ "${include_agent}" -eq 1 && ( "${built_world_service}" -eq 1 || "${built_gateway}" -eq 1 ) ]]; then
    fatal "prod-copy unexpectedly triggered in-guest build"
  fi
  if [[ "${include_agent}" -eq 0 && ( "${built_world_service}" -eq 0 || "${built_gateway}" -eq 0 ) ]]; then
    fatal "prod-build did not trigger the expected in-guest world-service/gateway builds"
  fi
  if [[ "${include_agent}" -eq 0 ]]; then
    assert_limactl_fragment_env \
      "${limactl_log}" \
      "${ACCOUNT_HOME}" \
      "${ACCOUNT_LIMA_HOME}" \
      "shell named-vm env BUILD_PROFILE=release bash" \
      "prod-build should launch lima-warm with the account-database HOME/LIMA_HOME"
    assert_limactl_fragment_not_ambient \
      "${limactl_log}" \
      "${HOME}" \
      "${ambient_lima_home}" \
      "shell named-vm env BUILD_PROFILE=release bash" \
      "prod-build warm launch"
  fi
  assert_limactl_fragment_env \
    "${limactl_log}" \
    "${ACCOUNT_HOME}" \
    "${ACCOUNT_LIMA_HOME}" \
    "shell named-vm test -x /usr/local/bin/substrate-world-service" \
    "${label} should verify world-service via the account-database Lima control root"
  assert_limactl_fragment_env \
    "${limactl_log}" \
    "${ACCOUNT_HOME}" \
    "${ACCOUNT_LIMA_HOME}" \
    "shell named-vm test -x /usr/local/bin/substrate-gateway" \
    "${label} should verify substrate-gateway via the account-database Lima control root"
  assert_limactl_fragment_not_ambient \
    "${limactl_log}" \
    "${HOME}" \
    "${ambient_lima_home}" \
    "shell named-vm test -x /usr/local/bin/substrate-world-service" \
    "${label}"
  assert_limactl_fragment_not_ambient \
    "${limactl_log}" \
    "${HOME}" \
    "${ambient_lima_home}" \
    "shell named-vm test -x /usr/local/bin/substrate-gateway" \
    "${label}"
  assert_not_contains "${limactl_log}" "ambient-vm" \
    "${label} should ignore ambient VM fallback during post-warm verification"
  local expected_commitment
  expected_commitment="$(authoritative_install_commitment "${prefix}")"
  assert_captured_copy_contains_literal \
    "${capture_dir}" \
    "substrate-world-service.service" \
    "Environment=\"SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${expected_commitment}\"" \
    "${label} should project the verified install bootstrap commitment into the rendered guest service"
  assert_captured_copy_contains_literal \
    "${capture_dir}" \
    "substrate-world-service.service" \
    "Environment=\"SUBSTRATE_LIMA_INSTANCE_NAME=named-vm\"" \
    "${label} should project the verified Lima instance name into the rendered guest service"
  assert_captured_copy_contains_literal \
    "${capture_dir}" \
    "substrate-world-service.service" \
    "Environment=\"SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT=${ACCOUNT_LIMA_HOME}\"" \
    "${label} should project the authoritative Lima control root into the rendered guest service"
  assert_captured_copy_contains_literal \
    "${capture_dir}" \
    "substrate-world-service.service" \
    "Environment=\"SUBSTRATE_LIMA_HOST_SOCKET=${prefix}/sock/agent.sock\"" \
    "${label} should project the verified host socket into the rendered guest service"
  assert_captured_copy_contains_literal \
    "${capture_dir}" \
    "substrate-world-service.service" \
    "Environment=\"SUBSTRATE_LIMA_GUEST_SOCKET=/run/substrate.sock\"" \
    "${label} should project the canonical guest socket into the rendered guest service"
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
  local prefix="${WORK_ROOT}/${label}-prefix"
  local carrier="fixture-carrier"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! env \
    SUBSTRATE_HOME="${prefix}" \
    SUBSTRATE_ROOT="${prefix}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${carrier}" \
    SUBSTRATE_LIMA_VM_NAME="named-vm" \
    LIMA_VM_NAME="ambient-vm" \
    "${stage_dir}/scripts/mac/lima-warm.sh" \
      --install-prefix "${prefix}" \
      --install-bootstrap-context-v1 "${carrier}" \
      "${stage_dir}" >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "lima warm stub failed for dev scenario"
  fi
  local capture_dir="${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"
  if [[ ! -d "${capture_dir}" ]] || ! grep -R "cargo build -p world-service" "${capture_dir}" >/dev/null 2>&1; then
    fatal "dev scenario did not trigger in-guest world-service build"
  fi
  if [[ ! -d "${capture_dir}" ]] || ! grep -R "cargo build -p substrate-gateway" "${capture_dir}" >/dev/null 2>&1; then
    fatal "dev scenario did not trigger in-guest substrate-gateway build"
  fi
  assert_contains "${SUBSTRATE_TEST_LIMACTL_LOG}" "shell named-vm env BUILD_PROFILE=release bash" \
    "dev build should address the declared VM instead of any ambient fallback"
  assert_not_contains "${SUBSTRATE_TEST_LIMACTL_LOG}" "ambient-vm" \
    "dev build should ignore ambient VM fallback"
  info "Scenario ${label} complete:"
  info "  host build log: ${build_log}"
  info "  lima stub log: ${log}"
  info "  limactl log: ${SUBSTRATE_TEST_LIMACTL_LOG}"
  info "  capture dir: ${capture_dir}"
}

run_dev_call_section_scenario() {
  local label="dev-call-section"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  write_stub_cargo

  local dev_repo
  dev_repo="$(prepare_dev_repo_fixture 0)"
  local warm_log="${WORK_ROOT}/${label}-warm.log"
  export SUBSTRATE_TEST_WARM_LOG="${warm_log}"
  cat >"${dev_repo}/scripts/mac/lima-warm.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
log_path="${SUBSTRATE_TEST_WARM_LOG:?}"
printf '%s\n' "$*" >>"${log_path}"
prefix=""
carrier=""
project_path=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-prefix)
      prefix="${2:-}"
      shift 2
      ;;
    --install-bootstrap-context-v1)
      carrier="${2:-}"
      shift 2
      ;;
    -*)
      printf '[dev-call-section-stub][ERROR] unknown arg: %s\n' "$1" >&2
      exit 2
      ;;
    *)
      project_path="$1"
      shift
      break
      ;;
  esac
done
[[ -n "${prefix}" ]] || exit 11
[[ -n "${carrier}" ]] || exit 12
[[ "${prefix}" == "${SUBSTRATE_HOME:-}" ]] || exit 13
[[ "${prefix}" == "${SUBSTRATE_ROOT:-}" ]] || exit 14
[[ "${carrier}" == "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" ]] || exit 15
[[ -n "${project_path}" ]] || exit 16
exit 0
SCRIPT
  chmod +x "${dev_repo}/scripts/mac/lima-warm.sh"

  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${dev_repo}/scripts/substrate/dev-install-substrate.sh" \
    --prefix "${prefix}" \
    --profile release \
    --no-shims >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "dev-install-substrate failed for ${label}"
  fi

  assert_contains "${warm_log}" "--install-prefix ${prefix}" \
    "dev call-section should pass the selected install prefix"
  assert_contains "${warm_log}" "--install-bootstrap-context-v1 " \
    "dev call-section should pass the authenticated bootstrap carrier"
  unset SUBSTRATE_TEST_WARM_LOG
  info "Scenario ${label} complete:"
  info "  dev repo: ${dev_repo}"
  info "  install log: ${log}"
  info "  warm log: ${warm_log}"
}

prepare_dev_repo_fixture() {
  local include_linux_bundle="${1:-1}"
  local repo_dir="${WORK_ROOT}/dev-repo"
  mkdir -p \
    "${repo_dir}/scripts/substrate" \
    "${repo_dir}/scripts/mac/lima" \
    "${repo_dir}/scripts/mac" \
    "${repo_dir}/config" \
    "${repo_dir}/target/release"

  cp "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" "${repo_dir}/scripts/substrate/dev-install-substrate.sh"
  chmod +x "${repo_dir}/scripts/substrate/dev-install-substrate.sh"
  cp "${REPO_ROOT}/scripts/substrate/dev-uninstall-substrate.sh" "${repo_dir}/scripts/substrate/dev-uninstall-substrate.sh"
  chmod +x "${repo_dir}/scripts/substrate/dev-uninstall-substrate.sh"

  cat >"${repo_dir}/scripts/substrate/world-enable.sh" <<'SCRIPT'
#!/usr/bin/env bash
exit 0
SCRIPT
  chmod +x "${repo_dir}/scripts/substrate/world-enable.sh"

  cat >"${repo_dir}/scripts/substrate/install-substrate.sh" <<'SCRIPT'
#!/usr/bin/env bash
exit 0
SCRIPT
  chmod +x "${repo_dir}/scripts/substrate/install-substrate.sh"

  cat >"${repo_dir}/scripts/substrate/world-deps.yaml" <<'YAML'
version: 1
packages: []
YAML

  cp "${REPO_ROOT}/scripts/mac/lima-warm.sh" "${repo_dir}/scripts/mac/lima-warm.sh"
  chmod +x "${repo_dir}/scripts/mac/lima-warm.sh"
  cp "${REPO_ROOT}/scripts/mac/lima/substrate.yaml" "${repo_dir}/scripts/mac/lima/substrate.yaml"
  cp "${REPO_ROOT}/scripts/mac/lima/substrate-dev.yaml" "${repo_dir}/scripts/mac/lima/substrate-dev.yaml"
  mkdir -p "${repo_dir}/scripts/mac/lima/units"
  cp "${REPO_ROOT}/scripts/mac/lima/units/substrate-world-service.service.tmpl" \
    "${repo_dir}/scripts/mac/lima/units/substrate-world-service.service.tmpl"
  cp "${REPO_ROOT}/scripts/mac/lima/units/substrate-world-service.socket" \
    "${repo_dir}/scripts/mac/lima/units/substrate-world-service.socket"

  cat >"${repo_dir}/config/manager_hooks.yaml" <<'YAML'
version: 1
tools: []
YAML

  if [[ "${include_linux_bundle}" -eq 1 ]]; then
    mkdir -p "${repo_dir}/bin/linux"
    printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${repo_dir}/bin/linux/substrate"
    chmod +x "${repo_dir}/bin/linux/substrate"
    printf '%s\n' "${SUBSTRATE_TEST_FILE_SENTINEL:-ELF-STUB}" >"${repo_dir}/bin/linux/world-agent"
    chmod +x "${repo_dir}/bin/linux/world-agent"
  else
    cat >"${repo_dir}/Cargo.toml" <<'TOML'
[workspace]
members = []
resolver = "2"
TOML
    cat >"${repo_dir}/Cargo.lock" <<'LOCK'
# This file is automatically @generated by Cargo.
version = 4
LOCK
  fi

  printf '%s\n' "${repo_dir}"
}

run_dev_runtime_bundle_scenario() {
  local label="dev-runtime-bundle"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  write_stub_cargo

  local dev_repo
  dev_repo="$(prepare_dev_repo_fixture 1)"
  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${dev_repo}/scripts/substrate/dev-install-substrate.sh" \
    --prefix "${prefix}" \
    --profile release \
    --no-world \
    --no-shims >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "dev-install-substrate failed for ${label}"
  fi

  [[ -L "${prefix}/scripts/substrate/world-enable.sh" ]] || fatal "expected prefix world-enable helper symlink"
  [[ -L "${prefix}/scripts/substrate/install-substrate.sh" ]] || fatal "expected prefix install helper symlink"
  [[ -L "${prefix}/scripts/substrate/world-deps.yaml" ]] || fatal "expected prefix world-deps symlink"
  [[ -L "${prefix}/scripts/mac/lima-warm.sh" ]] || fatal "expected prefix lima-warm symlink"
  [[ -L "${prefix}/scripts/mac/lima/substrate.yaml" ]] || fatal "expected prefix lima profile symlink"
  [[ -L "${prefix}/scripts/mac/lima/substrate-dev.yaml" ]] || fatal "expected prefix dev lima profile symlink"
  [[ -L "${prefix}/bin/linux/substrate" ]] || fatal "expected prefix linux substrate symlink"
  [[ -L "${prefix}/bin/linux/world-agent" ]] || fatal "expected prefix linux world-agent symlink"

  if [[ -e "${dev_repo}/target/scripts/substrate/world-enable.sh" ]]; then
    fatal "legacy target helper bridge should not remain after dev install"
  fi

  local warm_log="${WORK_ROOT}/${label}-warm.log"
  if ! (cd "${prefix}" && "${prefix}/scripts/mac/lima-warm.sh" "${prefix}") >"${warm_log}" 2>&1; then
    cat "${warm_log}" >&2 || true
    fatal "staged lima-warm failed for ${label}"
  fi

  mkdir -p "${prefix}/scripts/mac/lima"
  cat >"${prefix}/scripts/mac/lima/user-managed.txt" <<'TXT'
keep me
TXT

  local uninstall_log="${WORK_ROOT}/${label}-uninstall.log"
  if ! "${dev_repo}/scripts/substrate/dev-uninstall-substrate.sh" --prefix "${prefix}" >"${uninstall_log}" 2>&1; then
    cat "${uninstall_log}" >&2 || true
    fatal "dev-uninstall-substrate failed for ${label}"
  fi

  [[ ! -e "${prefix}/scripts/mac/lima/substrate.yaml" ]] || fatal "expected staged lima profile to be removed"
  [[ ! -e "${prefix}/scripts/mac/lima/substrate-dev.yaml" ]] || fatal "expected staged dev lima profile to be removed"
  [[ ! -e "${prefix}/scripts/mac/lima-warm.sh" ]] || fatal "expected staged lima-warm to be removed"
  [[ -f "${prefix}/scripts/mac/lima/user-managed.txt" ]] || fatal "expected user-managed file to survive uninstall"

  info "Scenario ${label} complete:"
  info "  dev repo: ${dev_repo}"
  info "  install log: ${log}"
  info "  warm log: ${warm_log}"
  info "  uninstall log: ${uninstall_log}"
}

run_dev_runtime_bundle_self_contained_scenario() {
  local label="dev-runtime-bundle-self-contained"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  write_stub_cargo

  local dev_repo
  dev_repo="$(prepare_dev_repo_fixture 0)"
  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${dev_repo}/scripts/substrate/dev-install-substrate.sh" \
    --prefix "${prefix}" \
    --profile release \
    --no-shims >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "dev-install-substrate failed for ${label}"
  fi

  [[ -L "${prefix}/scripts/substrate/world-enable.sh" ]] || fatal "expected prefix world-enable helper symlink"
  [[ -L "${prefix}/scripts/substrate/install-substrate.sh" ]] || fatal "expected prefix install helper symlink"
  [[ -L "${prefix}/scripts/substrate/world-deps.yaml" ]] || fatal "expected prefix world-deps symlink"
  [[ -L "${prefix}/scripts/mac/lima-warm.sh" ]] || fatal "expected prefix lima-warm symlink"
  [[ -f "${prefix}/bin/linux/substrate" && ! -L "${prefix}/bin/linux/substrate" ]] || fatal "expected copied prefix linux substrate binary"
  [[ -f "${prefix}/bin/linux/world-agent" && ! -L "${prefix}/bin/linux/world-agent" ]] || fatal "expected copied prefix linux world-agent binary"
  assert_contains "${SUBSTRATE_TEST_LIMACTL_LOG}" "copy substrate:/usr/local/bin/substrate ${prefix}/bin/linux/substrate" \
    "mac dev-install should cache the Linux substrate CLI into the prefix bundle"
  assert_contains "${SUBSTRATE_TEST_LIMACTL_LOG}" "copy substrate:/usr/local/bin/substrate-world-agent ${prefix}/bin/linux/world-agent" \
    "mac dev-install should cache the Linux world-agent into the prefix bundle"
  local manifest="${prefix}/.dev-install-managed/mac-linux-binaries.txt"
  [[ -f "${manifest}" ]] || fatal "expected managed mac Linux binary manifest"
  assert_contains "${manifest}" "${prefix}/bin/linux/substrate" "manifest should record cached substrate CLI"
  assert_contains "${manifest}" "${prefix}/bin/linux/world-agent" "manifest should record cached world-agent"
  assert_not_contains "${log}" "World has been disabled" "self-contained dev install should keep world enabled"

  : >"${SUBSTRATE_TEST_LIMACTL_LOG}"
  rm -rf "${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"
  mkdir -p "${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}"

  local warm_log="${WORK_ROOT}/${label}-warm.log"
  if ! (cd "${prefix}" && "${prefix}/scripts/mac/lima-warm.sh" "${prefix}") >"${warm_log}" 2>&1; then
    cat "${warm_log}" >&2 || true
    fatal "staged lima-warm failed for ${label}"
  fi
  assert_contains "${warm_log}" "Installing Linux substrate CLI from ${prefix}/bin/linux/substrate" \
    "staged lima-warm should reuse cached Linux substrate CLI"
  assert_contains "${warm_log}" "Installing Linux world-agent from ${prefix}/bin/linux/world-agent" \
    "staged lima-warm should reuse cached world-agent"
  assert_not_contains "${warm_log}" "does not contain Cargo sources" \
    "staged lima-warm should not require Cargo sources once binaries are cached"
  if [[ -d "${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}" ]] && grep -R "cargo build -p world-agent" "${SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR}" >/dev/null 2>&1; then
    fatal "staged lima-warm unexpectedly triggered an in-guest world-agent build"
  fi

  mkdir -p "${prefix}/bin/linux"
  cat >"${prefix}/bin/linux/user-managed.txt" <<'TXT'
keep me
TXT

  local uninstall_log="${WORK_ROOT}/${label}-uninstall.log"
  if ! "${dev_repo}/scripts/substrate/dev-uninstall-substrate.sh" --prefix "${prefix}" >"${uninstall_log}" 2>&1; then
    cat "${uninstall_log}" >&2 || true
    fatal "dev-uninstall-substrate failed for ${label}"
  fi

  [[ ! -e "${prefix}/bin/linux/substrate" ]] || fatal "expected cached Linux substrate binary to be removed"
  [[ ! -e "${prefix}/bin/linux/world-agent" ]] || fatal "expected cached Linux world-agent to be removed"
  [[ ! -e "${manifest}" ]] || fatal "expected managed binary manifest to be removed"
  [[ -f "${prefix}/bin/linux/user-managed.txt" ]] || fatal "expected user-managed Linux bundle file to survive uninstall"
  assert_not_contains "${uninstall_log}" "Protected-path refusal class exit 5" \
    "self-contained uninstall should not report protected-path refusal"
  assert_not_contains "${uninstall_log}" "Preserving protected path" \
    "self-contained uninstall should not preserve protected paths"

  info "Scenario ${label} complete:"
  info "  dev repo: ${dev_repo}"
  info "  install log: ${log}"
  info "  warm log: ${warm_log}"
  info "  uninstall log: ${uninstall_log}"
}

run_dev_runtime_bundle_protected_path_conflicts_scenario() {
  local label="dev-runtime-bundle-protected-path-conflicts"
  info "Running scenario ${label}"
  setup_workspace "${label}"
  install_common_stubs
  write_stub_cargo

  local dev_repo
  dev_repo="$(prepare_dev_repo_fixture 0)"
  local prefix="${WORK_ROOT}/${label}-prefix"
  mkdir -p "${prefix}"
  local log="${WORK_ROOT}/${label}.log"
  if ! "${dev_repo}/scripts/substrate/dev-install-substrate.sh" \
    --prefix "${prefix}" \
    --profile release \
    --no-shims >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "dev-install-substrate failed for ${label}"
  fi

  local warm_log="${WORK_ROOT}/${label}-warm.log"
  if ! (cd "${prefix}" && "${prefix}/scripts/mac/lima-warm.sh" "${prefix}") >"${warm_log}" 2>&1; then
    cat "${warm_log}" >&2 || true
    fatal "staged lima-warm failed for ${label}"
  fi

  local managed_removed_target="${prefix}/scripts/substrate/world-enable.sh"
  [[ -L "${managed_removed_target}" ]] || fatal "expected managed target to exist before uninstall"

  local managed_file_target="${prefix}/scripts/mac/lima/substrate.yaml"
  local managed_symlink_target="${prefix}/scripts/substrate/install-substrate.sh"
  local unmanaged_link_source="${WORK_ROOT}/${label}-unmanaged-link-source.txt"
  printf '%s\n' 'keep me too' >"${unmanaged_link_source}"

  rm -f "${managed_file_target}"
  cat >"${managed_file_target}" <<'TXT'
user-managed file at managed target
TXT

  rm -f "${managed_symlink_target}"
  ln -s "${unmanaged_link_source}" "${managed_symlink_target}"

  local uninstall_log="${WORK_ROOT}/${label}-uninstall.log"
  local uninstall_status=0
  if "${dev_repo}/scripts/substrate/dev-uninstall-substrate.sh" --prefix "${prefix}" >"${uninstall_log}" 2>&1; then
    fatal "dev-uninstall-substrate unexpectedly succeeded for ${label}; protected targets should exit 5"
  else
    uninstall_status=$?
  fi
  if [[ "${uninstall_status}" -ne 5 ]]; then
    cat "${uninstall_log}" >&2 || true
    fatal "dev-uninstall-substrate returned ${uninstall_status} instead of 5 for ${label}"
  fi

  [[ ! -e "${managed_removed_target}" ]] || fatal "expected managed target to be removed"
  [[ -f "${managed_file_target}" ]] || fatal "expected protected regular file to survive uninstall"
  [[ -L "${managed_symlink_target}" ]] || fatal "expected protected symlink to survive uninstall"
  [[ ! -e "${prefix}/bin/linux/substrate" ]] || fatal "expected cached Linux substrate binary to be removed"
  [[ ! -e "${prefix}/bin/linux/world-agent" ]] || fatal "expected cached Linux world-agent to be removed"
  assert_contains_literal "${uninstall_log}" "Preserving protected path ${managed_file_target}" \
    "uninstall log should report protected regular file"
  assert_contains_literal "${uninstall_log}" "Preserving protected path ${managed_symlink_target}" \
    "uninstall log should report protected symlink"
  assert_not_contains "${uninstall_log}" "Preserving protected path ${prefix}/scripts/substrate/world-enable.sh" \
    "uninstall log should only report explicit conflicting targets"
  assert_contains "${uninstall_log}" "Removing managed symlink ${managed_removed_target}" \
    "uninstall should still remove at least one managed target"

  info "Scenario ${label} complete:"
  info "  dev repo: ${dev_repo}"
  info "  install log: ${log}"
  info "  warm log: ${warm_log}"
  info "  uninstall log: ${uninstall_log}"
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
  assert_contains "${log}" "Syncing world dependencies via 'substrate world deps current sync'" \
    "sync-deps should announce world deps current sync"
  assert_contains "${substrate_log}" "world deps current sync" \
    "sync-deps should invoke world deps current sync"
  unset SUBSTRATE_TEST_SUBSTRATE_LOG
  info "Scenario ${label} complete:"
  info "  install log: ${log}"
  info "  substrate log: ${substrate_log}"
}

run_sync_deps_remediation_scenario() {
  local label="sync-deps-remediation"
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
  export SUBSTRATE_TEST_SYNC_DEPS_EXIT_4=1
  if ! "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --version "${FAKE_VERSION}" \
    --prefix "${prefix}" \
    --artifact-dir "${artifact_dir}" \
    --no-shims \
    --sync-deps >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "install-substrate failed for ${label}"
  fi
  assert_contains "${substrate_log}" "world deps current sync" \
    "sync-deps remediation should still invoke world deps current sync"
  assert_contains "${log}" "substrate world enable --provision-deps" \
    "sync-deps remediation should print the provision-deps remediation"
  assert_contains "${log}" "world deps sync failed; run 'substrate world deps current sync' later to finish provisioning." \
    "sync-deps remediation should still print the generic follow-up warning"
  unset SUBSTRATE_TEST_SYNC_DEPS_EXIT_4
  unset SUBSTRATE_TEST_SUBSTRATE_LOG
  info "Scenario ${label} complete:"
  info "  install log: ${log}"
  info "  substrate log: ${substrate_log}"
}

run_sync_deps_generic_failure_scenario() {
  local label="sync-deps-generic-failure"
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
  export SUBSTRATE_TEST_SYNC_DEPS_EXIT_CODE=7
  if ! "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --version "${FAKE_VERSION}" \
    --prefix "${prefix}" \
    --artifact-dir "${artifact_dir}" \
    --no-shims \
    --sync-deps >"${log}" 2>&1; then
    cat "${log}" >&2 || true
    fatal "install-substrate failed for ${label}"
  fi
  assert_contains "${substrate_log}" "world deps current sync" \
    "sync-deps generic failure should still invoke world deps current sync"
  assert_contains "${log}" "world deps sync failed; run 'substrate world deps current sync' later to finish provisioning." \
    "sync-deps generic failure should print the generic follow-up warning"
  assert_not_contains "${log}" "substrate world enable --provision-deps" \
    "sync-deps generic failure should not print the exit-4 remediation"
  unset SUBSTRATE_TEST_SYNC_DEPS_EXIT_CODE
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
    dev-call-section)
      run_dev_call_section_scenario
      ;;
    dev-runtime-bundle)
      run_dev_runtime_bundle_scenario
      ;;
    dev-runtime-bundle-self-contained)
      run_dev_runtime_bundle_self_contained_scenario
      ;;
    dev-runtime-bundle-protected-path-conflicts)
      run_dev_runtime_bundle_protected_path_conflicts_scenario
      ;;
    sync-deps)
      run_sync_deps_scenario
      ;;
    sync-deps-remediation)
      run_sync_deps_remediation_scenario
      ;;
    sync-deps-generic-failure)
      run_sync_deps_generic_failure_scenario
      ;;
    cleanup-guidance)
      run_cleanup_guidance
      ;;
    all)
      run_prod_scenario "prod-copy" 1
      run_prod_scenario "prod-build" 0
      run_dev_scenario
      run_dev_call_section_scenario
      run_dev_runtime_bundle_scenario
      run_dev_runtime_bundle_self_contained_scenario
      run_dev_runtime_bundle_protected_path_conflicts_scenario
      run_sync_deps_scenario
      run_sync_deps_remediation_scenario
      run_sync_deps_generic_failure_scenario
      run_cleanup_guidance
      ;;
    *)
      fatal "unsupported scenario: ${SCENARIO}"
      ;;
  esac
}

run_selected
