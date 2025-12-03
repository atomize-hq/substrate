#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

log() {
  printf '[install-smoke] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

usage() {
  cat <<'USAGE' >&2
Usage: tests/installers/install_smoke.sh --scenario <prod|prod-no-world|dev> [--keep-root]

Scenarios:
  prod           Production installer with world provisioning.
  prod-no-world  Production installer with --no-world.
  dev            Dev installer (builds binaries + runs linux/world-provision.sh).

The harness stubs privileged/systemd commands and uses fake release bundles so
it never touches the host. It verifies config/manifest output plus socket
activation behavior without mutating the real system.
USAGE
}

record_skip() {
  local reason="$1"
  if [[ -n "${SKIP_LOG:-}" ]]; then
    printf 'skipped: %s\n' "${reason}" > "${SKIP_LOG}"
  fi
  log "Skipping scenario ${SCENARIO}: ${reason}"
  if [[ -n "${SKIP_LOG:-}" ]]; then
    log "Skip details written to ${SKIP_LOG}"
  fi
  exit 0
}

maybe_skip_platform() {
  local uname_s
  uname_s="$(uname -s 2>/dev/null || true)"
  if [[ "${uname_s}" != "Linux" ]]; then
    record_skip "non-Linux platform (${uname_s:-unknown})"
  fi
  if ! command -v systemctl >/dev/null 2>&1; then
    record_skip "systemctl not available (systemd not detected)"
  fi
  local init_comm
  init_comm="$(cat /proc/1/comm 2>/dev/null || true)"
  if [[ "${init_comm}" != "systemd" ]]; then
    record_skip "init system is '${init_comm:-unknown}', requires systemd"
  fi
}

SCENARIO="prod"
SCENARIO_KIND="prod"
SCENARIO_WORLD_ENABLED=1
KEEP_ROOT=0
FAKE_VERSION="${FAKE_VERSION:-0.0.0-test}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      [[ $# -lt 2 ]] && fatal "Missing value for --scenario"
      SCENARIO="$2"
      shift 2
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
      fatal "Unknown argument: $1"
      ;;
  esac
done

case "${SCENARIO}" in
  prod|default)
    SCENARIO="prod"
    SCENARIO_KIND="prod"
    SCENARIO_WORLD_ENABLED=1
    ;;
  prod-no-world|no-world)
    SCENARIO="prod-no-world"
    SCENARIO_KIND="prod"
    SCENARIO_WORLD_ENABLED=0
    ;;
  dev)
    SCENARIO="dev"
    SCENARIO_KIND="dev"
    SCENARIO_WORLD_ENABLED=1
    ;;
  *)
    usage
    fatal "Unsupported scenario: ${SCENARIO}"
    ;;
esac

WORK_ROOT="$(mktemp -d "/tmp/substrate-installer-${SCENARIO}.XXXXXX")"
PREFIX="${WORK_ROOT}/prefix"
ARTIFACT_DIR="${WORK_ROOT}/artifacts"
FAKE_ROOT="${WORK_ROOT}/fakeroot"
STUB_BIN="${WORK_ROOT}/stub-bin"
HOME_DIR="${WORK_ROOT}/home"
INSTALL_LOG="${WORK_ROOT}/install.log"
UNINSTALL_LOG="${WORK_ROOT}/uninstall.log"
SYSTEMCTL_LOG="${WORK_ROOT}/systemctl.current.log"
EXPECT_SOCKET_UNITS="${SUBSTRATE_INSTALLER_EXPECT_SOCKET:-0}"
if [[ "${EXPECT_SOCKET_UNITS}" -eq 1 ]]; then
  log "Socket unit enforcement enabled (SUBSTRATE_INSTALLER_EXPECT_SOCKET=1)."
fi
mkdir -p "${PREFIX}" "${ARTIFACT_DIR}" "${FAKE_ROOT}" "${STUB_BIN}" "${HOME_DIR}"
: > "${SYSTEMCTL_LOG}"
SKIP_LOG="${WORK_ROOT}/skip.log"
maybe_skip_platform

