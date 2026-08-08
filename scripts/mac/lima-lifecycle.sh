#!/usr/bin/env bash
set -euo pipefail

# This wrapper is deliberately a two-tag membrane.  In particular it does not accept an
# action selector and it never invokes the privileged executor directly.  The fixed control
# binary owns the closed request decoder and the fixed XPC request kind.
TAG="${1:-}"
[[ -n "${TAG}" ]] || { printf '%s\n' 'missing mapped lifecycle tag' >&2; exit 2; }
shift

INSTALL_PREFIX=""
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PLATFORM_BOOTSTRAP_MAPPING_V1=""
EXECUTOR_BUILD_EVIDENCE_V1=""
PUBLISHER_REQUEST_V1=""
LIMA_STAGE_ONE_AUTHORIZATION_V1=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install-prefix) [[ $# -ge 2 && -z "${INSTALL_PREFIX}" && -n "$2" ]] || exit 2; INSTALL_PREFIX="$2"; shift 2 ;;
        --install-bootstrap-context-v1) [[ $# -ge 2 && -z "${INSTALL_BOOTSTRAP_CONTEXT_V1}" && -n "$2" ]] || exit 2; INSTALL_BOOTSTRAP_CONTEXT_V1="$2"; shift 2 ;;
        --platform-bootstrap-mapping-v1) [[ $# -ge 2 && -z "${PLATFORM_BOOTSTRAP_MAPPING_V1}" && -n "$2" ]] || exit 2; PLATFORM_BOOTSTRAP_MAPPING_V1="$2"; shift 2 ;;
        --executor-build-evidence-v1) [[ $# -ge 2 && -z "${EXECUTOR_BUILD_EVIDENCE_V1}" && -n "$2" ]] || exit 2; EXECUTOR_BUILD_EVIDENCE_V1="$2"; shift 2 ;;
        --publisher-request-v1) [[ $# -ge 2 && -z "${PUBLISHER_REQUEST_V1}" && -n "$2" ]] || exit 2; PUBLISHER_REQUEST_V1="$2"; shift 2 ;;
        --lima-stage-one-authorization-v1) [[ $# -ge 2 && -z "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" && -n "$2" ]] || exit 2; LIMA_STAGE_ONE_AUTHORIZATION_V1="$2"; shift 2 ;;
        *) printf '%s\n' 'unexpected mapped lifecycle argument' >&2; exit 2 ;;
    esac
done

load_mapped_lifecycle_v1() {
    [[ "${INSTALL_PREFIX}" == /* && "${INSTALL_PREFIX}" != / && "${INSTALL_PREFIX}" != *$'\n'* && "${INSTALL_PREFIX}" != *$'\r'* ]] || {
        printf '%s\n' 'mapped lifecycle requires one normalized install prefix' >&2; return 1;
    }
    [[ -n "${INSTALL_BOOTSTRAP_CONTEXT_V1}" && -n "${EXECUTOR_BUILD_EVIDENCE_V1}" ]] || {
        printf '%s\n' 'mapped lifecycle requires its exact IH carrier and ExecutorBuildEvidenceV1 join' >&2; return 1;
    }
    python3 - "${TAG}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "${PLATFORM_BOOTSTRAP_MAPPING_V1}" "${EXECUTOR_BUILD_EVIDENCE_V1}" "${PUBLISHER_REQUEST_V1}" "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" <<'PY'
import json
import re
import sys

tag, carrier, mapping, evidence_raw, publisher_raw, stage_raw = sys.argv[1:]
allowed = {
    "mac.lima.instance": {"start", "stop", "remove", "restore"},
    "mac.lima.staged-workspace": {"create", "replace", "remove", "restore"},
    "mac.lima.guest-group": {"create", "remove", "restore"},
    "mac.lima.guest-private-home": {"create", "remove", "restore"},
    "mac.lima.layout-sentinel": {"create", "replace", "remove", "restore"},
    "mac.lima.publisher-executor": {"create", "replace", "remove", "restore"},
    "mac.lima.publisher-state-directory": {"create", "remove"},
    "mac.lima.publisher-service-unit": {"create", "restore"},
    "mac.lima.publisher-socket-unit": {"create", "restore"},
    "mac.lima.publisher-signing-key": {"create"},
    "mac.lima.publisher-current-anchor": {"create", "replace"},
    "mac.lima.publisher-bootstrap-intent": {"create", "replace"},
    "mac.host.known-hosts-entry": {"create", "replace", "remove", "restore"},
}
def closed_pair(role, action):
    if role in allowed:
        return action in allowed[role]
    if re.fullmatch(r"mac\.lima\.guest-(binary|unit)\([a-z0-9-]+\)", role):
        return action in {"create", "replace", "remove", "restore"}
    if re.fullmatch(r"mac\.lima\.guest-(directory|membership)\([^()]+\)", role):
        return action in {"create", "remove", "restore"}
    if re.fullmatch(r"mac\.lima\.guest-service-state\((service|socket)\)", role):
        return action in {"enable", "disable", "start", "stop", "restore"}
    return False
def fail():
    raise SystemExit("invalid mapped lifecycle authority")
try:
    if tag not in ("stage_one_create", "post_pm_action"):
        fail()
    if not carrier:
        fail()
    evidence = json.loads(evidence_raw)
    if not isinstance(evidence, dict) or evidence.get("schema_owner") != "substrate.executor-build-evidence" or evidence.get("schema_version") != 1:
        fail()
    if tag == "stage_one_create":
        if publisher_raw or not stage_raw or mapping:
            fail()
        stage = json.loads(stage_raw)
        if not isinstance(stage, dict) or stage.get("schema_owner") != "substrate.lima-stage-one-authorization" or stage.get("schema_version") != 1 or stage.get("expected_absent") is not True:
            fail()
    else:
        if stage_raw or not publisher_raw or not mapping:
            fail()
        request = json.loads(publisher_raw)
        required = {"host_context_commitment", "platform_mapping_commitment", "scope_id", "current_anchor_counter", "current_anchor_sha256", "manifest_generation", "manifest_sha256", "role", "action", "object_identity", "requester_principal", "attempt_nonce", "expected_executor_build"}
        if not isinstance(request, dict) or set(request) != required:
            fail()
        role, action = request.get("role"), request.get("action")
        if not closed_pair(role, action) or not isinstance(request.get("object_identity"), dict) or not isinstance(request.get("expected_executor_build"), dict):
            fail()
except (TypeError, ValueError, json.JSONDecodeError):
    fail()
PY
}

invoke_mac_lifecycle_control_v1() {
    local control="${INSTALL_PREFIX}/bin/substrate-lifecycle-control"
    [[ -x "${control}" ]] || { printf '%s\n' 'fixed mapped lifecycle control is unavailable' >&2; return 1; }
    python3 - "${TAG}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "${PLATFORM_BOOTSTRAP_MAPPING_V1}" "${EXECUTOR_BUILD_EVIDENCE_V1}" "${PUBLISHER_REQUEST_V1}" "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" <<'PY' | "${control}" submit-mapped-lifecycle-v1
import json
import sys
tag, carrier, mapping, evidence, publisher, stage = sys.argv[1:]
request = {
    "tag": tag,
    "install_bootstrap_context_v1": carrier,
    "executor_build_evidence": json.loads(evidence),
}
if tag == "stage_one_create":
    request["lima_stage_one_authorization_v1"] = json.loads(stage)
else:
    request["platform_bootstrap_mapping_v1"] = mapping
    request["publisher_request"] = json.loads(publisher)
print(json.dumps(request, separators=(",", ":")))
PY
}

main() {
    load_mapped_lifecycle_v1
    case "${TAG}" in
        stage_one_create) invoke_mac_lifecycle_control_v1 ;;
        post_pm_action) invoke_mac_lifecycle_control_v1 ;;
        *) printf '%s\n' 'unsupported mapped lifecycle tag' >&2; return 2 ;;
    esac
}

main
