#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -eq 0 ]]; then
  echo "Do not run this smoke script as root." >&2
  exit 1
fi

SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-${REPO_ROOT}/target/debug/substrate}"
CANONICAL_UNIT_SOURCE_DIR="${REPO_ROOT}/scripts/mac/lima/units"
VM_NAME="${SUBSTRATE_LIMA_VM_NAME:-${LIMA_VM_NAME:-substrate}}"
STAGED_WORKSPACE_CURRENT="${SUBSTRATE_LIMA_STAGED_WORKSPACE_CURRENT:-/var/lib/substrate/staged-workspace/current}"
RUN_GUEST_DIRECT_BREAKGLASS="${SUBSTRATE_MAC_SMOKE_INCLUDE_GUEST_DIRECT:-0}"

MODE="generic"
LOG_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/mac/smoke.sh [--gateway-conformance | --orchestration-conformance | --netfilter-conformance | --bedpm-installer-conformance] [--log-dir DIR]

Options:
  --gateway-conformance    Run the fixture-backed gateway lifecycle/status proof instead of the generic smoke
  --world-disabled-diagnostics
                           Run the world-disabled-diagnostics conformance smoke instead of the generic smoke
  --orchestration-conformance
                           Run the macOS/Lima orchestration conformance smoke
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
    --gateway-conformance)
      MODE="gateway-conformance"
      shift
      ;;
    --world-disabled-diagnostics)
      MODE="world-disabled-diagnostics"
      shift
      ;;
    --orchestration-conformance)
      MODE="orchestration-conformance"
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

note_routed_override_bypass() {
  if [[ -n "${SUBSTRATE_WORLD_SOCKET:-}" ]]; then
    log "Ignoring SUBSTRATE_WORLD_SOCKET during routed smoke proof; it remains advanced/test/breakglass on macOS."
  fi
}

run_routed_proof_command() {
  env -u SUBSTRATE_WORLD_SOCKET "$@"
}

run_guest_direct_gateway_compatibility_check() {
  local port="$1"
  log "Running guest-direct gateway compatibility check (breakglass only)"
  limactl shell "${VM_NAME}" curl --fail --silent "http://127.0.0.1:${port}/health" \
    | jq -e '.status == "ok" and .service == "substrate-gateway"' >/dev/null
}

run_guest_direct_readiness_diagnostics() {
  log "Running guest-direct readiness diagnostics (fallback/breakglass only)"
  limactl shell "${VM_NAME}" sudo test -x /usr/local/bin/substrate-world-service
  limactl shell "${VM_NAME}" sudo test -x /usr/local/bin/substrate-gateway
  limactl shell "${VM_NAME}" systemctl is-active --quiet substrate-world-service
}

host_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

