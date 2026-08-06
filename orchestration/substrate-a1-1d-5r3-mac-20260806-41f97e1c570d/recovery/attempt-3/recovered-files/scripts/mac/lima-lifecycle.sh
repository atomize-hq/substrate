#!/usr/bin/env bash
set -euo pipefail

SOURCE_PATH="${BASH_SOURCE[0]}"
while [[ -L "${SOURCE_PATH}" ]]; do
    SOURCE_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
    SOURCE_PATH="$(readlink "${SOURCE_PATH}")"
    [[ "${SOURCE_PATH}" != /* ]] && SOURCE_PATH="${SOURCE_DIR}/${SOURCE_PATH}"
done
SCRIPT_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"

ACTION="${1:-}"
[[ -n "${ACTION}" ]] || { printf '%s\n' 'missing mapped Lima lifecycle action' >&2; exit 2; }
shift
INSTALL_PREFIX=""
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PLATFORM_BOOTSTRAP_MAPPING_V1=""
EXECUTOR_BUILD_EVIDENCE_V1=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install-prefix)
            [[ $# -ge 2 && -z "${INSTALL_PREFIX}" && -n "$2" ]] || { printf '%s\n' 'invalid --install-prefix' >&2; exit 2; }
            INSTALL_PREFIX="$2"; shift 2 ;;
        --install-bootstrap-context-v1)
            [[ $# -ge 2 && -z "${INSTALL_BOOTSTRAP_CONTEXT_V1}" && -n "$2" ]] || { printf '%s\n' 'invalid --install-bootstrap-context-v1' >&2; exit 2; }
            INSTALL_BOOTSTRAP_CONTEXT_V1="$2"; shift 2 ;;
        --platform-bootstrap-mapping-v1)
            [[ $# -ge 2 && -z "${PLATFORM_BOOTSTRAP_MAPPING_V1}" && -n "$2" ]] || { printf '%s\n' 'invalid --platform-bootstrap-mapping-v1' >&2; exit 2; }
            PLATFORM_BOOTSTRAP_MAPPING_V1="$2"; shift 2 ;;
        --executor-build-evidence-v1)
            [[ $# -ge 2 && -z "${EXECUTOR_BUILD_EVIDENCE_V1}" && -n "$2" ]] || { printf '%s\n' 'invalid --executor-build-evidence-v1' >&2; exit 2; }
            EXECUTOR_BUILD_EVIDENCE_V1="$2"; shift 2 ;;
        *) printf 'unexpected mapped Lima lifecycle argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

load_mapped_lifecycle_v1() {
    [[ "${INSTALL_PREFIX}" == /* && "${INSTALL_PREFIX}" != / && "${INSTALL_PREFIX}" != *$'\n'* && "${INSTALL_PREFIX}" != *$'\r'* ]] || {
        printf '%s\n' 'mapped Lima lifecycle requires one normalized install prefix' >&2; return 1;
    }
    [[ -n "${INSTALL_BOOTSTRAP_CONTEXT_V1}" && -n "${PLATFORM_BOOTSTRAP_MAPPING_V1}" && -n "${EXECUTOR_BUILD_EVIDENCE_V1}" ]] || {
        printf '%s\n' 'mapped Lima lifecycle requires exact carrier, mapping, and ExecutorBuildEvidenceV1' >&2; return 1;
    }
    python3 - "${INSTALL_PREFIX}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "${PLATFORM_BOOTSTRAP_MAPPING_V1}" "${EXECUTOR_BUILD_EVIDENCE_V1}" <<'LIFECYCLE_VALIDATE'
import base64
import hashlib
import json
import re
import sys

prefix, carrier, mapping, evidence_raw = sys.argv[1:]
B64 = re.compile(r"[A-Za-z0-9_-]+")
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")

def fail():
    raise ValueError("invalid mapped Lima lifecycle authority")

def decode(value):
    if not B64.fullmatch(value):
        fail()
    raw = base64.urlsafe_b64decode(value.encode() + b"=" * ((-len(value)) % 4))
    if base64.urlsafe_b64encode(raw).rstrip(b"=").decode() != value:
        fail()
    return raw.decode("utf-8")

def exact_lines(raw, keys):
    if not raw.endswith("\n") or "\r" in raw or "\0" in raw:
        fail()
    lines = raw[:-1].split("\n")
    if len(lines) != len(keys):
        fail()
    values = {}
    for key, line in zip(keys, lines):
        if line.count("=") != 1:
            fail()
        actual, value = line.split("=", 1)
        if actual != key:
            fail()
        values[key] = value
    return values

def inner(value):
    return decode(value)

try:
    carrier_keys = ("domain", "version", "selected_host_prefix", "host_substrate_home", "host_substrate_root", "principal_kind", "principal_account", "principal_uid", "host_context_commitment")
    c = exact_lines(decode(carrier), carrier_keys)
    if c["domain"] != "substrate.install_bootstrap_context" or c["version"] != "1" or c["principal_kind"] != "unix":
        fail()
    if inner(c["selected_host_prefix"]) != prefix or inner(c["host_substrate_home"]) != prefix or inner(c["host_substrate_root"]) != prefix:
        fail()
    if not inner(c["principal_account"]) or not re.fullmatch(r"[1-9][0-9]*", c["principal_uid"]):
        fail()
    commitment_input = ("domain=substrate.install_bootstrap_context\nversion=1\nselected_host_prefix=" + c["selected_host_prefix"] + "\nhost_substrate_home=" + c["host_substrate_home"] + "\nhost_substrate_root=" + c["host_substrate_root"] + "\nprincipal_kind=unix\nprincipal_account=" + c["principal_account"] + "\nprincipal_uid=" + c["principal_uid"] + "\n").encode()
    if not HEX64.fullmatch(c["host_context_commitment"]) or hashlib.sha256(commitment_input).hexdigest() != c["host_context_commitment"]:
        fail()

    mapping_keys = ("domain", "version", "host_context_commitment", "platform_kind", "instance_name", "guest_machine_id", "host_platform_control_root", "realized_substrate_home", "realized_principal_account", "realized_principal_uid", "transport_kind", "transport_host", "transport_guest_socket")
    m = exact_lines(decode(mapping), mapping_keys)
    if m["domain"] != "substrate.platform_bootstrap_mapping" or m["version"] != "1" or m["host_context_commitment"] != c["host_context_commitment"]:
        fail()
    if m["platform_kind"] != "lima" or m["transport_kind"] != "lima" or not inner(m["instance_name"]):
        fail()
    if not re.fullmatch(r"[0-9a-f]{32}", m["guest_machine_id"]) or not inner(m["host_platform_control_root"]):
        fail()
    if not inner(m["realized_substrate_home"]) or not inner(m["realized_principal_account"]) or not re.fullmatch(r"[1-9][0-9]*", m["realized_principal_uid"]):
        fail()
    if inner(m["transport_host"]) != prefix + "/sock/agent.sock" or inner(m["transport_guest_socket"]) != "/run/substrate.sock":
        fail()

    evidence = json.loads(evidence_raw)
    required = {"schema_owner", "schema_version", "source_commit", "source_tree", "source_ref", "artifact_sha256", "artifact_identity", "target_triple", "tool_versions"}
    if set(evidence) != required or evidence["schema_owner"] != "substrate.executor-build-evidence" or evidence["schema_version"] != 1:
        fail()
    if not HEX40.fullmatch(evidence["source_commit"]) or not HEX40.fullmatch(evidence["source_tree"]) or not HEX64.fullmatch(evidence["artifact_sha256"]):
        fail()
    if not all(isinstance(evidence[key], str) and evidence[key] for key in ("source_ref", "artifact_identity", "target_triple")) or not isinstance(evidence["tool_versions"], dict):
        fail()
except Exception:
    raise SystemExit("invalid mapped Lima lifecycle authority")
LIFECYCLE_VALIDATE
}

invoke_mac_lifecycle_executor() {
    local executor="/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1"
    [[ -x "${executor}" ]] || {
        printf '%s\n' 'mapped Lima lifecycle executor is unavailable' >&2; return 1;
    }
    python3 - \
        "${ACTION}" \
        "${INSTALL_PREFIX}" \
        "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        "${EXECUTOR_BUILD_EVIDENCE_V1}" <<'LIFECYCLE_JSON' | "${executor}" lima-action
import json
import sys

try:
    evidence = json.loads(sys.argv[5])
except (TypeError, ValueError) as error:
    raise SystemExit(f"invalid ExecutorBuildEvidenceV1 JSON: {error}")
if not isinstance(evidence, dict):
    raise SystemExit("ExecutorBuildEvidenceV1 must be a JSON object")
print(json.dumps({
    "action": sys.argv[1],
    "install_prefix": sys.argv[2],
    "install_bootstrap_context_v1": sys.argv[3],
    "platform_bootstrap_mapping_v1": sys.argv[4],
    "executor_build_evidence": evidence,
}, separators=(",", ":")))
LIFECYCLE_JSON
}

install_mapped_lima_state_v1() { invoke_mac_lifecycle_executor; }
restore_mapped_lima_state_v1() { invoke_mac_lifecycle_executor; }
install_mac_publisher_v1() { invoke_mac_lifecycle_executor; }
bootstrap_lima_guest_publisher_v1() { invoke_mac_lifecycle_executor; }
retain_lima_guest_bootstrap_channel_v1() { invoke_mac_lifecycle_executor; }
display_lima_guest_pairing_challenge_v1() { invoke_mac_lifecycle_executor; }
join_lima_guest_bootstrap_transcript_v1() { invoke_mac_lifecycle_executor; }
record_lima_guest_pre_state_v1() { invoke_mac_lifecycle_executor; }
install_exact_guest_artifacts_v1() { invoke_mac_lifecycle_executor; }
persist_lima_guest_unused_proof_v1() { invoke_mac_lifecycle_executor; }
acknowledge_lima_guest_unused_proof_v1() { invoke_mac_lifecycle_executor; }
restore_lima_guest_state_v1() { invoke_mac_lifecycle_executor; }
retire_lima_guest_test_publisher_v1() { invoke_mac_lifecycle_executor; }

main() {
    load_mapped_lifecycle_v1
    case "${ACTION}" in
        ensure-vm-ready|stage-workspace|install-guest-artifacts|configure-guest|stop|destroy-vm)
            install_mapped_lima_state_v1 ;;
        restore-guest|retire-test-publisher) restore_mapped_lima_state_v1 ;;
        *) printf 'unsupported mapped Lima lifecycle action: %s\n' "${ACTION}" >&2; return 2 ;;
    esac
}

main
