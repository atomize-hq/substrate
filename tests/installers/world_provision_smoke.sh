#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

PROFILE="${LP1_PROVISION_PROFILE:-lp1test}"
KEEP_ROOT=0

log() {
  printf '[world-provision-smoke] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

usage() {
  cat <<'USAGE' >&2
Usage: tests/installers/world_provision_smoke.sh [--profile <name>] [--keep-root]

Verifies scripts/linux/world-provision.sh writes the substrate socket unit with
SocketGroup=substrate, installs the Linux lifecycle executor plus its socket-
activated units, records group membership operations, emits linger guidance,
skips the gateway proof on a clean install, runs the proof when config/policy
make it eligible, and only reports the world-deps ACL bridge green when the real
runtime probe path is reachable. The harness stubs systemd and gateway commands
so it never touches the host.
USAGE
}

record_skip() {
  local reason="$1"
  log "Skipping: ${reason}"
  exit 0
}

maybe_skip_platform() {
  local uname_s
  uname_s="$(uname -s 2>/dev/null || true)"
  if [[ "${uname_s}" != "Linux" ]]; then
    record_skip "non-Linux platform (${uname_s:-unknown})"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -lt 2 ]] && fatal "Missing value for --profile"
      PROFILE="$2"
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

maybe_skip_platform

CURRENT_ACCOUNT="$(id -un)"
CURRENT_UID="$(id -u)"

WORK_ROOT="$(mktemp -d "/tmp/substrate-world-provision.XXXXXX")"
STUB_BIN="${WORK_ROOT}/stub-bin"
mkdir -p "${STUB_BIN}"

cleanup() {
  if [[ "${KEEP_ROOT}" -eq 0 ]]; then
    rm -rf "${WORK_ROOT}"
  else
    log "Preserving artifacts under ${WORK_ROOT} (--keep-root set)."
  fi
}
trap cleanup EXIT

write_stub_sudo() {
  cat >"${STUB_BIN}/sudo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
FAKE_ROOT="${FAKE_ROOT:-}"
SYSTEMCTL_LOG="${SUBSTRATE_TEST_SYSTEMCTL_LOG:-}"
TEST_STUB_BIN="${SUBSTRATE_TEST_STUB_BIN:-}"
TOOL_LOG="${SUBSTRATE_TEST_SUDO_TOOL_LOG:-}"
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
if [[ -n "${TOOL_LOG}" ]]; then
  printf 'outer:%s\n' "${cmd}" >>"${TOOL_LOG}"
fi

scrubbed=0
seen_path=0
seen_home=0
seen_user=0
seen_logname=0
if [[ "${cmd##*/}" == "env" ]]; then
  [[ "${1:-}" == "-i" ]] || exit 90
  scrubbed=1
  shift
  while [[ $# -gt 0 && "$1" == *=* ]]; do
    case "$1" in
      PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin)
        seen_path=1
        ;;
      HOME=/root)
        seen_home=1
        ;;
      USER=root)
        seen_user=1
        ;;
      LOGNAME=root)
        seen_logname=1
        ;;
      *)
        exit 91
        ;;
    esac
    shift
  done
  [[ $# -gt 0 ]] || exit 92
  cmd="$1"
  shift
fi
cmd_name="${cmd##*/}"
if [[ -n "${TOOL_LOG}" ]]; then
  printf '%s\n' "${cmd}" >>"${TOOL_LOG}"
fi
dispatch_cmd="${cmd}"
if [[ -n "${TEST_STUB_BIN}" && -x "${TEST_STUB_BIN}/${cmd_name}" ]]; then
  dispatch_cmd="${TEST_STUB_BIN}/${cmd_name}"
fi
if [[ "${scrubbed}" -eq 1 ]]; then
  [[ "${seen_path}" -eq 1 ]] || exit 93
  if [[ "${cmd_name}" != "true" ]]; then
    [[ "${seen_home}" -eq 1 && "${seen_user}" -eq 1 && "${seen_logname}" -eq 1 ]] || exit 94
  fi
fi

rewrite_dest_arg() {
  local rewritten=()
  local last_index=$((${#args[@]} - 1))
  for i in "${!args[@]}"; do
    local val="${args[$i]}"
    if [[ "${i}" -eq "${last_index}" && "${val}" == /* && -n "${FAKE_ROOT}" && "${val}" != "${FAKE_ROOT}"/* ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  args=("${rewritten[@]}")
}

rewrite_all_paths() {
  local rewritten=()
  for val in "${args[@]}"; do
    if [[ "${val}" == /* && -n "${FAKE_ROOT}" && "${val}" != "${FAKE_ROOT}"/* ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  args=("${rewritten[@]}")
}

log_systemctl() {
  if [[ -z "${SYSTEMCTL_LOG}" ]]; then
    return
  fi
  printf 'systemctl %s\n' "$*" >>"${SYSTEMCTL_LOG}"
}

case "${cmd_name}" in
  systemctl)
    log_systemctl "$@"
    exec "${dispatch_cmd}" "$@"
    ;;
  getfacl|test)
    args=("$@")
    rewrite_all_paths
    exec "${dispatch_cmd}" "${args[@]}"
    ;;
  install|cp|mv|ln)
    args=("$@")
    rewrite_dest_arg
    exec "${dispatch_cmd}" "${args[@]}"
    ;;
  rm|mkdir|chmod|chown|ls)
    args=("$@")
    rewrite_all_paths
    exec "${dispatch_cmd}" "${args[@]}"
    ;;
  substrate-apply-socket-acl)
    case "$*" in
      "--socket /run/substrate.sock substrate"|\
      "--directory-traverse /var/lib/substrate substrate"|\
      "--tree-readonly /var/lib/substrate/world-deps substrate")
        exit 0
        ;;
      *)
        exit 95
        ;;
    esac
    ;;
  *)
    args=("$@")
    exec "${dispatch_cmd}" "${args[@]}"
    ;;
esac
EOF
  chmod +x "${STUB_BIN}/sudo"
}

write_stub_systemctl() {
  cat >"${STUB_BIN}/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_SYSTEMCTL_LOG:-}"
fake_root="${FAKE_ROOT:-}"
if [[ -n "${log}" ]]; then
  printf 'systemctl %s\n' "$*" >>"${log}"
fi
if [[ $# -ge 2 && "$1" == "is-enabled" ]]; then
  printf 'disabled\n'
  exit 1
fi
if [[ $# -ge 2 && "$1" == "is-active" ]]; then
  printf 'inactive\n'
  exit 3
fi
if [[ $# -ge 2 && "$1" == "start" && "$2" == "substrate-world-service.socket" && -n "${fake_root}" ]]; then
  socket_path="${fake_root}/run/substrate.sock"
  mkdir -p "$(dirname "${socket_path}")"
  : >"${socket_path}"
  chmod 0660 "${socket_path}" 2>/dev/null || true
  chgrp substrate "${socket_path}" 2>/dev/null || true
fi
if [[ $# -ge 2 && "$1" == "start" && "$2" == "substrate-lifecycle-publisher-v1.socket" && -n "${fake_root}" ]]; then
  socket_path="${fake_root}/run/substrate-lifecycle-publisher-v1.sock"
  mkdir -p "$(dirname "${socket_path}")"
  : >"${socket_path}"
  chmod 0600 "${socket_path}" 2>/dev/null || true
fi
exit 0
EOF
  chmod +x "${STUB_BIN}/systemctl"
}

write_stub_id() {
  cat >"${STUB_BIN}/id" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
primary="${SUBSTRATE_TEST_PRIMARY_USER:-substrate-smoke}"
active_groups="${SUBSTRATE_TEST_ACTIVE_GROUPS:-wheel docker}"
account_groups_before="${SUBSTRATE_TEST_ACCOUNT_GROUPS_BEFORE:-wheel docker}"
account_groups_after="${SUBSTRATE_TEST_ACCOUNT_GROUPS_AFTER:-wheel docker substrate}"
usermod_state="${SUBSTRATE_TEST_USERMOD_STATE:-}"
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
    if [[ $# -ge 2 ]]; then
      if [[ -n "${usermod_state}" && -f "${usermod_state}" ]]; then
        printf '%s\n' "${account_groups_after}"
      else
        printf '%s\n' "${account_groups_before}"
      fi
    else
      printf '%s\n' "${active_groups}"
    fi
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
home_dir="${SUBSTRATE_TEST_HOME:-/tmp/substrate-smoke-home}"
if [[ $# -ge 2 && "$1" == "group" ]]; then
  group="$2"
  if [[ "${group}" == "substrate" ]]; then
    if [[ "${SUBSTRATE_TEST_GROUP_EXISTS:-0}" -eq 1 ]]; then
      printf 'substrate:x:1234:\n'
      exit 0
    fi
    exit 2
  fi
fi
if [[ $# -ge 2 && "$1" == "passwd" ]]; then
  user="$2"
  printf '%s:x:1000:1000::%s:/bin/bash\n' "${user}" "${home_dir}"
  exit 0
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

write_stub_getfacl() {
  cat >"${STUB_BIN}/getfacl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
user="${SUBSTRATE_TEST_PRIMARY_USER:-substrate-smoke}"
for arg in "$@"; do
  [[ "${arg}" == -* ]] && continue
  printf '# file: %s\n' "${arg}"
  case "${arg}" in
    */run/substrate.sock)
      printf 'user::rw-\ngroup::rw-\nother::---\nuser:%s:rw-\n' "${user}"
      ;;
    */var/lib/substrate)
      printf 'user::rwx\ngroup::r-x\nother::---\nuser:%s:--x\n' "${user}"
      ;;
    */var/lib/substrate/world-deps|*/var/lib/substrate/world-deps/bin)
      printf 'user::rwx\ngroup::r-x\nother::---\nuser:%s:r-x\n' "${user}"
      ;;
    *)
      if [[ -d "${arg}" ]]; then
        printf 'user::rwx\ngroup::r-x\nother::---\nuser:%s:r-x\n' "${user}"
      else
        printf 'user::rw-\ngroup::r--\nother::---\nuser:%s:r-x\n' "${user}"
      fi
      ;;
  esac
done
EOF
  chmod +x "${STUB_BIN}/getfacl"
}

write_stub_install() {
  cat >"${STUB_BIN}/install" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
mode=""
make_dirs=0
create_parent=0
args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -Dm*)
      create_parent=1
      mode="${1#-Dm}"
      shift
      ;;
    -d)
      make_dirs=1
      shift
      ;;
    -D)
      create_parent=1
      shift
      ;;
    -m*)
      if [[ "$1" == "-m" ]]; then
        mode="${2:-}"
        shift 2
      else
        mode="${1#-m}"
        shift
      fi
      ;;
    -*)
      shift
      ;;
    *)
      args+=("$1")
      shift
      ;;
  esac