run_rendered_unit_parity_proof() {
  local parity_tmp=""
  local expected_dir=""
  local actual_dir=""
  local vm_user=""
  local vm_home=""
  local guest_substrate_home=""
  local enable_netfilter="${SUBSTRATE_WORLD_NETFILTER_ENABLE:-0}"
  local expected_netfilter_env=""
  local expected_service_sha=""
  local expected_socket_sha=""
  local actual_service_sha=""
  local actual_socket_sha=""

  if [[ ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" || ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" ]]; then
    echo "ERROR: missing canonical unit sources under ${CANONICAL_UNIT_SOURCE_DIR}" >&2
    exit 1
  fi

  if ! command -v envsubst >/dev/null 2>&1; then
    echo "ERROR: envsubst is required to render canonical macOS guest units locally" >&2
    exit 1
  fi

  parity_tmp="$(mktemp -d)"
  expected_dir="${parity_tmp}/expected"
  actual_dir="${parity_tmp}/actual"
  mkdir -p "${expected_dir}" "${actual_dir}"

  vm_user="$(limactl shell "${VM_NAME}" id -un 2>/dev/null | tr -d '\r' || true)"
  if [[ -z "${vm_user}" ]]; then
    echo "ERROR: unable to determine the Lima guest user for rendered-unit parity proof" >&2
    exit 1
  fi

  vm_home="$(limactl shell "${VM_NAME}" getent passwd "${vm_user}" 2>/dev/null | cut -d: -f6 | tr -d '\r' || true)"
  if [[ -z "${vm_home}" ]]; then
    vm_home="/home/${vm_user}"
  fi
  guest_substrate_home="${vm_home}/.substrate"

  case "${enable_netfilter}" in
    1|true|yes|TRUE|YES)
      expected_netfilter_env="Environment=WORLD_NETFILTER_ENABLE=1"
      ;;
  esac

  SUBSTRATE_GUEST_HOME="${guest_substrate_home}" WORLD_NETFILTER_ENV="${expected_netfilter_env}" \
    envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" > "${expected_dir}/substrate-world-service.service"
  envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" > "${expected_dir}/substrate-world-service.socket"

  if ! limactl shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.service \
    | sed '/^# \//d' \
    | awk 'BEGIN { seen=0 } { if (!seen && $0 == "") next; seen=1; print }' > "${actual_dir}/substrate-world-service.service"; then
    echo "ERROR: unable to capture the loaded guest service unit via systemctl cat" >&2
    exit 1
  fi

  if ! limactl shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.socket \
    | sed '/^# \//d' \
    | awk 'BEGIN { seen=0 } { if (!seen && $0 == "") next; seen=1; print }' > "${actual_dir}/substrate-world-service.socket"; then
    echo "ERROR: unable to capture the loaded guest socket unit via systemctl cat" >&2
    exit 1
  fi

  expected_service_sha="$(host_sha256 "${expected_dir}/substrate-world-service.service")"
  expected_socket_sha="$(host_sha256 "${expected_dir}/substrate-world-service.socket")"
  actual_service_sha="$(host_sha256 "${actual_dir}/substrate-world-service.service")"
  actual_socket_sha="$(host_sha256 "${actual_dir}/substrate-world-service.socket")"

  if ! cmp -s "${expected_dir}/substrate-world-service.service" "${actual_dir}/substrate-world-service.service"; then
    echo "ERROR: guest service unit differs from the canonical rendered contract (expected sha256 ${expected_service_sha}, loaded guest sha256 ${actual_service_sha:-unknown})" >&2
    exit 1
  fi

  if ! cmp -s "${expected_dir}/substrate-world-service.socket" "${actual_dir}/substrate-world-service.socket"; then
    echo "ERROR: guest socket unit differs from the canonical rendered contract (expected sha256 ${expected_socket_sha}, loaded guest sha256 ${actual_socket_sha:-unknown})" >&2
    exit 1
  fi

  rm -rf "${parity_tmp}"
  log "Rendered unit parity proof passed for substrate-world-service.service/.socket"
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

prepare_host_gateway_smoke_auth() {
  local home_dir="$1"
  local auth_path="${home_dir}/.codex/auth.json"
  if [[ -f "${auth_path}" ]]; then
    printf 'present\n'
    return 0
  fi

  local tmp
  tmp="$(mktemp)"
  cat >"${tmp}" <<'JSON'
{
  "account_id": "acct_smoke",
  "access_token": "header.payload.signature"
}
JSON
  install -d -m0700 "${home_dir}/.codex"
  install -m0600 "${tmp}" "${auth_path}"
  rm -f "${tmp}"
  printf 'created\n'
}

cleanup_host_gateway_smoke_auth() {
  local home_dir="$1"
  rm -f "${home_dir}/.codex/auth.json"
}

write_gateway_smoke_config() {
  local substrate_home="$1"
  mkdir -p "${substrate_home}"
  cat >"${substrate_home}/config.yaml" <<'EOF'
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: cli:codex
EOF
}

