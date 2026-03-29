#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -eq 0 ]]; then
  echo "Do not run this smoke script as root." >&2
  exit 1
fi

SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-${REPO_ROOT}/target/debug/substrate}"

MODE="generic"
LOG_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/mac/smoke.sh [--netfilter-conformance | --bedpm-installer-conformance] [--log-dir DIR]

Options:
  --netfilter-conformance  Run the posture-aware Lima netfilter smoke instead of the generic smoke
  --bedpm-installer-conformance
                           Run the BEDPM Linux installer smoke through the Lima-backed guest path
  --log-dir DIR            Directory for doctor JSON and command transcripts (default: artifacts/mac/netfilter-smoke-<timestamp> in netfilter mode)
  -h, --help               Show this help text
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --netfilter-conformance)
      MODE="netfilter-conformance"
      shift
      ;;
    --bedpm-installer-conformance)
      MODE="bedpm-installer-conformance"
      shift
      ;;
    --log-dir)
      if [[ $# -lt 2 ]]; then
        echo "ERROR: --log-dir requires a value." >&2
        exit 1
      fi
      LOG_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

log() {
  printf '[mac-smoke] %s\n' "$*"
}

require_cmd() {
  local cmd="$1"
  local hint="${2:-}"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    if [[ -n "${hint}" ]]; then
      echo "ERROR: ${cmd} not found on PATH. ${hint}" >&2
    else
      echo "ERROR: ${cmd} not found on PATH." >&2
    fi
    exit 1
  fi
}

ensure_host_prereqs() {
  if ! command -v limactl >/dev/null 2>&1; then
    PATH="/opt/homebrew/opt/lima/bin:/opt/homebrew/bin:$PATH"
  fi

  require_cmd limactl "Install Lima via Homebrew (brew install lima)."
  require_cmd jq
}

ensure_substrate_binary() {
  if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
    log "Building substrate binary for smoke test..."
    (cd "${REPO_ROOT}" && cargo build --bin substrate >/dev/null)
  fi
}

default_netfilter_log_dir() {
  printf '%s\n' "${REPO_ROOT}/artifacts/mac/netfilter-smoke-$(date -u '+%Y%m%d-%H%M%S')"
}

write_smoke_config() {
  local substrate_home="$1"
  mkdir -p "${substrate_home}"
  cat > "${substrate_home}/config.yaml" <<'EOF'
world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: false
  net:
    filter: false

policy:
  mode: observe
EOF
}

write_smoke_policy() {
  local substrate_home="$1"
  local net_allowed_yaml="$2"
  mkdir -p "${substrate_home}"
  cat > "${substrate_home}/policy.yaml" <<EOF
id: mac-netfilter-smoke
name: mac-netfilter-smoke
world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true
net_allowed: ${net_allowed_yaml}
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
EOF
}

run_fixture_command() {
  local fixture_home="$1"
  local substrate_home="$2"
  local project_dir="$3"
  shift 3
  env \
    HOME="${fixture_home}" \
    USERPROFILE="${fixture_home}" \
    SUBSTRATE_HOME="${substrate_home}" \
    SHIM_TRACE_LOG="${fixture_home}/.substrate/trace.jsonl" \
    SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1 \
    "${SUBSTRATE_BIN}" "$@"
}

run_probe_with_capture() {
  local fixture_home="$1"
  local substrate_home="$2"
  local project_dir="$3"
  local stdout_log="$4"
  local stderr_log="$5"
  local exit_log="$6"
  local probe_cmd='getent hosts example.com'

  local rc=0
  if (
    cd "${project_dir}"
    run_fixture_command "${fixture_home}" "${substrate_home}" "${project_dir}" --world -c "${probe_cmd}"
  ) >"${stdout_log}" 2>"${stderr_log}"; then
    rc=0
  else
    rc=$?
  fi
  printf '%s\n' "${rc}" > "${exit_log}"
  return "${rc}"
}

capture_world_doctor() {
  local fixture_home="$1"
  local substrate_home="$2"
  local project_dir="$3"
  local doctor_json="$4"

  (
    cd "${project_dir}"
    run_fixture_command "${fixture_home}" "${substrate_home}" "${project_dir}" world doctor --json
  ) > "${doctor_json}"
}

print_doctor_failure() {
  local doctor_json="$1"
  local failure_reason
  failure_reason="$(jq -r '.world.netfilter_status.last_failure_reason // empty' "${doctor_json}")"
  if [[ -n "${failure_reason}" ]]; then
    printf '%s\n' "${failure_reason}" >&2
  fi
}

assert_allow_all_doctor() {
  local doctor_json="$1"
  if ! jq -e '
    .world.netfilter_status.requested == false and
    .world.netfilter_status.enabled == false and
    .world.netfilter_status.world_netfilter_enable_present == true
  ' "${doctor_json}" >/dev/null; then
    echo "ERROR: allow-all doctor assertions failed for ${doctor_json}" >&2
    jq '.world.netfilter_status' "${doctor_json}" >&2
    exit 1
  fi
}

assert_deny_all_doctor() {
  local doctor_json="$1"
  if ! jq -e '
    .world.netfilter_status.requested == true and
    .world.netfilter_status.enabled == true and
    .world.netfilter_status.world_netfilter_enable_present == true and
    .world.netfilter_status.last_failure_reason == null
  ' "${doctor_json}" >/dev/null; then
    echo "ERROR: deny-all doctor assertions failed for ${doctor_json}" >&2
    jq '.world.netfilter_status' "${doctor_json}" >&2
    exit 1
  fi
}

run_netfilter_posture() {
  local posture="$1"
  local net_allowed_yaml="$2"
  local expect_probe_success="$3"
  local fixture_home="$4"
  local substrate_home="$5"
  local project_dir="$6"
  local log_dir="$7"

  local prefix="${log_dir}/${posture}"
  local stdout_log="${prefix}-probe.stdout.log"
  local stderr_log="${prefix}-probe.stderr.log"
  local exit_log="${prefix}-probe.exit"
  local doctor_json="${prefix}-world-doctor.json"

  log "Running ${posture} posture smoke"
  write_smoke_policy "${substrate_home}" "${net_allowed_yaml}"

  local probe_rc=0
  if run_probe_with_capture "${fixture_home}" "${substrate_home}" "${project_dir}" "${stdout_log}" "${stderr_log}" "${exit_log}"; then
    probe_rc=0
  else
    probe_rc=$?
  fi

  capture_world_doctor "${fixture_home}" "${substrate_home}" "${project_dir}" "${doctor_json}"

  if [[ "${expect_probe_success}" == "yes" ]]; then
    if [[ "${probe_rc}" -ne 0 ]]; then
      echo "ERROR: ${posture} probe failed unexpectedly with exit ${probe_rc}" >&2
      print_doctor_failure "${doctor_json}"
      jq '.world.netfilter_status' "${doctor_json}" >&2
      exit 1
    fi
  else
    if [[ "${probe_rc}" -eq 0 ]]; then
      echo "ERROR: ${posture} probe succeeded unexpectedly" >&2
      print_doctor_failure "${doctor_json}"
      jq '.world.netfilter_status' "${doctor_json}" >&2
      exit 1
    fi
  fi

  if [[ "${posture}" == "allow-all" ]]; then
    assert_allow_all_doctor "${doctor_json}"
  else
    assert_deny_all_doctor "${doctor_json}"
  fi

  jq '.world.netfilter_status' "${doctor_json}" > "${prefix}-netfilter-status.json"
}

run_generic_smoke() {
  local trace_log
  trace_log="${SHIM_TRACE_LOG:-$HOME/.substrate/trace.jsonl}"

  rm -rf "${REPO_ROOT}/world-mac-smoke"
  "${SCRIPTS_ROOT}/lima-warm.sh"
  "${SUBSTRATE_BIN}" -c 'echo smoke-nonpty'
  "${SUBSTRATE_BIN}" --pty -c 'printf smoke-pty\n'
  mkdir -p "$(dirname "${trace_log}")"

  "${SUBSTRATE_BIN}" -c 'rm -rf world-mac-smoke'
  local payload_cmd
  payload_cmd="(cd /src 2>/dev/null || cd \"${REPO_ROOT}\") && (test -d world-mac-smoke || mkdir world-mac-smoke) && printf 'data\n' > world-mac-smoke/file.txt"
  "${SUBSTRATE_BIN}" -c "${payload_cmd}"

  if [[ ! -f "${trace_log}" ]]; then
    echo "ERROR: Trace log not found at ${trace_log}" >&2
    exit 1
  fi

  local span
  span="$(jq -r 'select(.event_type=="command_complete" and ((.fs_diff.mods? // []) | index("world-mac-smoke/file.txt") != null)) | .span_id' "${trace_log}" | tail -n 1)"

  if [[ -z "${span}" ]]; then
    echo "ERROR: failed to locate span id for world-mac-smoke command (${payload_cmd})" >&2
    echo "Last few trace lines:" >&2
    tail -n 20 "${trace_log}" >&2
    exit 1
  fi

  "${SUBSTRATE_BIN}" --replay "${span}" --replay-verbose
  "${SUBSTRATE_BIN}" --trace "${span}" | tee /tmp/world-mac-replay.json
  jq '.fs_diff | ((.writes // []) + (.mods // []))' /tmp/world-mac-replay.json | grep 'world-mac-smoke/file.txt'
}

run_bedpm_installer_conformance() {
  local smoke_cmd

  smoke_cmd="(cd /src 2>/dev/null || cd \"${REPO_ROOT}\") && bash docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh"

  log "Running BEDPM Linux smoke through the Lima-backed guest path"
  "${SCRIPTS_ROOT}/lima-warm.sh"
  "${SUBSTRATE_BIN}" -c "${smoke_cmd}"
}

run_netfilter_conformance() {
  local log_dir="$1"
  local fixture_home="${log_dir}/home"
  local substrate_home="${fixture_home}/.substrate"
  local project_dir="${log_dir}/no-workspace-project"

  mkdir -p "${log_dir}" "${fixture_home}" "${project_dir}"
  write_smoke_config "${substrate_home}"
  : > "${fixture_home}/.substrate/trace.jsonl"

  log "Using log directory ${log_dir}"
  SUBSTRATE_WORLD_NETFILTER_ENABLE=1 "${SCRIPTS_ROOT}/lima-warm.sh"

  run_netfilter_posture "allow-all" '["*"]' yes "${fixture_home}" "${substrate_home}" "${project_dir}" "${log_dir}"
  run_netfilter_posture "deny-all" '[]' no "${fixture_home}" "${substrate_home}" "${project_dir}" "${log_dir}"

  cat <<EOF
Netfilter conformance artifacts:
  ${log_dir}/allow-all-world-doctor.json
  ${log_dir}/deny-all-world-doctor.json
  ${log_dir}/allow-all-probe.stdout.log
  ${log_dir}/allow-all-probe.stderr.log
  ${log_dir}/deny-all-probe.stdout.log
  ${log_dir}/deny-all-probe.stderr.log
EOF
}

ensure_host_prereqs
ensure_substrate_binary

if [[ "${MODE}" == "netfilter-conformance" ]]; then
  if [[ -z "${LOG_DIR}" ]]; then
    LOG_DIR="$(default_netfilter_log_dir)"
  fi
  run_netfilter_conformance "${LOG_DIR}"
elif [[ "${MODE}" == "bedpm-installer-conformance" ]]; then
  run_bedpm_installer_conformance
else
  run_generic_smoke
fi