GROUP_OP_LOG="${WORK_ROOT}/group-ops.log"
LINGER_STATE_LOG="${WORK_ROOT}/linger.log"
: > "${GROUP_OP_LOG}"
: > "${LINGER_STATE_LOG}"
FAKE_USER="${SUBSTRATE_TEST_FAKE_USER:-substrate-smoke}"
export SUBSTRATE_TEST_GROUP_LOG="${GROUP_OP_LOG}"
export SUBSTRATE_TEST_LINGER_LOG="${LINGER_STATE_LOG}"
export SUBSTRATE_TEST_PRIMARY_USER="${FAKE_USER}"
export SUBSTRATE_TEST_USER_GROUPS="${SUBSTRATE_TEST_USER_GROUPS_OVERRIDE:-wheel docker}"
export SUBSTRATE_TEST_GROUP_EXISTS=0
export SUBSTRATE_TEST_LINGER_STATE="${SUBSTRATE_TEST_LINGER_STATE_OVERRIDE:-no}"

cleanup() {
  if [[ "${KEEP_ROOT}" -eq 0 ]]; then
    rm -rf "${WORK_ROOT}"
  else
    log "Preserving ${WORK_ROOT} for inspection (--keep-root set)."
  fi
}
trap cleanup EXIT

reset_systemctl_log() {
  : > "${SYSTEMCTL_LOG}"
}

capture_systemctl_log() {
  local phase="$1"
  local target="${WORK_ROOT}/systemctl-${phase}.log"
  if [[ -s "${SYSTEMCTL_LOG}" ]]; then
    cp "${SYSTEMCTL_LOG}" "${target}"
  else
    : > "${target}"
  fi
  reset_systemctl_log
  log "Captured systemctl log for ${phase}: ${target}"
  printf '%s\n' "${target}"
}

assert_systemctl_log() {
  local phase="$1"
  local log_path="$2"
  local require_activity="${3:-1}"
  if [[ ! -s "${log_path}" ]]; then
    if [[ "${require_activity}" -eq 0 ]]; then
      log "No systemctl activity recorded for ${phase} (allowed for this scenario)."
      return
    fi
    fatal "Expected systemctl activity during ${phase}, but log is empty: ${log_path}"
  fi
  local total
  total="$(wc -l < "${log_path}")"
  local socket_hits
  socket_hits="$(grep -c 'substrate-world-agent\.socket' "${log_path}" || true)"
  log "[${phase}] systemctl calls: ${total}; socket entries: ${socket_hits}"
  if [[ "${EXPECT_SOCKET_UNITS}" -eq 1 && "${socket_hits}" -eq 0 ]]; then
    fatal "[${phase}] expected socket unit commands (SUBSTRATE_INSTALLER_EXPECT_SOCKET=1), but none recorded. See ${log_path}."
  fi
}

compute_sha256() {
  local path="$1"
  python3 - <<'PY' "${path}"
import hashlib, pathlib, sys
path = pathlib.Path(sys.argv[1])
h = hashlib.sha256()
with path.open('rb') as f:
    for chunk in iter(lambda: f.read(1024 * 1024), b''):
        h.update(chunk)
print(h.hexdigest())
PY
}

write_stub_command() {
  local name="$1"
  cat >"${STUB_BIN}/${name}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${STUB_BIN}/${name}"
}

write_stub_jq() {
  cat >"${STUB_BIN}/jq" <<'EOF'
#!/usr/bin/env bash
cat
EOF
  chmod +x "${STUB_BIN}/jq"
}