write_gateway_smoke_policy() {
  local substrate_home="$1"
  mkdir -p "${substrate_home}"
  cat >"${substrate_home}/policy.yaml" <<'EOF'
id: gateway-smoke
name: gateway-smoke

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "cli:codex"
  secrets:
    env_allowed:
      - "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID"
      - "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"

agents:
  host_credentials:
    read:
      allowed_backends:
        - "cli:codex"

net_allowed: []
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

write_gateway_smoke_inventory() {
  local substrate_home="$1"
  mkdir -p "${substrate_home}/agents"
  cp "${REPO_ROOT}/config/agents/codex.yaml" "${substrate_home}/agents/codex.yaml"
}

run_gateway_lifecycle_proof() {
  local status_json=""
  local base_url=""
  local port=""
  local fixture_root=""
  local substrate_home=""
  local gateway_cwd="/"
  local codex_account_id="acct_smoke"
  local codex_access_token="header.payload.signature"

  log "Running gateway lifecycle proof"
  fixture_root="$(mktemp -d)"
  substrate_home="${fixture_root}/substrate-home"
  mkdir -p "${substrate_home}"
  write_gateway_smoke_config "${substrate_home}"
  write_gateway_smoke_policy "${substrate_home}"
  write_gateway_smoke_inventory "${substrate_home}"
  trap 'rm -rf "'"${fixture_root}"'"' RETURN

  # Slice 09 staged-workspace cutover removed guest reliance on the host checkout path.
  # Run lifecycle/status proofs from a stable cwd that exists on both host and guest,
  # while sourcing the gateway contract from the dedicated smoke SUBSTRATE_HOME fixture.
  (
    cd "${gateway_cwd}"
    run_routed_proof_command env SUBSTRATE_HOME="${substrate_home}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID="${codex_account_id}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN="${codex_access_token}" \
      "${SUBSTRATE_BIN}" world gateway sync
  )
  status_json="$(
    cd "${gateway_cwd}"
    run_routed_proof_command env SUBSTRATE_HOME="${substrate_home}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID="${codex_account_id}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN="${codex_access_token}" \
      "${SUBSTRATE_BIN}" world gateway status --json
  )"
  printf '%s\n' "${status_json}" | jq -e '
    .status == "available" and
    .client_wiring.openai_base_url == .client_wiring.anthropic_base_url
  ' >/dev/null

  (
    cd "${gateway_cwd}"
    run_routed_proof_command env SUBSTRATE_HOME="${substrate_home}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID="${codex_account_id}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN="${codex_access_token}" \
      "${SUBSTRATE_BIN}" world gateway restart
  )
  status_json="$(
    cd "${gateway_cwd}"
    run_routed_proof_command env SUBSTRATE_HOME="${substrate_home}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID="${codex_account_id}" \
      SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN="${codex_access_token}" \
      "${SUBSTRATE_BIN}" world gateway status --json
  )"
  base_url="$(printf '%s\n' "${status_json}" | jq -r '.client_wiring.openai_base_url')"
  port="$(printf '%s\n' "${base_url}" | sed -n 's#http://127\.0\.0\.1:\([0-9][0-9]*\)$#\1#p')"
  if [[ -z "${port}" ]]; then
    echo "ERROR: unable to derive gateway port from ${base_url}" >&2
    exit 1
  fi

  run_rendered_unit_parity_proof

  if [[ "${RUN_GUEST_DIRECT_BREAKGLASS}" == "1" ]]; then
    run_guest_direct_gateway_compatibility_check "${port}"
  else
    log "Skipping guest-direct gateway compatibility check; set SUBSTRATE_MAC_SMOKE_INCLUDE_GUEST_DIRECT=1 to run it as breakglass evidence."
  fi
}