done

if [[ ${make_dirs} -eq 1 ]]; then
  for dest in "${args[@]}"; do
    mkdir -p "${dest}"
    if [[ -n "${mode}" ]]; then
      chmod "${mode}" "${dest}" 2>/dev/null || true
    fi
  done
  exit 0
fi

if [[ ${#args[@]} -ne 2 ]]; then
  printf 'stub install expected src and dest, got %s args\n' "${#args[@]}" >&2
  exit 1
fi

src="${args[0]}"
dest="${args[1]}"
if [[ ${create_parent} -eq 1 ]]; then
  mkdir -p "$(dirname "${dest}")"
fi
cp "${src}" "${dest}"
if [[ -n "${mode}" ]]; then
  chmod "${mode}" "${dest}" 2>/dev/null || true
fi
EOF
  chmod +x "${STUB_BIN}/install"
}

write_stub_usermod() {
  cat >"${STUB_BIN}/usermod" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
usermod_state="${SUBSTRATE_TEST_USERMOD_STATE:-}"
if [[ -n "${log}" ]]; then
  printf 'usermod %s\n' "$*" >>"${log}"
fi
if [[ -n "${usermod_state}" ]]; then
  : > "${usermod_state}"
fi
exit 0
EOF
  chmod +x "${STUB_BIN}/usermod"
}

write_stub_loginctl() {
  cat >"${STUB_BIN}/loginctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
state="${SUBSTRATE_TEST_LINGER_STATE:-no}"
log="${SUBSTRATE_TEST_LINGER_LOG:-}"
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

write_stub_substrate() {
  local bin_path="${REPO_ROOT}/target/${PROFILE}/substrate"
  mkdir -p "$(dirname "${bin_path}")"
  cat >"${bin_path}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GATEWAY_LOG:-}"
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
case "${command}" in
  "--install-bootstrap-home-v1")
    exit 0
    ;;
  "config current show --json")
    printf '%s\n' "${SUBSTRATE_TEST_CONFIG_JSON:-}"
    exit 0
    ;;
  "policy current show --json")
    printf '%s\n' "${SUBSTRATE_TEST_POLICY_JSON:-}"
    exit 0
    ;;
  "world gateway sync")
    if [[ -n "${SUBSTRATE_TEST_EXPECTED_SYNTHETIC_AUTH:-}" ]]; then
      [[ -f "${SUBSTRATE_TEST_EXPECTED_SYNTHETIC_AUTH}" ]] || exit 94
      [[ -n "${log}" ]] && printf 'synthetic-auth-present %s\n' "${SUBSTRATE_TEST_EXPECTED_SYNTHETIC_AUTH}" >>"${log}"
    fi
    [[ -n "${log}" ]] && printf 'substrate %s\n' "${command}" >>"${log}"
    exit 0
    ;;
  "world gateway status --json")
    [[ -n "${log}" ]] && printf 'substrate %s\n' "${command}" >>"${log}"
    printf '{"status":"available","openai_base_url":"http://127.0.0.1:43123"}\n'
    exit 0
    ;;
  "world gateway restart")
    [[ -n "${log}" ]] && printf 'substrate %s\n' "${command}" >>"${log}"
    exit 0
    ;;
esac
printf 'unexpected substrate args\n' >&2
exit 1
EOF
  chmod +x "${bin_path}"
}

write_stub_curl() {
  cat >"${STUB_BIN}/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GATEWAY_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'curl %s\n' "$*" >>"${log}"
fi
printf '{"status":"ok","service":"substrate-gateway"}\n'
EOF
  chmod +x "${STUB_BIN}/curl"
}

write_stub_helpers() {
  write_stub_sudo
  write_stub_systemctl
  write_stub_id
  write_stub_getent
  write_stub_groupadd
  write_stub_getfacl
  write_stub_install
  write_stub_usermod
  write_stub_loginctl
  write_stub_curl
}

