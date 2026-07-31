#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -eq 0 ]]; then
  echo "Do not run this orchestration smoke script as root." >&2
  exit 1
fi

SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-${REPO_ROOT}/target/debug/substrate}"
WORLD_DOCTOR_JSON=""

cleanup() {
  if [[ -n "${WORLD_DOCTOR_JSON}" && -f "${WORLD_DOCTOR_JSON}" ]]; then
    rm -f "${WORLD_DOCTOR_JSON}"
  fi
}
trap cleanup EXIT

log() {
  printf '[mac-orchestration-smoke] %s\n' "$*"
}

note_routed_override_bypass() {
  if [[ -n "${SUBSTRATE_WORLD_SOCKET:-}" ]]; then
    log "Ignoring SUBSTRATE_WORLD_SOCKET during routed orchestration proof; it remains advanced/test/breakglass on macOS."
  fi
}

run_routed_proof_command() {
  env -u SUBSTRATE_WORLD_SOCKET "$@"
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
  require_cmd cargo "Install Rust via rustup."
  require_cmd jq
  require_cmd python3
}

run_repo_cmd() {
  log "$*"
  (
    cd "${REPO_ROOT}"
    "$@"
  )
}

ensure_substrate_binary() {
  if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
    log "Building substrate binary for orchestration smoke..."
    run_repo_cmd cargo build --bin substrate >/dev/null
  fi
}

run_routed_doctor_proof() {
  note_routed_override_bypass
  log "Running routed doctor proof before orchestration checks"
  run_routed_proof_command "${SUBSTRATE_BIN}" host doctor --json \
    | jq -e '.ok == true and .host.ok == true' >/dev/null
  WORLD_DOCTOR_JSON="$(mktemp)"
  run_routed_proof_command "${SUBSTRATE_BIN}" world doctor --json >"${WORLD_DOCTOR_JSON}"
  jq -e '.ok == true and .host.ok == true and .world.ok == true and .world.status == "ok"' \
    "${WORLD_DOCTOR_JSON}" >/dev/null
}

build_install_bootstrap_context_v1() {
  local selected_prefix="$1"
  local commitment="$2"

  python3 - "${selected_prefix}" "${commitment}" <<'PY'
import base64
import hashlib
import os
import pwd
import re
import sys

DOMAIN = "substrate.install_bootstrap_context"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def normalize_path(raw: str) -> str:
    if (
        not raw
        or raw == "/"
        or not raw.startswith("/")
        or raw.startswith("//")
        or any(ch in raw for ch in "\0\r\n")
    ):
        fail("invalid selected host prefix")
    parts = []
    for part in raw[1:].split("/"):
        if not part:
            continue
        if part in (".", ".."):
            fail("invalid selected host prefix")
        parts.append(part)
    if not parts:
        fail("invalid selected host prefix")
    return "/" + "/".join(parts)


def b64_encode(value: bytes) -> str:
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


selected_prefix, commitment = sys.argv[1:]
if re.fullmatch(r"[0-9a-f]{64}", commitment) is None:
    fail("invalid host context commitment")
entry = pwd.getpwuid(os.geteuid())
if os.geteuid() == 0 or not entry.pw_name or any(ch in entry.pw_name for ch in "\0\r\n"):
    fail("invalid current Unix principal")
if pwd.getpwnam(entry.pw_name).pw_uid != os.geteuid():
    fail("current Unix principal does not round-trip through the account database")
normalized_prefix = normalize_path(selected_prefix)
record = (
    f"domain={DOMAIN}\n"
    "version=1\n"
    f"selected_host_prefix={b64_encode(normalized_prefix.encode('utf-8'))}\n"
    f"host_substrate_home={b64_encode(normalized_prefix.encode('utf-8'))}\n"
    f"host_substrate_root={b64_encode(normalized_prefix.encode('utf-8'))}\n"
    "principal_kind=unix\n"
    f"principal_account={b64_encode(entry.pw_name.encode('utf-8'))}\n"
    f"principal_uid={os.geteuid()}\n"
).encode("ascii")
if hashlib.sha256(record).hexdigest() != commitment:
    fail("current Unix principal does not match the routed host context commitment")
record += f"host_context_commitment={commitment}\n".encode("ascii")
print(b64_encode(record))
print(entry.pw_dir)
PY
}

