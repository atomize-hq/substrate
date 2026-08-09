#!/usr/bin/env bash
set -euo pipefail

# The canonical installer route admits only the signed Stage-1 result.  This wrapper also retains
# the existing separately validated ordinary closed post-PM branch for lima-stop; it is not an install
# sequence and never accepts a catalogue or a caller-specified Stage-1 continuation.
TAG="${1:-}"
case "${TAG}" in
    stage_one_create|post_pm_action) ;;
    *) printf '%s\n' 'unsupported mapped lifecycle tag' >&2; exit 2 ;;
esac
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
    [[ -n "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]] || {
        printf '%s\n' 'mapped lifecycle requires its exact IH carrier' >&2; return 1;
    }
    python3 - "${TAG}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "${PLATFORM_BOOTSTRAP_MAPPING_V1}" "${EXECUTOR_BUILD_EVIDENCE_V1}" "${PUBLISHER_REQUEST_V1}" "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" <<'PY'
import json
import re
import sys

tag, carrier, mapping, evidence_raw, publisher_raw, stage_raw = sys.argv[1:]

def fail():
    raise SystemExit("invalid mapped lifecycle authority")

def closed_pair(role, action):
    create_replace_remove_restore = {"create", "replace", "remove", "restore"}
    create_remove_restore = {"create", "remove", "restore"}
    if role == "mac.lima.instance":
        return action in {"start", "stop", "remove", "restore"}
    if role in {
        "mac.lima.staged-workspace",
        "mac.lima.layout-sentinel",
        "mac.lima.publisher-executor",
        "mac.host.known-hosts-entry",
    }:
        return action in create_replace_remove_restore
    if role in {"mac.lima.guest-group", "mac.lima.guest-private-home"}:
        return action in create_remove_restore
    if role == "mac.lima.publisher-state-directory":
        return action in {"create", "remove"}
    if role in {"mac.lima.publisher-service-unit", "mac.lima.publisher-socket-unit"}:
        return action in {"create", "restore"}
    if role == "mac.lima.publisher-signing-key":
        return action == "create"
    if role in {"mac.lima.publisher-current-anchor", "mac.lima.publisher-bootstrap-intent"}:
        return action in {"create", "replace"}
    if role in {
        "mac.lima.guest-binary(substrate-world-service)",
        "mac.lima.guest-binary(substrate-gateway)",
        "mac.lima.guest-binary(substrate)",
        "mac.lima.guest-unit(service)",
        "mac.lima.guest-unit(socket)",
    }:
        return action in create_replace_remove_restore
    if re.fullmatch(r"mac\\.lima\\.guest-(directory|membership)\\([^()]+\\)", role):
        return action in create_remove_restore
    if re.fullmatch(r"mac\\.lima\\.guest-service-state\\((service|socket)\\)", role):
        return action in {"enable", "disable", "start", "stop", "restore"}
    return False

try:
    if not carrier:
        fail()
    if tag == "stage_one_create":
        if mapping or evidence_raw or publisher_raw or not stage_raw:
            fail()
        stage = json.loads(stage_raw)
        if not isinstance(stage, dict) or stage.get("schema_owner") != "substrate.lima-stage-one-authorization" or stage.get("schema_version") != 1 or stage.get("expected_absent") is not True:
            fail()
        template = stage.get("successor_template")
        evidence = template.get("executor_build_evidence") if isinstance(template, dict) else None
        if not isinstance(evidence, dict) or evidence.get("schema_owner") != "substrate.executor-build-evidence" or evidence.get("schema_version") != 1:
            fail()
    elif tag == "post_pm_action":
        if stage_raw or not mapping or not evidence_raw or not publisher_raw:
            fail()
        evidence = json.loads(evidence_raw)
        if not isinstance(evidence, dict) or evidence.get("schema_owner") != "substrate.executor-build-evidence" or evidence.get("schema_version") != 1:
            fail()
        request = json.loads(publisher_raw)
        required = {"host_context_commitment", "platform_mapping_commitment", "scope_id", "current_anchor_counter", "current_anchor_sha256", "manifest_generation", "manifest_sha256", "role", "action", "object_identity", "requester_principal", "attempt_nonce", "expected_executor_build"}
        if not isinstance(request, dict) or set(request) != required:
            fail()
        role, action = request.get("role"), request.get("action")
        if not closed_pair(role, action) or not isinstance(request.get("object_identity"), dict) or not isinstance(request.get("expected_executor_build"), dict):
            fail()
    else:
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
}
if tag == "stage_one_create":
    stage_value = json.loads(stage)
    request["executor_build_evidence"] = stage_value["successor_template"]["executor_build_evidence"]
    request["lima_stage_one_authorization_v1"] = stage_value
elif tag == "post_pm_action":
    request["executor_build_evidence"] = json.loads(evidence)
    request["platform_bootstrap_mapping_v1"] = mapping
    request["publisher_request"] = json.loads(publisher)
else:
    raise SystemExit("invalid mapped lifecycle authority")
print(json.dumps(request, separators=(",", ":")))
PY
}

main() {
    load_mapped_lifecycle_v1
    invoke_mac_lifecycle_control_v1
}

main