ensure_stub_binaries() {
  local world_agent_bin="${REPO_ROOT}/target/${PROFILE}/world-service"
  local gateway_bin="${REPO_ROOT}/target/${PROFILE}/substrate-gateway"
  local lifecycle_executor_bin="${REPO_ROOT}/target/${PROFILE}/substrate-lifecycle-linux"
  mkdir -p "$(dirname "${world_agent_bin}")"

  cat >"${world_agent_bin}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${world_agent_bin}"

  cat >"${gateway_bin}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${gateway_bin}"

  cat >"${lifecycle_executor_bin}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${lifecycle_executor_bin}"

  write_stub_substrate
}

# shellcheck disable=SC2030,SC2034
verify_synthetic_auth_account_home_cleanup() (
  set -euo pipefail
  local isolated_root="${WORK_ROOT}/synthetic-auth-isolated"
  local created_home="${isolated_root}/account-home-created"
  local existing_home="${isolated_root}/account-home-existing"
  local created_auth_path="${created_home}/.codex/auth.json"
  local existing_auth_path="${existing_home}/.codex/auth.json"
  local existing_codex_dir="${existing_home}/.codex"
  local gateway_log="${isolated_root}/gateway.log"
  local carrier="isolated-authenticated-carrier"
  local substrate_stub="${REPO_ROOT}/target/${PROFILE}/substrate"
  local existing_mode_before
  local existing_mode_after
  mkdir -p "${created_home}" "${existing_codex_dir}"
  chmod 0755 "${existing_codex_dir}"
  : > "${gateway_log}"

  # shellcheck disable=SC1090
  source <(awk '/^while \[\[ \$# -gt 0 \]\]; do/ { exit } { print }' \
    "${REPO_ROOT}/scripts/linux/world-provision.sh")
  REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
  INSTALL_BOOTSTRAP_CONTEXT_V1="${carrier}"
  SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${carrier}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1
  export SUBSTRATE_TEST_GATEWAY_LOG="${gateway_log}"
  PATH="${STUB_BIN}:${PATH}"

  run_auth_cleanup_probe() {
    local account_home="$1"
    local auth_path="$2"
    INSTALL_BOOTSTRAP_ACCOUNT_HOME="${account_home}"
    INVOKING_HOME="${account_home}"
    export SUBSTRATE_TEST_EXPECTED_SYNTHETIC_AUTH="${auth_path}"
    : > "${gateway_log}"
    run_gateway_lifecycle_proof "${substrate_stub}" synthetic_auth_file
    assert_contains "synthetic-auth-present ${auth_path}" "${gateway_log}" \
      "gateway proof must create synthetic auth in the committed account home before dispatch"
    [[ ! -e "${auth_path}" ]] \
      || fatal "gateway proof did not remove the synthetic auth file it created at ${auth_path}"
  }

  run_auth_cleanup_probe "${created_home}" "${created_auth_path}"
  [[ ! -e "${created_home}/.codex" ]] \
    || fatal "gateway proof must remove the synthetic auth parent directory it created under ${created_home}"

  existing_mode_before="$(stat -c '%a' "${existing_codex_dir}")"
  run_auth_cleanup_probe "${existing_home}" "${existing_auth_path}"
  [[ -d "${existing_codex_dir}" ]] \
    || fatal "gateway proof must preserve a pre-existing synthetic auth parent directory at ${existing_codex_dir}"
  existing_mode_after="$(stat -c '%a' "${existing_codex_dir}")"
  [[ "${existing_mode_after}" == "${existing_mode_before}" ]] \
    || fatal "gateway proof must preserve the pre-existing ${existing_codex_dir} mode (${existing_mode_before} != ${existing_mode_after})"
)

verify_world_lifecycle_acl_restore() {
  local output
  local script_path

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"
  output="$(
    SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

FAKE_ROOT="$(mktemp -d)"
SOCKET_FS_PATH="${FAKE_ROOT}/run/substrate.sock"
SUBSTRATE_STATE_PATH="${FAKE_ROOT}/var/lib/substrate"
WORLD_DEPS_ROOT_PATH="${SUBSTRATE_STATE_PATH}/world-deps"
WORLD_DEPS_BIN_PATH="${WORLD_DEPS_ROOT_PATH}/bin"
WORLD_AGENT_BIN_PATH="${FAKE_ROOT}/artifacts/world-agent"
GATEWAY_BIN_PATH="${FAKE_ROOT}/artifacts/gateway"
ACL_HELPER_SOURCE_PATH="${FAKE_ROOT}/artifacts/acl-helper"
ACL_HELPER_INSTALL_PATH="${FAKE_ROOT}/usr/libexec/substrate/substrate-apply-socket-acl"
LIFECYCLE_EXECUTOR_BIN_PATH="${FAKE_ROOT}/artifacts/lifecycle"
SCRIPT_DIR="${FAKE_ROOT}/artifacts/scripts/linux"
SERVICE_PATH="${FAKE_ROOT}/etc/systemd/system/substrate-world-service.service"
SOCKET_PATH="${FAKE_ROOT}/etc/systemd/system/substrate-world-service.socket"
ACL_DROPIN_PATH="${FAKE_ROOT}/etc/systemd/system/substrate-world-service.socket.d/20-substrate-group-acl.conf"
SERVICE_UNIT_CONTENT="service"
SOCKET_UNIT_CONTENT="socket"
SOCKET_DROPIN_CONTENT="dropin"
INVOKING_USER="tester"
SUBSTRATE_CLI_BIN_PATH="/bin/true"
SUBSTRATE_GROUP="substrate"
DRY_RUN=0

mkdir -p \
  "${FAKE_ROOT}/run" \
  "${WORLD_DEPS_BIN_PATH}" \
  "${FAKE_ROOT}/artifacts" \
  "$(dirname "${ACL_HELPER_INSTALL_PATH}")" \
  "${SCRIPT_DIR}" \
  "$(dirname "${SERVICE_PATH}")" \
  "$(dirname "${ACL_DROPIN_PATH}")"
install -m0644 /dev/null "${WORLD_AGENT_BIN_PATH}"
install -m0644 /dev/null "${GATEWAY_BIN_PATH}"
install -m0644 /dev/null "${ACL_HELPER_SOURCE_PATH}"
install -m0644 /dev/null "${ACL_HELPER_INSTALL_PATH}"
install -m0644 /dev/null "${LIFECYCLE_EXECUTOR_BIN_PATH}"
install -m0644 /dev/null "${SERVICE_PATH}"
install -m0644 /dev/null "${SOCKET_PATH}"
install -m0644 /dev/null "${ACL_DROPIN_PATH}"
install -m0644 /dev/null "${SOCKET_FS_PATH}"

getfacl() { :; }
setfacl() { :; }

sudo_cmd() {
  local cmd="$1"
  shift || true
  case "${cmd}" in
    test|cp|rm|install)
      command "${cmd}" "$@"
      ;;
    getfacl)
      local arg
      for arg in "$@"; do
        [[ "${arg}" == -* ]] && continue
        printf '# file: %s\n' "${arg}"
        printf 'user::rwx\ngroup::r-x\nother::---\n'
      done
      ;;
    setfacl)
      printf 'setfacl %s\n' "$*"
      ;;
    *)
      return 0
      ;;
  esac
}