run_dev_install_readiness_proof() {
  local install_prefix="$1"
  local install_bin="${install_prefix%/}/bin/substrate"
  local host_doctor_json=""
  local doctor_json=""

  log "Running macOS dev-install routed readiness proof"
  "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" --prefix "${install_prefix}" --profile debug

  if [[ ! -x "${install_bin}" ]]; then
    echo "ERROR: dev-install did not produce ${install_bin}" >&2
    exit 1
  fi

  if ! host_doctor_json="$(run_routed_proof_command env SUBSTRATE_HOME="${install_prefix}" SUBSTRATE_ROOT="${install_prefix}" \
    "${install_bin}" host doctor --json)"; then
    echo "ERROR: substrate host doctor --json failed during routed readiness proof" >&2
    run_guest_direct_readiness_diagnostics || true
    exit 1
  fi
  if ! printf '%s\n' "${host_doctor_json}" | jq -e '
    .ok == true and
    .host.ok == true
  ' >/dev/null; then
    echo "ERROR: substrate host doctor --json reported host readiness failure" >&2
    printf '%s\n' "${host_doctor_json}" >&2
    run_guest_direct_readiness_diagnostics || true
    exit 1
  fi

  if ! doctor_json="$(run_routed_proof_command env SUBSTRATE_HOME="${install_prefix}" SUBSTRATE_ROOT="${install_prefix}" \
    "${install_bin}" world doctor --json)"; then
    echo "ERROR: substrate world doctor --json failed during routed readiness proof" >&2
    run_guest_direct_readiness_diagnostics || true
    exit 1
  fi
  if ! printf '%s\n' "${doctor_json}" | jq -e '
    .ok == true and
    .host.ok == true and
    .world.ok == true and
    .world.status == "ok"
  ' >/dev/null; then
    echo "ERROR: substrate world doctor --json reported world readiness failure" >&2
    printf '%s\n' "${doctor_json}" >&2
    run_guest_direct_readiness_diagnostics || true
    exit 1
  fi

  if [[ "${RUN_GUEST_DIRECT_BREAKGLASS}" == "1" ]]; then
    run_guest_direct_readiness_diagnostics
  else
    log "Skipping guest-direct readiness diagnostics; set SUBSTRATE_MAC_SMOKE_INCLUDE_GUEST_DIRECT=1 to run them as breakglass evidence."
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
  run_routed_proof_command env \
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

run_world_disabled_diagnostics() {
  local slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-WDD2}"

  case "${slice_id}" in
    WDD0|WDD1|WDD2) ;;
    *)
      echo "ERROR: unsupported SUBSTRATE_SMOKE_SLICE_ID=${slice_id} (expected WDD0, WDD1, or WDD2)" >&2
      exit 2
      ;;
  esac

  WDD_WORKDIR="$(mktemp -d)"
  trap 'rm -rf "${WDD_WORKDIR:-}"' EXIT

  require_cmd jq

  require_contains() {
    local haystack="$1"
    local needle="$2"
    printf '%s\n' "$haystack" | grep -Fq "$needle" || {
      echo "ERROR: missing expected line: $needle" >&2
      return 1
    }
  }

  require_not_contains() {
    local haystack="$1"
    local needle="$2"
    if printf '%s\n' "$haystack" | grep -Fq "$needle"; then
      echo "ERROR: found forbidden substring: $needle" >&2
      return 1
    fi
  }

  run_json_capture() {
    local label="$1"
    shift

    local stdout_file stderr_file
    stdout_file="$(mktemp)"
    stderr_file="$(mktemp)"

    set +e
    "$@" 1>"${stdout_file}" 2>"${stderr_file}"
    local rc=$?
    set -e

    if [[ "${rc}" -ne 0 ]]; then
      echo "ERROR: ${label} (exit=${rc})" >&2
      cat "${stderr_file}" >&2 || true
      cat "${stdout_file}" >&2 || true
      return "${rc}"
    fi

    local json
    json="$(cat "${stdout_file}")"
    rm -f "${stdout_file}" "${stderr_file}"
    printf '%s' "${json}"
  }

  check_invalid_config() {
    local home
    home="$(mktemp -d)"
    printf 'world: [\n' > "${home}/config.yaml"

    local out rc

    set +e
    out="$(SUBSTRATE_HOME="${home}" "${SUBSTRATE_BIN}" shim doctor 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 2 ]] || { echo "ERROR: shim doctor invalid config expected exit=2, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }
    printf '%s\n' "${out}" | grep -Fq "config.yaml" || { echo "ERROR: shim doctor invalid config stderr must mention config.yaml" >&2; printf '%s\n' "${out}" >&2; return 1; }

    set +e
    out="$(SUBSTRATE_HOME="${home}" "${SUBSTRATE_BIN}" shim doctor --json 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 2 ]] || { echo "ERROR: shim doctor --json invalid config expected exit=2, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }
    printf '%s\n' "${out}" | grep -Fq "config.yaml" || { echo "ERROR: shim doctor --json invalid config stderr must mention config.yaml" >&2; printf '%s\n' "${out}" >&2; return 1; }

    set +e
    out="$(SUBSTRATE_HOME="${home}" "${SUBSTRATE_BIN}" health 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 2 ]] || { echo "ERROR: health invalid config expected exit=2, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }
    printf '%s\n' "${out}" | grep -Fq "config.yaml" || { echo "ERROR: health invalid config stderr must mention config.yaml" >&2; printf '%s\n' "${out}" >&2; return 1; }

    set +e
    out="$(SUBSTRATE_HOME="${home}" "${SUBSTRATE_BIN}" health --json 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 2 ]] || { echo "ERROR: health --json invalid config expected exit=2, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }
    printf '%s\n' "${out}" | grep -Fq "config.yaml" || { echo "ERROR: health --json invalid config stderr must mention config.yaml" >&2; printf '%s\n' "${out}" >&2; return 1; }

    rm -rf "${home}"
  }

  check_shim_doctor_disabled_and_broken() {
    local home
    home="$(mktemp -d)"

    local out rc
    set +e
    out="$(SUBSTRATE_HOME="${home}" SUBSTRATE_OVERRIDE_WORLD=disabled "${SUBSTRATE_BIN}" shim doctor 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 0 ]] || { echo "ERROR: shim doctor disabled expected exit=0, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }

    require_contains "${out}" "World backend:"
    require_contains "${out}" "  Status: disabled"
    require_contains "${out}" "  Next: run \`substrate world enable\` to provision"
    require_contains "${out}" "World deps:"
    require_contains "${out}" "  Status: skipped (world disabled)"
    require_not_contains "${out}" "  Error:"

    local json
    json="$(run_json_capture "shim doctor --json (disabled)" env SUBSTRATE_HOME="${home}" SUBSTRATE_OVERRIDE_WORLD=disabled "${SUBSTRATE_BIN}" shim doctor --json)"
    printf '%s\n' "${json}" | jq -e '
      .world.status == "disabled" and
      .world_deps.status == "skipped_disabled" and
      (.world | has("error") | not) and
      (.world | has("stderr") | not) and
      (.world | has("exit_code") | not) and
      (.world | has("details") | not) and
      (.world_deps | has("error") | not) and
      (.world_deps | has("report") | not)
    ' >/dev/null

    local sock="${home}/does-not-exist.sock"
    rm -f "${sock}" || true

    set +e
    out="$(SUBSTRATE_HOME="${home}" SUBSTRATE_WORLD_SOCKET="${sock}" "${SUBSTRATE_BIN}" --world shim doctor 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 0 ]] || { echo "ERROR: shim doctor enabled-but-broken expected exit=0, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }

    require_contains "${out}" "World backend:"
    require_contains "${out}" "  Status: needs attention"
    require_contains "${out}" "  Details:"
    require_contains "${out}" "  Applied:"
    require_not_contains "${out}" "  Status: disabled"

    json="$(run_json_capture "shim doctor --json (broken)" env SUBSTRATE_HOME="${home}" SUBSTRATE_WORLD_SOCKET="${sock}" "${SUBSTRATE_BIN}" --world shim doctor --json)"
    printf '%s\n' "${json}" | jq -e '
      .world.status == "needs_attention" and
      (.world.details | type == "object") and
      .world_deps.status == "error" and
      (.world_deps.report | type == "object")
    ' >/dev/null

    rm -rf "${home}"
  }

  check_health_disabled_and_broken() {
    local home
    home="$(mktemp -d)"

    local out rc
    set +e
    out="$(SUBSTRATE_HOME="${home}" SUBSTRATE_OVERRIDE_WORLD=disabled "${SUBSTRATE_BIN}" health 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 0 ]] || { echo "ERROR: health disabled expected exit=0, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }

    require_contains "${out}" "World backend: disabled"
    require_contains "${out}" "  Next: run \`substrate world enable\` to provision"
    require_contains "${out}" "World deps: skipped (world disabled)"
    require_not_contains "${out}" "substrate world deps current"

    local json
    json="$(run_json_capture "health --json (disabled)" env SUBSTRATE_HOME="${home}" SUBSTRATE_OVERRIDE_WORLD=disabled "${SUBSTRATE_BIN}" health --json)"
    printf '%s\n' "${json}" | jq -e '
      .shim.world.status == "disabled" and
      .shim.world_deps.status == "skipped_disabled" and
      .summary.world_ok == null and
      (.summary | has("world_error") | not) and
      (.summary | has("world_deps_error") | not) and
      .summary.world_deps_missing == [] and
      .summary.world_deps_blocked == []
    ' >/dev/null

    local sock="${home}/does-not-exist.sock"
    rm -f "${sock}" || true

    set +e
    out="$(SUBSTRATE_HOME="${home}" SUBSTRATE_WORLD_SOCKET="${sock}" "${SUBSTRATE_BIN}" --world health 2>&1)"
    rc=$?
    set -e
    [[ "${rc}" -eq 0 ]] || { echo "ERROR: health enabled-but-broken expected exit=0, got=${rc}" >&2; printf '%s\n' "${out}" >&2; return 1; }

    require_contains "${out}" "World backend: needs attention"
    require_contains "${out}" "World deps: unavailable"
    require_contains "${out}" "Overall status: attention required"
    require_contains "${out}" "  - world backend health check failed"
    require_not_contains "${out}" "World backend: disabled"

    json="$(run_json_capture "health --json (broken)" env SUBSTRATE_HOME="${home}" SUBSTRATE_WORLD_SOCKET="${sock}" "${SUBSTRATE_BIN}" --world health --json)"
    printf '%s\n' "${json}" | jq -e '
      .shim.world.status == "needs_attention" and
      .summary.world_ok == false and
      .shim.world_deps.status == "error" and
      (.summary.world_deps_error | type == "string" and length > 0)
    ' >/dev/null

    rm -rf "${home}"
  }

  echo "INFO: world-disabled-diagnostics macOS smoke slice=${slice_id}"
  check_invalid_config
  if [[ "${slice_id}" == "WDD0" ]]; then
    echo "OK: world-disabled-diagnostics macOS smoke (${slice_id})"
    return 0
  fi

  check_shim_doctor_disabled_and_broken
  if [[ "${slice_id}" == "WDD1" ]]; then
    echo "OK: world-disabled-diagnostics macOS smoke (${slice_id})"
    return 0
  fi

  check_health_disabled_and_broken
  echo "OK: world-disabled-diagnostics macOS smoke (${slice_id})"
}