build_platform_bootstrap_mapping_v1() {
  local commitment="$1"
  local vm_name="$2"
  local machine_id="$3"
  local control_root="$4"
  local realized_home="$5"
  local account="$6"
  local uid="$7"
  local host_socket="$8"
  local guest_socket="$9"

  python3 - \
    "${commitment}" \
    "${vm_name}" \
    "${machine_id}" \
    "${control_root}" \
    "${realized_home}" \
    "${account}" \
    "${uid}" \
    "${host_socket}" \
    "${guest_socket}" <<'PY'
import base64
import re
import sys

DOMAIN = "substrate.platform_bootstrap_mapping"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def normalize_path(raw: str) -> str:
    if (
        not raw
        or raw == "/"
        or not raw.startswith("/")
        or raw.startswith("//")
        or any(ch in raw for ch in "\0\r\n")
    ):
        fail("invalid platform bootstrap mapping")
    parts = []
    for part in raw[1:].split("/"):
        if not part:
            continue
        if part in (".", ".."):
            fail("invalid platform bootstrap mapping")
        parts.append(part)
    if not parts:
        fail("invalid platform bootstrap mapping")
    return "/" + "/".join(parts)


def valid_text(raw: str) -> bool:
    return bool(raw) and all(ch not in "\0\r\n" for ch in raw)


def b64_encode(value: bytes) -> str:
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


(
    commitment,
    vm_name,
    machine_id,
    control_root,
    realized_home,
    account,
    uid,
    host_socket,
    guest_socket,
) = sys.argv[1:]
if re.fullmatch(r"[0-9a-f]{64}", commitment) is None:
    fail("invalid platform bootstrap mapping")
if re.fullmatch(r"[0-9a-f]{32}", machine_id) is None:
    fail("invalid platform bootstrap mapping")
if re.fullmatch(r"0|[1-9][0-9]*", uid) is None:
    fail("invalid platform bootstrap mapping")
if not valid_text(vm_name) or not valid_text(account):
    fail("invalid platform bootstrap mapping")
record = (
    f"domain={DOMAIN}\n"
    "version=1\n"
    f"host_context_commitment={commitment}\n"
    "platform_kind=lima\n"
    f"instance_name={b64_encode(vm_name.encode('utf-8'))}\n"
    f"guest_machine_id={machine_id}\n"
    f"host_platform_control_root={b64_encode(normalize_path(control_root).encode('utf-8'))}\n"
    f"realized_substrate_home={b64_encode(normalize_path(realized_home).encode('utf-8'))}\n"
    f"realized_principal_account={b64_encode(account.encode('utf-8'))}\n"
    f"realized_principal_uid={uid}\n"
    "transport_kind=lima\n"
    f"transport_host={b64_encode(normalize_path(host_socket).encode('utf-8'))}\n"
    f"transport_guest_socket={b64_encode(normalize_path(guest_socket).encode('utf-8'))}\n"
).encode("ascii")
print(b64_encode(record))
PY
}