LINUX_MANAGED_STATE_SNAPSHOT_ROOT="$(mktemp -d)"
: >"$(linux_snapshot_acl_state_path)"
linux_snapshot_acl_state "world-socket-acl" "${SOCKET_FS_PATH}" "0"
linux_snapshot_acl_state "state-root-acl" "${SUBSTRATE_STATE_PATH}" "0"
linux_snapshot_acl_state "world-deps-acl" "${WORLD_DEPS_ROOT_PATH}" "1"
install -m0644 /dev/null "${SOCKET_FS_PATH}"
cat "$(linux_snapshot_acl_state_path)"
while IFS=$'\t' read -r label target recursive backup; do
  [[ -n "${label}" ]] || continue
  linux_restore_acl_state "${label}" "${target}" "${recursive}" "${backup}"
done <"$(linux_snapshot_acl_state_path)"
EOF
  )"

  if ! grep -Fq -- $'world-socket-acl\t' <<<"${output}"; then
    fatal "world-lifecycle must snapshot pre-existing socket ACL state before mutation"
  fi
  if ! grep -Fq -- $'state-root-acl\t' <<<"${output}"; then
    fatal "world-lifecycle must snapshot pre-existing state-root ACL state before mutation"
  fi
  if ! grep -Fq -- $'world-deps-acl\t' <<<"${output}"; then
    fatal "world-lifecycle must snapshot pre-existing world-deps ACL state before mutation"
  fi
  if [[ "$(grep -c '^setfacl --restore=' <<<"${output}")" -ne 3 ]]; then
    fatal "world-lifecycle must restore all pre-existing ACL snapshots after service-state replay"
  fi
}

verify_world_lifecycle_symlink_restore() {
  local output
  local script_path

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"
  output="$(
    SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

DRY_RUN=0
FAKE_ROOT="$(mktemp -d)"
TARGET="${FAKE_ROOT}/etc/systemd/system/substrate-world-service.service"
LINK_TARGET="${FAKE_ROOT}/units/substrate-world-service.service"

mkdir -p "$(dirname "${TARGET}")" "$(dirname "${LINK_TARGET}")"
printf 'unit\n' >"${LINK_TARGET}"
ln -s "${LINK_TARGET}" "${TARGET}"

sudo_cmd() {
  local cmd="$1"
  shift || true
  case "${cmd}" in
    test|cp|rm|install|mkdir|stat)
      command "${cmd}" "$@"
      ;;
    chown)
      return 0
      ;;
    *)
      return 0
      ;;
  esac
}

LINUX_MANAGED_STATE_SNAPSHOT_ROOT="$(mktemp -d)"
: >"$(linux_snapshot_manifest_path)"
linux_snapshot_path "world-service-unit" "${TARGET}"
rm -f "${TARGET}"
printf 'replacement\n' >"${TARGET}"
while IFS=$'\t' read -r label target type mode backup owner group; do
  [[ -n "${label}" ]] || continue
  linux_restore_path "${label}" "${target}" "${type}" "${mode}" "${backup}" "${owner}" "${group}"
done <"$(linux_snapshot_manifest_path)"

type_field="$(cut -f3 "$(linux_snapshot_manifest_path)")"
printf 'type=%s\n' "${type_field}"
if [[ -L "${TARGET}" ]]; then
  printf 'link=%s\n' "$(readlink "${TARGET}")"
else
  printf 'link=missing\n'
fi
EOF
  )"

  if ! grep -Fq -- "type=symlink" <<<"${output}"; then
    fatal "world-lifecycle must snapshot symlinked managed targets without collapsing them into regular files"
  fi
  if ! grep -Fq -- "link=" <<<"${output}"; then
    fatal "world-lifecycle symlink restore test did not report the restored link target"
  fi
  if grep -Fq -- "link=missing" <<<"${output}"; then
    fatal "world-lifecycle must restore a symlinked managed target as a symlink"
  fi
}

verify_world_lifecycle_directory_owner_restore() {
  local output
  local script_path

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"
  output="$(
    EXPECTED_UID="1111" EXPECTED_GID="2222" SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

DRY_RUN=0
FAKE_ROOT="$(mktemp -d)"
TARGET="${FAKE_ROOT}/var/lib/substrate"

mkdir -p "${TARGET}"
chmod 0750 "${TARGET}"

sudo_cmd() {
  local cmd="$1"
  shift || true
  case "${cmd}" in
    test|cp|rm|install|mkdir)
      command "${cmd}" "$@"
      ;;
    stat)
      if [[ "${1:-}" == "-c" ]]; then
        case "${2:-}" in
          %a)
            command stat "$@"
            ;;
          %u)
            printf '%s\n' "${EXPECTED_UID}"
            ;;
          %g)
            printf '%s\n' "${EXPECTED_GID}"
            ;;
          *)
            command stat "$@"
            ;;
        esac
      else
        command stat "$@"
      fi
      ;;
    chown)
      printf 'chown %s\n' "$*"
      ;;
    *)
      return 0
      ;;
  esac
}

LINUX_MANAGED_STATE_SNAPSHOT_ROOT="$(mktemp -d)"
: >"$(linux_snapshot_manifest_path)"
linux_snapshot_path "state-root" "${TARGET}"
rm -rf "${TARGET}"
while IFS=$'\t' read -r label target type mode backup owner group; do
  [[ -n "${label}" ]] || continue
  printf 'manifest=%s:%s\n' "${owner}" "${group}"
  linux_restore_path "${label}" "${target}" "${type}" "${mode}" "${backup}" "${owner}" "${group}"
done <"$(linux_snapshot_manifest_path)"
printf 'mode=%s\n' "$(stat -c '%a' "${TARGET}")"
EOF
  )"

  if ! grep -Fq -- "manifest=1111:2222" <<<"${output}"; then
    fatal "world-lifecycle must record directory owner/group metadata in the rollback manifest"
  fi
  if ! grep -Fq -- "chown 1111:2222 " <<<"${output}"; then
    fatal "world-lifecycle must replay directory owner/group metadata during rollback"
  fi
  if ! grep -Fq -- "mode=750" <<<"${output}"; then
    fatal "world-lifecycle owner restore test did not preserve the original directory mode"
  fi
}

verify_world_lifecycle_dry_run_skips_snapshot() {
  local output
  local script_path

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"
  output="$(
    SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

WORLD_AGENT_BIN_PATH="/bin/true"
GATEWAY_BIN_PATH="/bin/true"
ACL_HELPER_SOURCE_PATH="/bin/true"
LIFECYCLE_EXECUTOR_BIN_PATH="/bin/true"
LIFECYCLE_EXECUTOR_INSTALL_PATH="/usr/libexec/substrate/substrate-lifecycle-linux"
SCRIPT_DIR="/tmp"
SERVICE_PATH="/tmp/substrate-world-service.service"
SOCKET_PATH="/tmp/substrate-world-service.socket"
ACL_DROPIN_PATH="/tmp/20-substrate-group-acl.conf"
SERVICE_UNIT_CONTENT="service"
SOCKET_UNIT_CONTENT="socket"
SOCKET_DROPIN_CONTENT="dropin"
INVOKING_USER="tester"
SUBSTRATE_CLI_BIN_PATH="/bin/true"
SUBSTRATE_STATE_PATH="/tmp/substrate-state"
WORLD_DEPS_ROOT_PATH="/tmp/world-deps"
WORLD_DEPS_BIN_PATH="/tmp/world-deps/bin"
SOCKET_FS_PATH="/tmp/substrate.sock"
DRY_RUN=1

record_linux_managed_state() {
  printf 'record-called\n'
  return 99
}

install_linux_managed_state() {
  printf 'install-called\n'
}

resolve_substrate_cli() {
  printf '/bin/true\n'
}

maybe_run_gateway_lifecycle_proof() {
  :
}

print_linger_guidance() {
  :
}

main
EOF
  )"

  if grep -Fq -- "record-called" <<<"${output}"; then
    fatal "world-lifecycle dry-run must skip mutable pre-state snapshot capture"
  fi
  if ! grep -Fq -- "install-called" <<<"${output}"; then
    fatal "world-lifecycle dry-run test did not reach the bounded install path"
  fi
}