run_generic_smoke() {
  local trace_log
  local dev_install_prefix
  local dev_install_bin
  local routed_cwd="/"
  local -a dev_install_env
  dev_install_prefix="$(mktemp -d)"
  dev_install_bin="${dev_install_prefix}/bin/substrate"
  trace_log="${SHIM_TRACE_LOG:-${dev_install_prefix}/trace.jsonl}"
  dev_install_env=(
    env
    SUBSTRATE_HOME="${dev_install_prefix}"
    SUBSTRATE_ROOT="${dev_install_prefix}"
    SHIM_TRACE_LOG="${trace_log}"
    SUBSTRATE_WORLD_PROJECT_DIR="${STAGED_WORKSPACE_CURRENT}"
  )

  note_routed_override_bypass
  rm -rf "${REPO_ROOT}/world-mac-smoke"
  run_dev_install_readiness_proof "${dev_install_prefix}"
  run_gateway_lifecycle_proof
  (
    cd "${routed_cwd}"
    run_routed_proof_command "${dev_install_env[@]}" "${dev_install_bin}" --world -c 'echo smoke-nonpty'
  )
  (
    cd "${routed_cwd}"
    run_routed_proof_command "${dev_install_env[@]}" "${dev_install_bin}" --world --pty -c 'printf smoke-pty\n'
  )
  mkdir -p "$(dirname "${trace_log}")"

  local payload_cmd
  payload_cmd="mkdir -p world-mac-smoke && printf 'smoke-%s\n' '$(date -u +%s)' > world-mac-smoke/file.txt"
  (
    cd "${routed_cwd}"
    run_routed_proof_command "${dev_install_env[@]}" "${dev_install_bin}" --world -c "${payload_cmd}"
  )

  if [[ ! -f "${trace_log}" ]]; then
    echo "ERROR: Trace log not found at ${trace_log}" >&2
    exit 1
  fi

  local span
  span="$(jq -r '
    select(
      .event_type == "command_complete"
      and (((.fs_diff.writes? // []) + (.fs_diff.mods? // [])) | index("world-mac-smoke/file.txt") != null)
    )
    | .span_id
  ' "${trace_log}" | tail -n 1)"

  if [[ -z "${span}" ]]; then
    echo "ERROR: failed to locate span id for world-mac-smoke command (${payload_cmd})" >&2
    echo "Last few trace lines:" >&2
    tail -n 20 "${trace_log}" >&2
    exit 1
  fi

  (
    cd "${routed_cwd}"
    run_routed_proof_command "${dev_install_env[@]}" "${dev_install_bin}" --world --replay "${span}" --replay-verbose
  )
  (
    cd "${routed_cwd}"
    run_routed_proof_command "${dev_install_env[@]}" "${dev_install_bin}" --world --trace "${span}" | tee /tmp/world-mac-replay.json
  )
  jq '.fs_diff | ((.writes // []) + (.mods // []))' /tmp/world-mac-replay.json | grep 'world-mac-smoke/file.txt'
  rm -rf "${dev_install_prefix}"
}

run_bedpm_installer_conformance() {
  echo "ERROR: --bedpm-installer-conformance was retired with project-management pack automation." >&2
  echo "See docs/PROJECT_MANAGEMENT_RETIREMENT.md for the replacement workflow." >&2
  exit 2
}

run_orchestration_conformance() {
  log "Running macOS/Lima orchestration conformance smoke"
  "${SCRIPTS_ROOT}/orchestration-smoke.sh"
}

run_netfilter_conformance() {
  local log_dir="$1"
  local fixture_home="${log_dir}/home"
  local substrate_home="${fixture_home}/.substrate"
  local project_dir="${log_dir}/no-workspace-project"

  mkdir -p "${log_dir}" "${fixture_home}" "${project_dir}"
  write_smoke_config "${substrate_home}"
  : > "${fixture_home}/.substrate/trace.jsonl"

  note_routed_override_bypass
  log "Using log directory ${log_dir}"
  SUBSTRATE_WORLD_NETFILTER_ENABLE=1 "${SCRIPTS_ROOT}/lima-warm.sh"
  run_gateway_lifecycle_proof

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

ensure_substrate_binary

if [[ "${MODE}" == "world-disabled-diagnostics" ]]; then
  run_world_disabled_diagnostics
elif [[ "${MODE}" == "orchestration-conformance" ]]; then
  ensure_host_prereqs
  run_orchestration_conformance
elif [[ "${MODE}" == "gateway-conformance" ]]; then
  ensure_host_prereqs
  run_gateway_lifecycle_proof
elif [[ "${MODE}" == "netfilter-conformance" ]]; then
  ensure_host_prereqs
  if [[ -z "${LOG_DIR}" ]]; then
    LOG_DIR="$(default_netfilter_log_dir)"
  fi
  run_netfilter_conformance "${LOG_DIR}"
elif [[ "${MODE}" == "bedpm-installer-conformance" ]]; then
  ensure_host_prereqs
  run_bedpm_installer_conformance
else
  ensure_host_prereqs
  run_generic_smoke
fi
