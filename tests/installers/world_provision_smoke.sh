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
SocketGroup=substrate, records group membership operations, and emits linger guidance.
The harness stubs systemd/group commands so it never touches the host.
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

WORK_ROOT="$(mktemp -d "/tmp/substrate-world-provision.XXXXXX")"
FAKE_ROOT="${WORK_ROOT}/fakeroot"
STUB_BIN="${WORK_ROOT}/stub-bin"
LOG_DIR="${WORK_ROOT}/logs"
mkdir -p "${FAKE_ROOT}" "${STUB_BIN}" "${LOG_DIR}"

SYSTEMCTL_LOG="${LOG_DIR}/systemctl.log"
GROUP_OP_LOG="${LOG_DIR}/group_ops.log"
LINGER_LOG="${LOG_DIR}/linger.log"
PROVISION_LOG="${LOG_DIR}/provision.log"
: > "${SYSTEMCTL_LOG}"
: > "${GROUP_OP_LOG}"
: > "${LINGER_LOG}"
: > "${PROVISION_LOG}"

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
    exec "${cmd}" "$@"
    ;;
  install|cp|mv|ln)
    args=("$@")
    rewrite_dest_arg args
    exec "${cmd}" "${args[@]}"
    ;;
  rm|mkdir|chmod|chown|ls)
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

write_stub_systemctl() {
  cat >"${STUB_BIN}/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_SYSTEMCTL_LOG:-}"
fake_root="${FAKE_ROOT:-}"
if [[ -n "${log}" ]]; then
  printf 'systemctl %s\n' "$*" >>"${log}"
fi
if [[ $# -ge 2 && "$1" == "start" && "$2" == "substrate-world-agent.socket" && -n "${fake_root}" ]]; then
  socket_path="${fake_root}/run/substrate.sock"
  mkdir -p "$(dirname "${socket_path}")"
  : >"${socket_path}"
  chmod 0660 "${socket_path}" 2>/dev/null || true
  chgrp substrate "${socket_path}" 2>/dev/null || true
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
  if [[ "${group}" == "substrate" ]]; then
    if [[ "${SUBSTRATE_TEST_GROUP_EXISTS:-0}" -eq 1 ]]; then
      printf 'substrate:x:1234:\n'
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

write_stub_helpers() {
  write_stub_sudo
  write_stub_systemctl
  write_stub_id
  write_stub_getent
  write_stub_groupadd
  write_stub_usermod
  write_stub_loginctl
}

ensure_stub_binary() {
  local bin_path="${REPO_ROOT}/target/${PROFILE}/world-agent"
  mkdir -p "$(dirname "${bin_path}")"
  if [[ ! -x "${bin_path}" ]]; then
    cat >"${bin_path}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
    chmod +x "${bin_path}"
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

assert_socket_unit() {
  local unit="${FAKE_ROOT}/etc/systemd/system/substrate-world-agent.socket"
  if [[ ! -f "${unit}" ]]; then
    fatal "socket unit missing at ${unit}"
  fi
  assert_contains "SocketMode=0660" "${unit}" "socket mode must be 0660"
  assert_contains "SocketUser=root" "${unit}" "socket user must remain root"
  assert_contains "SocketGroup=substrate" "${unit}" "socket group must be substrate"
}

assert_group_ops() {
  if [[ ! -s "${GROUP_OP_LOG}" ]]; then
    fatal "group operation log empty (expected groupadd/usermod calls)"
  fi
  if ! grep -Eq 'groupadd .*substrate' "${GROUP_OP_LOG}"; then
    fatal "expected groupadd substrate entry in ${GROUP_OP_LOG}"
  fi
  if ! grep -Eq 'usermod .*substrate' "${GROUP_OP_LOG}"; then
    fatal "expected usermod substrate entry in ${GROUP_OP_LOG}"
  fi
}

assert_linger_guidance() {
  if ! grep -qi 'loginctl enable-linger' "${PROVISION_LOG}"; then
    fatal "provisioner output missing loginctl enable-linger guidance"
  fi
}

assert_group_guidance() {
  if grep -qi 'newgrp substrate' "${PROVISION_LOG}"; then
    return
  fi
  if grep -qi 'log out' "${PROVISION_LOG}"; then
    return
  fi
  fatal "provisioner output missing logout/newgrp guidance"
}

run_provisioner() {
  local path_env="${STUB_BIN}:$PATH"
  log "Running world-provision.sh with profile ${PROFILE}"
  if ! (cd "${REPO_ROOT}" && \
    env \
      PATH="${path_env}" \
      FAKE_ROOT="${FAKE_ROOT}" \
      SUBSTRATE_TEST_SYSTEMCTL_LOG="${SYSTEMCTL_LOG}" \
      SUBSTRATE_TEST_GROUP_LOG="${GROUP_OP_LOG}" \
      SUBSTRATE_TEST_LINGER_LOG="${LINGER_LOG}" \
      SUBSTRATE_TEST_PRIMARY_USER="substrate-smoke" \
      SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
      SUBSTRATE_TEST_GROUP_EXISTS=0 \
      SUBSTRATE_TEST_LINGER_STATE="no" \
      scripts/linux/world-provision.sh --profile "${PROFILE}" --skip-build >"${PROVISION_LOG}" 2>&1); then
    log "Provisioner output:"
    sed 's/^/[provision] /' "${PROVISION_LOG}" >&2 || true
    fatal "world-provision.sh failed"
  fi
}

write_stub_helpers
ensure_stub_binary
run_provisioner
assert_socket_unit
assert_group_ops
assert_linger_guidance
assert_group_guidance

log "All checks passed."
log "Artifacts: ${LOG_DIR}"