verify_world_lifecycle_err_trap_inherits() {
  local output
  local script_path
  local status

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"

  set +e
  output="$(
    SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

SUBSTRATE_GROUP="substrate"
INVOKING_USER="tester"
GROUP_STATE="$(mktemp)"
USER_MEMBER_STATE="$(mktemp)"
rm -f "${GROUP_STATE}" "${USER_MEMBER_STATE}"

linux_require_world_context() {
  return 0
}

record_linux_managed_state() {
  LINUX_MANAGED_STATE_SNAPSHOT_ROOT="$(mktemp -d)"
  : >"$(linux_snapshot_manifest_path)"
  : >"$(linux_snapshot_service_path)"
  : >"$(linux_snapshot_acl_state_path)"
  printf '%s\t%s\t%s\t%s\n' "${SUBSTRATE_GROUP}" "${INVOKING_USER}" "0" "0" >"$(linux_snapshot_account_state_path)"
}

failing_helper() {
  false
}

install_linux_managed_state() {
  : >"${GROUP_STATE}"
  : >"${USER_MEMBER_STATE}"
  failing_helper
}

sudo_cmd() {
  local cmd="$1"
  shift || true
  case "${cmd}" in
    systemctl)
      return 0
      ;;
    gpasswd)
      printf 'restore-gpasswd %s\n' "$*"
      rm -f "${USER_MEMBER_STATE}"
      return 0
      ;;
    groupdel)
      printf 'restore-groupdel %s\n' "$*"
      rm -f "${GROUP_STATE}"
      return 0
      ;;
    usermod)
      printf 'restore-usermod %s\n' "$*"
      : >"${USER_MEMBER_STATE}"
      return 0
      ;;
    groupadd)
      printf 'restore-groupadd %s\n' "$*"
      : >"${GROUP_STATE}"
      return 0
      ;;
    *)
      return 0
      ;;
  esac
}

getent() {
  if [[ "${1:-}" == "group" && "${2:-}" == "${SUBSTRATE_GROUP}" && -f "${GROUP_STATE}" ]]; then
    printf '%s:x:1000:\n' "${SUBSTRATE_GROUP}"
    return 0
  fi
  return 2
}

id() {
  if [[ "${1:-}" == "-nG" && "${2:-}" == "${INVOKING_USER}" ]]; then
    if [[ -f "${USER_MEMBER_STATE}" ]]; then
      printf 'wheel %s\n' "${SUBSTRATE_GROUP}"
    else
      printf 'wheel\n'
    fi
    return 0
  fi
  if [[ "${1:-}" == "${INVOKING_USER}" ]]; then
    printf 'uid=1000(%s) gid=1000(%s) groups=1000(%s)\n' "${INVOKING_USER}" "${INVOKING_USER}" "${INVOKING_USER}"
    return 0
  fi
  command id "$@"
}

user_in_group() {
  [[ -f "${USER_MEMBER_STATE}" ]]
}

resolve_substrate_cli() {
  printf '/bin/true\n'
}

maybe_run_gateway_lifecycle_proof() {
  :
}

print_linger_guidance() {
  :
}

main
EOF
  )"
  status=$?
  set -e

  if [[ "${status}" -eq 0 ]]; then
    fatal "world-lifecycle ERR trap inheritance test unexpectedly succeeded"
  fi
  if ! grep -Fq -- "restore-gpasswd -d tester substrate" <<<"${output}"; then
    fatal "world-lifecycle ERR trap did not remove restored group membership after a helper failure"
  fi
  if ! grep -Fq -- "restore-groupdel substrate" <<<"${output}"; then
    fatal "world-lifecycle ERR trap did not remove the created substrate group after a helper failure"
  fi
}

verify_world_lifecycle_exact_service_restore() {
  local output
  local script_path

  script_path="$(cd "${SCRIPT_DIR}/../.." && pwd)/scripts/linux/world-lifecycle.sh"
  output="$(
    SCRIPT_PATH="${script_path}" bash 2>&1 <<'EOF'
set -euo pipefail
source "${SCRIPT_PATH}"

EXECUTOR_LOG="$(mktemp)"

sudo_cmd() {
  local cmd="$1"
  shift || true
  if [[ "${cmd}" == "systemctl" ]]; then
    printf 'systemctl %s\n' "$*"
    if [[ "${1:-}" == "start" && "${2:-}" == "broken.service" ]]; then
      return 1
    fi
  fi
  return 0
}

invoke_linux_lifecycle_executor() {
  printf '%s\n' "$*" >>"${EXECUTOR_LOG}"
  return 0
}

linux_restore_service_state "masked.service" "masked" "inactive"
linux_restore_service_state "masked-active.service" "masked" "active"
linux_restore_service_state "substrate-lifecycle-publisher-v1.service" "enabled" "active"
linux_restore_service_state "runtime.service" "enabled-runtime" "active"

set +e
linux_restore_service_state "failed.service" "disabled" "failed"
status=$?
linux_restore_service_state "broken.service" "enabled" "active"
broken_status=$?
set -e
printf 'failed-status=%s\n' "${status}"
printf 'broken-status=%s\n' "${broken_status}"
cat "${EXECUTOR_LOG}"
EOF
  )"

  if ! grep -Fq -- "systemctl mask masked.service" <<<"${output}"; then
    fatal "world-lifecycle restore must preserve masked unit state"
  fi
  local masked_active_unmask_line
  local masked_active_start_line
  local masked_active_mask_line
  masked_active_unmask_line="$(grep -n "systemctl unmask masked-active.service" <<<"${output}" | cut -d: -f1)"
  masked_active_start_line="$(grep -n "systemctl start masked-active.service" <<<"${output}" | cut -d: -f1)"
  masked_active_mask_line="$(grep -n "systemctl mask masked-active.service" <<<"${output}" | cut -d: -f1)"
  if [[ -z "${masked_active_unmask_line}" || -z "${masked_active_start_line}" || -z "${masked_active_mask_line}" ]]; then
    fatal "world-lifecycle restore must unmask, start, and remask masked active units"
  fi
  if ! [[ "${masked_active_unmask_line}" -lt "${masked_active_start_line}" && "${masked_active_start_line}" -lt "${masked_active_mask_line}" ]]; then
    fatal "world-lifecycle restore must start masked active units before restoring the masked state"
  fi
  if ! grep -Fq -- "systemctl enable --runtime runtime.service" <<<"${output}"; then
    fatal "world-lifecycle restore must preserve runtime-only enablement"
  fi
  if ! grep -Fq -- "systemctl start runtime.service" <<<"${output}"; then
    fatal "world-lifecycle restore must preserve active unit state"
  fi
  if ! grep -Fq -- "service-state --service-unit substrate-lifecycle-publisher-v1.service --action start" <<<"${output}"; then
    fatal "world-lifecycle restore must delegate lifecycle publisher service start through the Linux lifecycle executor"
  fi
  if ! grep -Fq -- "failed-status=1" <<<"${output}"; then
    fatal "world-lifecycle restore must fail closed for unsupported active states"
  fi
  if ! grep -Fq -- "broken-status=1" <<<"${output}"; then
    fatal "world-lifecycle restore must fail closed when a systemctl restore transition fails"
  fi
}