write_stub_sudo() {
  cat >"${STUB_BIN}/sudo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
FAKE_ROOT="${FAKE_ROOT:-}"
SYSTEMCTL_LOG="${SUBSTRATE_TEST_SYSTEMCTL_LOG:-}"
if [[ $# -lt 1 ]]; then
  exit 0
fi
cmd="$1"
shift || true
while [[ "${cmd}" == -* && $# -gt 0 ]]; do
  cmd="$1"
  shift || true
done
if [[ "${cmd}" == -* ]]; then
  exit 0
fi

rewrite_dest_arg() {
  local -n src_args=$1
  local rewritten=()
  local last_index=$((${#src_args[@]} - 1))
  for i in "${!src_args[@]}"; do
    local val="${src_args[$i]}"
    if [[ "${i}" -eq "${last_index}" && "${val}" == /* && -n "${FAKE_ROOT}" ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  src_args=("${rewritten[@]}")
}

rewrite_all_paths() {
  local -n src_args=$1
  local rewritten=()
  for val in "${src_args[@]}"; do
    if [[ "${val}" == /* && -n "${FAKE_ROOT}" ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  src_args=("${rewritten[@]}")
}

log_systemctl() {
  if [[ -z "${SYSTEMCTL_LOG}" ]]; then
    return
  fi
  printf 'systemctl %s\n' "$*" >>"${SYSTEMCTL_LOG}"
}

case "${cmd}" in
  systemctl)
    log_systemctl "$@"
    exit 0
    ;;
  install|cp|mv|ln)
    args=("$@")
    rewrite_dest_arg args
    exec "${cmd}" "${args[@]}"
    ;;
  rm|mkdir|chmod|chown)
    args=("$@")
    rewrite_all_paths args
    exec "${cmd}" "${args[@]}"
    ;;
  *)
    args=("$@")
    exec "${cmd}" "${args[@]}"
    ;;
esac
EOF
  chmod +x "${STUB_BIN}/sudo"
}

write_stub_id() {
  cat >"${STUB_BIN}/id" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
primary="${SUBSTRATE_TEST_PRIMARY_USER:-substrate-smoke}"
groups="${SUBSTRATE_TEST_USER_GROUPS:-wheel docker}"
if [[ $# -eq 0 ]]; then
  printf '%s\n' "${primary}"
  exit 0
fi
case "$1" in
  -un)
    printf '%s\n' "${primary}"
    exit 0
    ;;
  -nG)
    printf '%s\n' "${groups}"
    exit 0
    ;;
esac
printf '%s\n' "${primary}"
exit 0
EOF
  chmod +x "${STUB_BIN}/id"
}

write_stub_getent() {
  cat >"${STUB_BIN}/getent" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ge 2 && "$1" == "group" ]]; then
  group="$2"
  entry="${SUBSTRATE_TEST_GROUP_ENTRY:-${group}:x:999:}"
  if [[ "${group}" == "substrate" ]]; then
    if [[ "${SUBSTRATE_TEST_GROUP_EXISTS:-0}" -eq 1 ]]; then
      printf '%s\n' "${entry}"
      exit 0
    fi
    exit 2
  fi
fi
exit 2
EOF
  chmod +x "${STUB_BIN}/getent"
}

write_stub_groupadd() {
  cat >"${STUB_BIN}/groupadd" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'groupadd %s\n' "$*" >>"${log}"
fi
exit 0
EOF
  chmod +x "${STUB_BIN}/groupadd"
}

write_stub_usermod() {
  cat >"${STUB_BIN}/usermod" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'usermod %s\n' "$*" >>"${log}"
fi
exit 0
EOF
  chmod +x "${STUB_BIN}/usermod"
}

write_stub_loginctl() {
  cat >"${STUB_BIN}/loginctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_LINGER_LOG:-}"
state="${SUBSTRATE_TEST_LINGER_STATE:-no}"
if [[ $# -ge 1 && "$1" == "show-user" ]]; then
  user="${2:-unknown}"
  if [[ "${3:-}" == "-p" && "${4:-}" == "Linger" ]]; then
    if [[ -n "${log}" ]]; then
      printf '%s %s\n' "${user}" "${state}" >>"${log}"
    fi
    printf 'Linger=%s\n' "${state}"
    exit 0
  fi
fi
if [[ $# -ge 1 && "$1" == "enable-linger" ]]; then
  user="${2:-unknown}"
  if [[ -n "${log}" ]]; then
    printf '%s requested-enable\n' "${user}" >>"${log}"
  fi
  exit 0
fi
exit 0
EOF
  chmod +x "${STUB_BIN}/loginctl"
}

write_stub_cargo() {
  cat >"${STUB_BIN}/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# -lt 1 ]]; then
  exit 0
fi
cmd="$1"
shift || true
if [[ "${cmd}" != "build" ]]; then
  exit 0
fi
profile="debug"
bins=()
packages=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      profile="release"
      shift
      ;;
    --bin)
      if [[ $# -ge 2 ]]; then
        bins+=("$2")
        shift 2
      else
        shift
      fi
      ;;
    -p|--package)
      if [[ $# -ge 2 ]]; then
        packages+=("$2")
        shift 2
      else
        shift
      fi
      ;;
    *)
      shift
      ;;
  esac
  done
if [[ ${#bins[@]} -eq 0 && ${#packages[@]} -eq 0 ]]; then
  bins=(substrate)
fi
target_root="${SUBSTRATE_TEST_CARGO_TARGET_ROOT:-}"
if [[ -z "${target_root}" ]]; then
  exit 0
fi
make_binary() {
  local name="$1"
  local path="${target_root}/${profile}/${name}"
  mkdir -p "$(dirname "${path}")"
  cat >"${path}" <<'BIN'
#!/usr/bin/env bash
exit 0
BIN
  chmod +x "${path}"
}
for bin in "${bins[@]}"; do
  make_binary "${bin}"
done
for pkg in "${packages[@]}"; do
  if [[ "${pkg}" == "world-agent" ]]; then
    make_binary "world-agent"
    make_binary "substrate-world-agent"
  fi
done
for extra in substrate-forwarder host-proxy; do
  make_binary "${extra}"
done
exit 0
EOF
  chmod +x "${STUB_BIN}/cargo"
}

prepare_stub_bin() {
  write_stub_sudo
  write_stub_jq
  write_stub_id
  write_stub_getent
  write_stub_groupadd
  write_stub_usermod
  write_stub_loginctl
  write_stub_cargo
  for cmd in curl fusermount fuse-overlayfs ip nft systemctl limactl pkill pgrep; do
    write_stub_command "${cmd}"
  done
}

detect_bundle_label() {
  local arch
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64) printf 'linux_x86_64' ;;
    aarch64|arm64) printf 'linux_aarch64' ;;
    *) fatal "Unsupported architecture for harness: ${arch}" ;;
  esac
}

write_stub_binary() {
  local path="$1"
  cat >"${path}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

require_manifest() {
  local root="${SUBSTRATE_ROOT:-}"
  if [[ -z "${root}" ]]; then
    root="$(cd "$(dirname "$0")/.." && pwd)"
  fi
  local latest
  latest="$(ls -1 "${root}/versions" 2>/dev/null | sort | tail -n1 || true)"
  local manifest="${latest:+${root}/versions/${latest}/config/manager_hooks.yaml}"
  if [[ -z "${manifest}" || ! -f "${manifest}" ]]; then
    echo '{"error":"missing manager manifest"}' >&2
    exit 1
  fi
}

cmd="${1:-}"
shift || true

case "${cmd}" in
  --shim-deploy)
    if [[ -n "${SUBSTRATE_ROOT:-}" ]]; then
      mkdir -p "${SUBSTRATE_ROOT}/shims"
    fi
    exit 0
    ;;
  world)
    sub="${1:-}"
    shift || true
    case "${sub}" in
      doctor)
        require_manifest
        printf '{"status":"ok"}\n'
        exit 0
        ;;
      deps)
        exit 0
        ;;
    esac
    ;;
  health)
    require_manifest
    printf '{"health":"ok"}\n'
    exit 0
    ;;
esac

exit 0
EOF
  chmod +x "${path}"
}

build_fake_release() {
  local label
  label="$(detect_bundle_label)"
  local bundle_root="${WORK_ROOT}/bundle/${label}"
  mkdir -p "${bundle_root}/bin" "${bundle_root}/config"

  cp "${REPO_ROOT}/config/manager_hooks.yaml" "${bundle_root}/config/manager_hooks.yaml"
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${bundle_root}/config/world-deps.yaml"

  write_stub_binary "${bundle_root}/bin/substrate"
  write_stub_binary "${bundle_root}/bin/host-proxy"
  write_stub_binary "${bundle_root}/bin/world-agent"

  local archive="substrate-v${FAKE_VERSION}-${label}.tar.gz"
  tar -czf "${ARTIFACT_DIR}/${archive}" -C "${WORK_ROOT}/bundle" "${label}"

  local checksum
  checksum="$(compute_sha256 "${ARTIFACT_DIR}/${archive}")"
  printf '%s  %s\n' "${checksum}" "${archive}" >"${ARTIFACT_DIR}/SHA256SUMS"
}

assert_install_config() {
  local config="${PREFIX}/config.toml"
  local expected_flag="true"
  if [[ "${SCENARIO_WORLD_ENABLED}" -eq 0 ]]; then
    expected_flag="false"
  fi

  if [[ ! -f "${config}" ]]; then
    if [[ -f "${PREFIX}/config.json" ]]; then
      fatal "expected ${config} but found legacy config.json; installer should emit TOML"
    fi
    fatal "install config missing after install: ${config}"
  fi

  python3 - <<'PY' "${config}" "${expected_flag}"
import sys
from pathlib import Path

path = Path(sys.argv[1])
expected_enabled = sys.argv[2].lower() == "true"
body = path.read_text()
world_enabled = None
anchor_mode = None
anchor_path = None
root_mode = None
root_path = None
caged = None
section = None


def parse_bool(raw):
    val = raw.strip().lower()
    if val in ("true", "false"):
        return val == "true"
    raise SystemExit(f"invalid boolean value: {raw}")


def parse_string(raw):
    val = raw.strip()
    if val.startswith('"') and val.endswith('"') and len(val) >= 2:
        return val[1:-1]
    return val


for raw in body.splitlines():
    line = raw.split("#", 1)[0].strip()
    if not line:
        continue
    if line.startswith("[") and line.endswith("]"):
        section = line[1:-1].strip()
        continue
    if "=" not in line or section is None:
        continue
    key, value = line.split("=", 1)
    key = key.strip()
    value = value.strip()
    if section == "install" and key == "world_enabled":
        world_enabled = parse_bool(value)
    elif section == "world":
        if key == "anchor_mode":
            anchor_mode = parse_string(value)
        elif key == "anchor_path":
            anchor_path = parse_string(value)
        elif key == "root_mode":
            root_mode = parse_string(value)
        elif key == "root_path":
            root_path = parse_string(value)
        elif key == "caged":
            caged = parse_bool(value)

if world_enabled is None:
    raise SystemExit("world_enabled missing under [install]")
if world_enabled != expected_enabled:
    raise SystemExit(f"world_enabled={world_enabled} (expected {expected_enabled})")
if anchor_mode is None:
    raise SystemExit("world.anchor_mode missing under [world]")
if anchor_mode != "project":
    raise SystemExit(f"world.anchor_mode={anchor_mode} (expected project)")
if anchor_path is None:
    raise SystemExit("world.anchor_path missing under [world]")
if anchor_path != "":
    raise SystemExit(f"world.anchor_path={anchor_path!r} (expected empty string)")
if root_mode is None:
    raise SystemExit("world.root_mode missing under [world] (expected backward compatibility)")
if root_mode != anchor_mode:
    raise SystemExit(f"world.root_mode={root_mode} (expected {anchor_mode})")
if root_path is None:
    raise SystemExit("world.root_path missing under [world] (expected backward compatibility)")
if root_path != anchor_path:
    raise SystemExit(f"world.root_path={root_path!r} (expected {anchor_path!r})")
if caged is None:
    raise SystemExit("world.caged missing under [world]")
if caged is not True:
    raise SystemExit(f"world.caged={caged} (expected true)")
PY

  log "Verified install config at ${config} (world_enabled=${expected_flag}; anchor_mode=project anchor_path=\"\" root_mode=project root_path=\"\" caged=true)"
}

assert_manifest_present() {
  local manifest="${PREFIX}/versions/${FAKE_VERSION}/config/manager_hooks.yaml"
  if [[ ! -f "${manifest}" ]]; then
    fatal "manager manifest missing after install: ${manifest}"
  fi
  log "Verified manager manifest at ${manifest}"
}

run_health_smoke() {
  local substrate_bin="${PREFIX}/bin/substrate"
  local output
  if ! output="$(SUBSTRATE_ROOT="${PREFIX}" PATH="${PREFIX}/bin:${PATH}" "${substrate_bin}" health --json)"; then
    fatal "substrate health failed"
  fi
  python3 - <<'PY' "${output}"
import json, sys
try:
    json.loads(sys.argv[1])
except json.JSONDecodeError as err:
    sys.stderr.write(f"health JSON parse error: {err}\n")
    sys.exit(1)
PY
  log "Health check succeeded: ${output}"
}

run_prod_install() {
  local args=("--prefix" "${PREFIX}" "--version" "${FAKE_VERSION}" "--artifact-dir" "${ARTIFACT_DIR}")
  if [[ "${SCENARIO_WORLD_ENABLED}" -eq 0 ]]; then
    args+=("--no-world")
  fi
  local harness_path="${STUB_BIN}:${PATH}"

  if ! HOME="${HOME_DIR}" \
    PATH="${harness_path}" \
    SHIM_ORIGINAL_PATH="${harness_path}" \
    FAKE_ROOT="${FAKE_ROOT}" \
    SUBSTRATE_TEST_SYSTEMCTL_LOG="${SYSTEMCTL_LOG}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${FAKE_USER}" \
    USER="${FAKE_USER}" \
    LOGNAME="${FAKE_USER}" \
    "${REPO_ROOT}/scripts/substrate/install-substrate.sh" "${args[@]}" \
    >"${INSTALL_LOG}" 2>&1; then
    cat "${INSTALL_LOG}" >&2
    fatal "Install script failed; see ${INSTALL_LOG}"
  fi
  cat "${INSTALL_LOG}"
}

run_prod_uninstall() {
  local harness_path="${STUB_BIN}:${PATH}"
  if ! HOME="${HOME_DIR}" \
    PATH="${harness_path}" \
    SHIM_ORIGINAL_PATH="${harness_path}" \
    FAKE_ROOT="${FAKE_ROOT}" \
    SUBSTRATE_TEST_SYSTEMCTL_LOG="${SYSTEMCTL_LOG}" \
    "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh" \
    >"${UNINSTALL_LOG}" 2>&1; then
    cat "${UNINSTALL_LOG}" >&2
    fatal "Uninstall script failed; see ${UNINSTALL_LOG}"
  fi
  cat "${UNINSTALL_LOG}"
}

run_dev_install() {
  local harness_path="${STUB_BIN}:${PATH}"
  local args=("--prefix" "${PREFIX}" "--version-label" "smoke-dev")
  if [[ "${SCENARIO_WORLD_ENABLED}" -eq 0 ]]; then
    args+=("--no-world")
  fi
  local dev_target_root="${REPO_ROOT}/target"
  local cargo_target_preexisting=0
  if [[ -d "${dev_target_root}" ]]; then
    cargo_target_preexisting=1
  fi

  if ! HOME="${HOME_DIR}" \
    PATH="${harness_path}" \
    SHIM_ORIGINAL_PATH="${harness_path}" \
    FAKE_ROOT="${FAKE_ROOT}" \
    SUBSTRATE_TEST_SYSTEMCTL_LOG="${SYSTEMCTL_LOG}" \
    SUBSTRATE_TEST_CARGO_TARGET_ROOT="${dev_target_root}" \
    USER="${FAKE_USER}" \
    LOGNAME="${FAKE_USER}" \
    "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" "${args[@]}" \
    >"${INSTALL_LOG}" 2>&1; then
    cat "${INSTALL_LOG}" >&2
    fatal "Dev install script failed; see ${INSTALL_LOG}"
  fi

  if [[ "${cargo_target_preexisting}" -eq 0 ]]; then
    rm -rf "${dev_target_root}"
  fi
  cat "${INSTALL_LOG}"
}

assert_config_init_hint_logged() {
  if [[ ! -f "${INSTALL_LOG}" ]]; then
    fatal "Installer output log missing; expected ${INSTALL_LOG}"
  fi
  if ! grep -Fq "substrate config init" "${INSTALL_LOG}"; then
    fatal "Installer output missing 'substrate config init' hint (see ${INSTALL_LOG})"
  fi
  log "Installer output references substrate config init hint (${INSTALL_LOG})"
}

prepare_stub_bin
reset_systemctl_log

if [[ "${SCENARIO_KIND}" == "prod" ]]; then
  build_fake_release
  run_prod_install
  install_systemctl_log="$(capture_systemctl_log install)"
  assert_config_init_hint_logged
  assert_install_config
  assert_manifest_present
  run_health_smoke
  if [[ "${SCENARIO_WORLD_ENABLED}" -eq 1 ]]; then
    assert_systemctl_log "install" "${install_systemctl_log}" 1
  else
    assert_systemctl_log "install" "${install_systemctl_log}" 0
  fi
  run_prod_uninstall
  uninstall_systemctl_log="$(capture_systemctl_log uninstall)"
  assert_systemctl_log "uninstall" "${uninstall_systemctl_log}" 1
elif [[ "${SCENARIO_KIND}" == "dev" ]]; then
  run_dev_install
  install_systemctl_log="$(capture_systemctl_log install)"
  assert_systemctl_log "install" "${install_systemctl_log}" 1
else
  fatal "Unsupported scenario kind: ${SCENARIO_KIND}"
fi

log "Scenario ${SCENARIO} completed using PREFIX=${PREFIX}"
log "Temp root: ${WORK_ROOT}"
