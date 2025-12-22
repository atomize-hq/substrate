#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

log() {
  printf '[mac-doctor] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-mac-doctor.XXXXXX")"
KEEP_ROOT="${KEEP_MAC_DOCTOR_FIXTURE:-0}"
STUB_BIN="${WORK_ROOT}/bin"
HOME_ROOT="${WORK_ROOT}/home"
mkdir -p "${STUB_BIN}" "${HOME_ROOT}"

cleanup() {
  if [[ "${KEEP_ROOT}" -eq 0 ]]; then
    rm -rf "${WORK_ROOT}"
  else
    log "Preserving ${WORK_ROOT} (--keep-root set)"
  fi
}
trap cleanup EXIT

write_stub_limactl() {
  cat >"${STUB_BIN}/limactl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
scenario="${MAC_DOCTOR_SCENARIO:-healthy}"
cmd="${1:-}"
if [[ -z "${cmd}" ]]; then
  exit 1
fi
shift || true

emit_status() {
  case "${scenario}" in
    vm_missing)
      exit 1
      ;;
    vm_stopped)
      printf '{"status":"Stopped"}\n'
      ;;
    *)
      printf '{"status":"Running"}\n'
      ;;
  esac
  exit 0
}

case "${cmd}" in
  list)
    target="${1:-}"
    if [[ "${target}" == "substrate" ]]; then
      shift || true
      if [[ "${1:-}" == "--json" ]]; then
        emit_status
      else
        if [[ "${scenario}" == "vm_missing" ]]; then
          exit 1
        fi
        exit 0
      fi
    fi
    exit 0
    ;;
  shell)
    target="${1:-}"
    if [[ "${target}" != "substrate" ]]; then
      printf 'limactl stub only supports substrate guest\n' >&2
      exit 1
    fi
    shift || true
    sub="${1:-}"
    shift || true
    case "${sub}" in
      uname)
        echo "Linux substrate"
        exit 0
        ;;
      systemctl)
        action="${1:-}"
        shift || true
        if [[ "${action}" == "is-active" ]]; then
          unit="${1:-}"
          if [[ "${unit}" == "substrate-world-agent" && "${scenario}" == "service_down" ]]; then
            exit 3
          fi
          echo "active"
          exit 0
        fi
        ;;
      sudo)
        while [[ "${1:-}" == -* ]]; do
          shift || true
        done
        inner="${1:-}"
        shift || true
        case "${inner}" in
          test)
            if [[ "${scenario}" == "socket_missing" ]]; then
              exit 1
            fi
            exit 0
            ;;
          timeout)
            # Skip timeout duration and arguments until curl
            while [[ $# -gt 0 && "${1}" != "curl" ]]; do
              shift || true
            done
            if [[ "${scenario}" == "caps_fail" ]]; then
              exit 124
            fi
            echo "{}"
            exit 0
            ;;
        esac
        ;;
      which)
        bin="${1:-}"
        if [[ "${bin}" == "nft" ]]; then
          echo "/usr/sbin/nft"
          exit 0
        fi
        ;;
      bash)
        if [[ "${1:-}" == "-lc" ]]; then
          echo "/dev/root      10G   5G   5G  50%  /"
          exit 0
        fi
        ;;
    esac
    ;;
  start)
    exit 0
    ;;
esac

printf 'limactl stub encountered unhandled args: %s %s\n' "${cmd}" "$*" >&2
exit 1
EOF
  chmod +x "${STUB_BIN}/limactl"
}

write_stub_sysctl() {
  cat >"${STUB_BIN}/sysctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == "-n" && "$2" == "kern.hv_support" ]]; then
  echo "${MAC_DOCTOR_VIRT:-1}"
else
  echo "0"
fi
EOF
  chmod +x "${STUB_BIN}/sysctl"
}

write_stub_jq() {
  cat >"${STUB_BIN}/jq" <<'EOF'
#!/usr/bin/env python3
import json
import sys

try:
    data = json.load(sys.stdin)
except Exception:
    print("unknown")
    sys.exit(0)

status = data.get("status") or "unknown"
print(status)
EOF
  chmod +x "${STUB_BIN}/jq"
}

write_stub_envsubst() {
  cat >"${STUB_BIN}/envsubst" <<'EOF'
#!/usr/bin/env bash
cat
EOF
  chmod +x "${STUB_BIN}/envsubst"
}

write_stub_vsock_proxy() {
  cat >"${STUB_BIN}/vsock-proxy" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${STUB_BIN}/vsock-proxy"
}

write_stub_limactl
write_stub_sysctl
write_stub_jq
write_stub_envsubst
write_stub_vsock_proxy

export PATH="${STUB_BIN}:${PATH}"
export HOME="${HOME_ROOT}"

run_doctor() {
  local scenario="$1"
  local needle="$2"
  local expected_exit="$3"
  local log_path="${WORK_ROOT}/doctor-${scenario}.log"
  local status=0
  MAC_DOCTOR_SCENARIO="${scenario}" \
    "${REPO_ROOT}/scripts/mac/lima-doctor.sh" >"${log_path}" 2>&1 || status=$?

  if ! grep -Fq "${needle}" "${log_path}"; then
    log "Scenario ${scenario} output missing '${needle}'"
    cat "${log_path}" >&2
    fatal "scenario ${scenario} did not include expected guidance"
  fi

  if [[ "${status}" -ne "${expected_exit}" ]]; then
    log "Scenario ${scenario} exit ${status} (expected ${expected_exit})"
    cat "${log_path}" >&2
    fatal "scenario ${scenario} exit mismatch"
  fi

  log "Scenario ${scenario} passed (exit=${status})"
}

run_doctor "vm_missing" "to create VM" 0
run_doctor "vm_stopped" "status: Stopped" 0
run_doctor "socket_missing" "Agent socket not found" 0
run_doctor "caps_fail" "Agent not responding" 1
run_doctor "service_down" "substrate-world-agent service is not active" 1
run_doctor "healthy" "All critical checks passed." 0

log "All Lima doctor fixture scenarios completed (artifacts: ${WORK_ROOT})"