assert_contains() {
  local needle="$1"
  local path="$2"
  local message="$3"
  if ! grep -Fq -- "${needle}" "${path}"; then
    fatal "${message} (missing '${needle}' in ${path})"
  fi
}

assert_not_contains() {
  local needle="$1"
  local path="$2"
  local message="$3"
  if grep -Fq -- "${needle}" "${path}"; then
    fatal "${message} (unexpected '${needle}' in ${path})"
  fi
}

assert_socket_unit() {
  local fake_root="$1"
  local unit="${fake_root}/etc/systemd/system/substrate-world-service.socket"
  if [[ ! -f "${unit}" ]]; then
    fatal "socket unit missing at ${unit}"
  fi
  assert_contains "SocketMode=0660" "${unit}" "socket mode must be 0660"
  assert_contains "SocketUser=root" "${unit}" "socket user must remain root"
  assert_contains "SocketGroup=substrate" "${unit}" "socket group must be substrate"
  assert_not_contains "PartOf=substrate-world-service.service" "${unit}" "world socket must not inherit PartOf propagation"
}

assert_lifecycle_install() {
  local fake_root="$1"
  local provision_log="$2"
  local systemctl_log="$3"
  local executor="${fake_root}/usr/libexec/substrate/substrate-lifecycle-linux"
  local service_unit="${fake_root}/etc/systemd/system/substrate-lifecycle-publisher-v1.service"
  local socket_unit="${fake_root}/etc/systemd/system/substrate-lifecycle-publisher-v1.socket"
  local socket_path="${fake_root}/run/substrate-lifecycle-publisher-v1.sock"

  [[ -x "${executor}" ]] || fatal "lifecycle executor missing at ${executor}"
  [[ -f "${service_unit}" ]] || fatal "lifecycle service unit missing at ${service_unit}"
  [[ -f "${socket_unit}" ]] || fatal "lifecycle socket unit missing at ${socket_unit}"
  [[ -e "${socket_path}" ]] || fatal "lifecycle socket endpoint missing at ${socket_path}"

  assert_contains "ExecStart=/usr/libexec/substrate/substrate-lifecycle-linux run-publisher" "${service_unit}" "lifecycle service must launch the Linux executor"
  assert_contains "ListenSequentialPacket=/run/substrate-lifecycle-publisher-v1.sock" "${socket_unit}" "lifecycle socket must listen on the fixed seqpacket endpoint"
  assert_contains "Service=substrate-lifecycle-publisher-v1.service" "${socket_unit}" "lifecycle socket must target the fixed publisher service"
  assert_contains "Verify lifecycle socket: sudo ls -l /run/substrate-lifecycle-publisher-v1.sock" "${provision_log}" "provisioner must surface lifecycle socket verification guidance"
  assert_contains "systemctl enable substrate-lifecycle-publisher-v1.socket" "${systemctl_log}" "lifecycle socket must be enabled"
  assert_contains "systemctl start substrate-lifecycle-publisher-v1.socket" "${systemctl_log}" "lifecycle socket must be started"
}

assert_service_context_unit() {
  local fake_root="$1"
  local selected_prefix="$2"
  local unit="${fake_root}/etc/systemd/system/substrate-world-service.service"
  local dropin="${fake_root}/etc/systemd/system/substrate-world-service.socket.d/20-substrate-group-acl.conf"
  local acl_helper="${fake_root}/usr/libexec/substrate/substrate-apply-socket-acl"
  [[ -f "${unit}" ]] || fatal "service unit missing at ${unit}"
  assert_contains "Environment=\"SUBSTRATE_HOME=${selected_prefix}\"" "${unit}" "service H must derive from A"
  assert_contains "Environment=\"SUBSTRATE_ROOT=${selected_prefix}\"" "${unit}" "service R must derive from A"
  assert_contains "Environment=\"SUBSTRATE_INSTALL_PRIMARY_USER=${CURRENT_ACCOUNT}\"" "${unit}" "service account must derive from IH"
  assert_contains "Environment=\"SUBSTRATE_INSTALL_PRIMARY_UID=${CURRENT_UID}\"" "${unit}" "service UID must derive from IH"
  assert_contains 'Environment="SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=' "${unit}" "service commitment projection missing"
  assert_contains 'Environment="SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=' "${unit}" "service carrier projection missing"
  assert_contains "ReadWritePaths=\"${selected_prefix}\" /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp" "${unit}" "ReadWritePaths must derive from A"
  [[ -f "${dropin}" ]] || fatal "socket ACL drop-in missing at ${dropin}"
  assert_contains 'ExecStartPost=-/usr/libexec/substrate/substrate-apply-socket-acl --socket /run/substrate.sock substrate' "${dropin}" "socket ACL drop-in tuple changed"
  [[ -x "${acl_helper}" ]] || fatal "installed ACL helper missing at ${acl_helper}"
}

assert_group_ops() {
  local group_log="$1"
  if [[ ! -s "${group_log}" ]]; then
    fatal "group operation log empty (expected groupadd/usermod calls)"
  fi
  if ! grep -Eq 'groupadd .*substrate' "${group_log}"; then
    fatal "expected groupadd substrate entry in ${group_log}"
  fi
  if ! grep -Eq 'usermod .*substrate' "${group_log}"; then
    fatal "expected usermod substrate entry in ${group_log}"
  fi
}

assert_linger_guidance() {
  local provision_log="$1"
  assert_contains "loginctl enable-linger" "${provision_log}" "provisioner output missing loginctl enable-linger guidance"
}

assert_group_guidance() {
  local provision_log="$1"
  if grep -qi 'newgrp substrate' "${provision_log}"; then
    return
  fi
  if grep -qi 'log out' "${provision_log}"; then
    return
  fi
  fatal "provisioner output missing logout/newgrp guidance"
}

assert_clean_install_skip() {
  local provision_log="$1"
  local gateway_log="$2"
  local fake_home="$3"
  assert_contains "Skipping gateway lifecycle proof" "${provision_log}" "clean install should skip the gateway proof"
  assert_contains "Provisioning continues without the proof." "${provision_log}" "clean install skip should explain provisioning continues"
  assert_contains "llm.gateway.enabled=true" "${provision_log}" "clean install skip should explain config remediation"
  assert_contains "agents.host_credentials.read.allowed_backends" "${provision_log}" "clean install skip should explain auth remediation"
  assert_contains "The installer does not modify config or policy to satisfy these checks." "${provision_log}" "clean install skip should explain installer behavior"
  if [[ -s "${gateway_log}" ]]; then
    fatal "gateway proof commands should not run on clean install (see ${gateway_log})"
  fi
  if [[ -e "${fake_home}/.codex/auth.json" ]]; then
    fatal "clean install skip should not create ${fake_home}/.codex/auth.json"
  fi
}