run_typed_mac_backend_contract_validation() {
  [[ -n "${WORLD_DOCTOR_JSON}" && -f "${WORLD_DOCTOR_JSON}" ]] \
    || { echo "ERROR: routed world doctor JSON is unavailable" >&2; exit 1; }

  local selected_prefix
  local commitment
  local control_root
  local realized_home
  local realized_account
  local realized_uid
  local transport_host
  local transport_guest
  local vm_name
  local mapping_attested
  local carrier
  local host_account_home
  local machine_id
  local machine_id_again
  local mapping
  local commitment_preview

  selected_prefix="$(jq -er '.host.lima.selected_host_prefix | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  commitment="$(jq -er '.host.lima.host_context_commitment | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  control_root="$(jq -er '.host.lima.host_platform_control_root | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  realized_home="$(jq -er '.host.lima.guest_substrate_home | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  realized_account="$(jq -er '.host.lima.guest_principal_account | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  realized_uid="$(jq -er '.host.lima.guest_principal_uid | tostring | select(length > 0)' "${WORLD_DOCTOR_JSON}")"
  transport_host="$(jq -er '.host.lima.transport_host_socket | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  transport_guest="$(jq -er '.host.lima.transport_guest_socket | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  vm_name="$(jq -er '.host.lima.vm_name | select(type == "string" and length > 0)' "${WORLD_DOCTOR_JSON}")"
  mapping_attested="$(jq -er '.host.lima.platform_mapping_attested' "${WORLD_DOCTOR_JSON}")"
  [[ "${mapping_attested}" == "true" ]] || { echo "ERROR: world doctor did not attest platform mapping" >&2; exit 1; }

  mapfile -t _carrier_parts < <(build_install_bootstrap_context_v1 "${selected_prefix}" "${commitment}")
  [[ "${#_carrier_parts[@]}" -eq 2 ]] || { echo "ERROR: failed to build install bootstrap carrier" >&2; exit 1; }
  carrier="${_carrier_parts[0]}"
  host_account_home="${_carrier_parts[1]}"

  machine_id="$(env -u SUBSTRATE_WORLD_SOCKET HOME="${host_account_home}" LIMA_HOME="${control_root}" \
    limactl shell "${vm_name}" cat /etc/machine-id 2>/dev/null | tr -d '\r\n' || true)"
  machine_id_again="$(env -u SUBSTRATE_WORLD_SOCKET HOME="${host_account_home}" LIMA_HOME="${control_root}" \
    limactl shell "${vm_name}" cat /etc/machine-id 2>/dev/null | tr -d '\r\n' || true)"
  [[ -n "${machine_id}" && "${machine_id}" == "${machine_id_again}" ]] \
    || { echo "ERROR: unable to observe a stable Lima guest machine identity" >&2; exit 1; }

  mapping="$(build_platform_bootstrap_mapping_v1 \
    "${commitment}" \
    "${vm_name}" \
    "${machine_id}" \
    "${control_root}" \
    "${realized_home}" \
    "${realized_account}" \
    "${realized_uid}" \
    "${transport_host}" \
    "${transport_guest}")"

  commitment_preview="${commitment:0:12}..."
  log "Running typed pre-R3 contract validation with explicit projection (vm=${vm_name}, commitment=${commitment_preview})"
  run_repo_cmd cargo run -p world-mac-lima --example mac_backend_smoke -- \
    --install-bootstrap-context-v1 "${carrier}" \
    --platform-bootstrap-mapping-v1 "${mapping}" \
    --project-dir "${REPO_ROOT}"
}

ensure_host_prereqs
ensure_substrate_binary

log "Warming the Lima-backed world backend for typed contract proof"
"${SCRIPTS_ROOT}/lima-warm.sh"

run_routed_doctor_proof

run_typed_mac_backend_contract_validation

log "Running macOS shared-owner bootstrap routing proof"
run_repo_cmd cargo test -p shell shared_owner_macos_allows_lima_backed_bootstrap -- --nocapture

log "Running shared-owner ready-proof acceptance and mismatch rejection checks"
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_accepts_shared_world_attach_create_ready_proof -- --nocapture
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_accepts_replacement_ready_when_generation_advances -- --nocapture
run_repo_cmd cargo test -p shell --test persistent_session_client_v1 persistent_session_client_v1_rejects_invalid_shared_world_ready_proof -- --nocapture

log "Running retained world-member launch, reuse, and cancel checks"
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_first_targeted_world_turn_uses_initial_prompt_in_member_dispatch -- --nocapture
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_first_world_backed_command_lazily_launches_member_runtime -- --nocapture
run_repo_cmd cargo test -p shell --test repl_world_first_routing_v1 c3_targeted_world_turn_uses_typed_submit_route_without_relaunching_member -- --nocapture

log "macOS/Lima typed pre-R3 contract validation proof passed"