assert_configured_eligible_proof() {
  local provision_log="$1"
  local gateway_log="$2"
  local fake_home="$3"
  assert_contains "Running gateway lifecycle proof (auth: env_handoff)" "${provision_log}" "eligible install should run the gateway proof"
  assert_not_contains "Skipping gateway lifecycle proof" "${provision_log}" "eligible install should not skip the gateway proof"
  assert_contains "substrate world gateway sync" "${gateway_log}" "gateway proof should sync the gateway"
  assert_contains "substrate world gateway status --json" "${gateway_log}" "gateway proof should read gateway status"
  assert_contains "substrate world gateway restart" "${gateway_log}" "gateway proof should restart the gateway"
  assert_contains "curl --fail --silent http://127.0.0.1:43123/health" "${gateway_log}" "gateway proof should health-check the gateway"
  if [[ -e "${fake_home}/.codex/auth.json" ]]; then
    fatal "eligible proof should clean up synthetic auth at ${fake_home}/.codex/auth.json"
  fi
}

assert_world_deps_probe_verified() {
  local provision_log="$1"
  local probe_path="$2"
  assert_contains "Verified named-user ACL bridge for ${CURRENT_ACCOUNT} on runtime probe ${probe_path}." "${provision_log}" "world-deps verification should go green only on the real probe path"
}

assert_world_deps_probe_missing_warning() {
  local provision_log="$1"
  local probe_path="$2"
  assert_contains "cannot yet verify the world-deps ACL bridge against runtime probe ${probe_path}" "${provision_log}" "missing probe path should not report a false-green bridge"
  assert_contains "is not present yet" "${provision_log}" "missing probe path warning should explain the missing runtime entrypoint"
}

assert_world_deps_probe_unreachable_warning() {
  local provision_log="$1"
  local probe_path="$2"
  assert_contains "current shell still cannot reach the world-scoped runtime probe ${probe_path}" "${provision_log}" "unreachable probe path should warn instead of going green"
  assert_contains "may resolve deeper into world-deps package directories" "${provision_log}" "unreachable probe path warning should explain the deeper runtime surface"
}

prepare_world_deps_probe() {
  local fake_root="$1"
  local probe_mode="$2"
  local world_deps_root="${fake_root}/var/lib/substrate/world-deps"
  local world_deps_bin="${world_deps_root}/bin"
  local package_bin="${world_deps_root}/packages/codex/bin"
  local target="${package_bin}/codex"

  mkdir -p "${world_deps_bin}"

  case "${probe_mode}" in
    missing)
      return
      ;;
    accessible|denied)
      mkdir -p "${package_bin}"
      cat >"${target}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
      chmod 0755 "${target}"
      ln -sf ../packages/codex/bin/codex "${world_deps_bin}/codex"
      if [[ "${probe_mode}" == "denied" ]]; then
        chmod 000 "${target}"
      fi
      ;;
    *)
      fatal "unknown probe mode '${probe_mode}'"
      ;;
  esac
}

# shellcheck disable=SC2031
run_scenario() {
  local scenario_name="$1"
  local config_json="$2"
  local policy_json="$3"
  local assertion_mode="$4"
  local probe_mode="$5"

  local scenario_root="${WORK_ROOT}/${scenario_name}"
  local fake_root="${scenario_root}/fakeroot"
  local logs_dir="${scenario_root}/logs"
  local fake_home="${scenario_root}/home/substrate-smoke"
  local systemctl_log="${logs_dir}/systemctl.log"
  local group_log="${logs_dir}/group_ops.log"
  local linger_log="${logs_dir}/linger.log"
  local gateway_log="${logs_dir}/gateway.log"
  local sudo_tool_log="${logs_dir}/sudo-tools.log"
  local provision_log="${logs_dir}/provision.log"
  local usermod_state="${scenario_root}/usermod.state"
  local path_env="${STUB_BIN}:$PATH"
  local selected_prefix="${scenario_root}/selected-prefix"

  if [[ "${assertion_mode}" == eligible_run* ]]; then
    policy_json="${policy_json/\"env_allowed\":[]/\"env_allowed\":[\"SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN\"]}"
  fi

  mkdir -p "${fake_root}" "${logs_dir}" "${fake_home}" "${selected_prefix}"
  : > "${systemctl_log}"
  : > "${group_log}"
  : > "${linger_log}"
  : > "${gateway_log}"
  : > "${sudo_tool_log}"
  : > "${provision_log}"

  prepare_world_deps_probe "${fake_root}" "${probe_mode}"
  local probe_path="${fake_root}/var/lib/substrate/world-deps/bin/codex"

  log "Running scenario '${scenario_name}'"
  if ! (
    cd "${REPO_ROOT}" && \
    env \
      PATH="${path_env}" \
      FAKE_ROOT="${fake_root}" \
      SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR="${fake_root}/var/lib/substrate/world-deps/bin" \
      SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
      SUBSTRATE_TEST_STUB_BIN="${STUB_BIN}" \
      SUBSTRATE_TEST_SUDO_TOOL_LOG="${sudo_tool_log}" \
      SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
      SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
      SUBSTRATE_TEST_GATEWAY_LOG="${gateway_log}" \
      SUBSTRATE_TEST_PRIMARY_USER="substrate-smoke" \
      SUBSTRATE_TEST_ACTIVE_GROUPS="wheel docker" \
      SUBSTRATE_TEST_ACCOUNT_GROUPS_BEFORE="wheel docker" \
      SUBSTRATE_TEST_ACCOUNT_GROUPS_AFTER="wheel docker substrate" \
      SUBSTRATE_TEST_GROUP_EXISTS=0 \
      SUBSTRATE_TEST_LINGER_STATE="no" \
      SUBSTRATE_TEST_HOME="${fake_home}" \
      SUBSTRATE_TEST_USERMOD_STATE="${usermod_state}" \
      SUBSTRATE_TEST_CONFIG_JSON="${config_json}" \
      SUBSTRATE_TEST_POLICY_JSON="${policy_json}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN="test-only-token" \
      scripts/linux/world-provision.sh --home "${selected_prefix}" --profile "${PROFILE}" --skip-build >"${provision_log}" 2>&1
  ); then
    log "Provisioner output for scenario '${scenario_name}':"
    sed 's/^/[provision] /' "${provision_log}" >&2 || true
    fatal "world-provision.sh failed for scenario '${scenario_name}'"
  fi

  assert_socket_unit "${fake_root}"
  assert_service_context_unit "${fake_root}" "${selected_prefix}"
  assert_lifecycle_install "${fake_root}" "${provision_log}" "${systemctl_log}"
  assert_group_ops "${group_log}"
  assert_linger_guidance "${provision_log}"
  assert_group_guidance "${provision_log}"
  assert_contains "Provisioning complete" "${provision_log}" "provisioner should report completion"
  assert_not_contains "${STUB_BIN}/" "${sudo_tool_log}" \
    "world-provision sudo boundary must not select privileged tools from ambient PATH"
  if grep -Fxq -- "outer:env" "${sudo_tool_log}"; then
    fatal "world-provision sudo boundary left env selection to ambient PATH"
  fi

  case "${assertion_mode}" in
    clean_skip)
      assert_clean_install_skip "${provision_log}" "${gateway_log}" "${fake_home}"
      assert_world_deps_probe_missing_warning "${provision_log}" "${probe_path}"
      ;;
    eligible_run)
      assert_configured_eligible_proof "${provision_log}" "${gateway_log}" "${fake_home}"
      assert_world_deps_probe_verified "${provision_log}" "${probe_path}"
      ;;
    eligible_run_probe_denied)
      assert_configured_eligible_proof "${provision_log}" "${gateway_log}" "${fake_home}"
      assert_world_deps_probe_unreachable_warning "${provision_log}" "${probe_path}"
      ;;
    *)
      fatal "unknown assertion mode '${assertion_mode}'"
      ;;
  esac
}

write_stub_helpers
ensure_stub_binaries
verify_synthetic_auth_account_home_cleanup
verify_world_lifecycle_acl_restore
verify_world_lifecycle_symlink_restore
verify_world_lifecycle_directory_owner_restore
verify_world_lifecycle_dry_run_skips_snapshot
verify_world_lifecycle_err_trap_inherits
verify_world_lifecycle_exact_service_restore

run_scenario \
  "clean-install" \
  '{"world":{"enabled":true,"anchor_mode":"workspace","anchor_path":"","caged":true,"net":{"filter":false},"env":{"inherit_from_host":false},"deps":{"enabled":[],"inventory_mode":"merged","builtins":"enabled"}},"policy":{"mode":"observe"},"sync":{"auto_sync":false,"direction":"from_world","conflict_policy":"prefer_host","exclude":[".git/**",".substrate/**"]},"repl":{"exit_cwd":"entered","max_pty_buffered_lines":2048},"llm":{"enabled":false,"gateway":{"enabled":false,"mode":"in_world"},"routing":{"default_backend":""}},"agents":{"enabled":false,"defaults":{"execution":{"scope":"world"},"cli":{"mode":"persistent"}},"hub":{"orchestrator_agent_id":"","world_restart":{"on_drift":"auto_restart"}},"toolbox":{"enabled":false,"bind":{"transport":"uds"}}}}' \
  '{"id":"default","name":"Default Policy","world_fs":{"host_visible":true,"fail_closed":{"routing":false},"caged_required":false,"write":{"enabled":true}},"llm":{"fail_closed":{"routing":true},"require_approval":false,"allowed_backends":[],"secrets":{"env_allowed":[]}},"agents":{"allowed_backends":[],"fail_closed":{"routing":true},"host_credentials":{"read":{"allowed_backends":[]}}},"workflow":{"router":{"enabled":false,"allow_cross_workspace":false,"allowed_rule_ids":[],"allowed_workflow_ids":[],"allowed_target_workspace_ids":[]}},"net_allowed":[],"cmd_allowed":[],"cmd_denied":["rm -rf *","curl * | bash","wget * | bash"],"cmd_isolated":[],"require_approval":false,"allow_shell_operators":true,"limits":{"max_memory_mb":null,"max_cpu_percent":null,"max_runtime_ms":null,"max_egress_bytes":null},"metadata":{}}' \
  "clean_skip" \
  "missing"

run_scenario \
  "configured-eligible" \
  '{"world":{"enabled":true,"anchor_mode":"workspace","anchor_path":"","caged":true,"net":{"filter":false},"env":{"inherit_from_host":false},"deps":{"enabled":[],"inventory_mode":"merged","builtins":"enabled"}},"policy":{"mode":"observe"},"sync":{"auto_sync":false,"direction":"from_world","conflict_policy":"prefer_host","exclude":[".git/**",".substrate/**"]},"repl":{"exit_cwd":"entered","max_pty_buffered_lines":2048},"llm":{"enabled":true,"gateway":{"enabled":true,"mode":"in_world"},"routing":{"default_backend":"cli:codex-host"}},"agents":{"enabled":false,"defaults":{"execution":{"scope":"world"},"cli":{"mode":"persistent"}},"hub":{"orchestrator_agent_id":"","world_restart":{"on_drift":"auto_restart"}},"toolbox":{"enabled":false,"bind":{"transport":"uds"}}}}' \
  '{"id":"default","name":"Default Policy","world_fs":{"host_visible":true,"fail_closed":{"routing":false},"caged_required":false,"write":{"enabled":true}},"llm":{"fail_closed":{"routing":true},"require_approval":false,"allowed_backends":["cli:codex-host"],"secrets":{"env_allowed":[]}},"agents":{"allowed_backends":[],"fail_closed":{"routing":true},"host_credentials":{"read":{"allowed_backends":["cli:codex-host"]}}},"workflow":{"router":{"enabled":false,"allow_cross_workspace":false,"allowed_rule_ids":[],"allowed_workflow_ids":[],"allowed_target_workspace_ids":[]}},"net_allowed":[],"cmd_allowed":[],"cmd_denied":["rm -rf *","curl * | bash","wget * | bash"],"cmd_isolated":[],"require_approval":false,"allow_shell_operators":true,"limits":{"max_memory_mb":null,"max_cpu_percent":null,"max_runtime_ms":null,"max_egress_bytes":null},"metadata":{}}' \
  "eligible_run" \
  "accessible"

run_scenario \
  "configured-eligible-probe-denied" \
  '{"world":{"enabled":true,"anchor_mode":"workspace","anchor_path":"","caged":true,"net":{"filter":false},"env":{"inherit_from_host":false},"deps":{"enabled":[],"inventory_mode":"merged","builtins":"enabled"}},"policy":{"mode":"observe"},"sync":{"auto_sync":false,"direction":"from_world","conflict_policy":"prefer_host","exclude":[".git/**",".substrate/**"]},"repl":{"exit_cwd":"entered","max_pty_buffered_lines":2048},"llm":{"enabled":true,"gateway":{"enabled":true,"mode":"in_world"},"routing":{"default_backend":"cli:codex-host"}},"agents":{"enabled":false,"defaults":{"execution":{"scope":"world"},"cli":{"mode":"persistent"}},"hub":{"orchestrator_agent_id":"","world_restart":{"on_drift":"auto_restart"}},"toolbox":{"enabled":false,"bind":{"transport":"uds"}}}}' \
  '{"id":"default","name":"Default Policy","world_fs":{"host_visible":true,"fail_closed":{"routing":false},"caged_required":false,"write":{"enabled":true}},"llm":{"fail_closed":{"routing":true},"require_approval":false,"allowed_backends":["cli:codex-host"],"secrets":{"env_allowed":[]}},"agents":{"allowed_backends":[],"fail_closed":{"routing":true},"host_credentials":{"read":{"allowed_backends":["cli:codex-host"]}}},"workflow":{"router":{"enabled":false,"allow_cross_workspace":false,"allowed_rule_ids":[],"allowed_workflow_ids":[],"allowed_target_workspace_ids":[]}},"net_allowed":[],"cmd_allowed":[],"cmd_denied":["rm -rf *","curl * | bash","wget * | bash"],"cmd_isolated":[],"require_approval":false,"allow_shell_operators":true,"limits":{"max_memory_mb":null,"max_cpu_percent":null,"max_runtime_ms":null,"max_egress_bytes":null},"metadata":{}}' \
  "eligible_run_probe_denied" \
  "denied"

log "All checks passed."
log "Artifacts: ${WORK_ROOT}"
