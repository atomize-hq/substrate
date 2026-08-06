#!/usr/bin/env bash
set -euo pipefail

INSTALL_PREFIX=""
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PLATFORM_BOOTSTRAP_MAPPING_V1=""
EXECUTOR_BUILD_EVIDENCE_V1=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install-prefix) INSTALL_PREFIX="${2:?missing --install-prefix value}"; shift 2 ;;
        --install-bootstrap-context-v1) INSTALL_BOOTSTRAP_CONTEXT_V1="${2:?missing --install-bootstrap-context-v1 value}"; shift 2 ;;
        --platform-bootstrap-mapping-v1) PLATFORM_BOOTSTRAP_MAPPING_V1="${2:?missing --platform-bootstrap-mapping-v1 value}"; shift 2 ;;
        --executor-build-evidence-v1) EXECUTOR_BUILD_EVIDENCE_V1="${2:?missing --executor-build-evidence-v1 value}"; shift 2 ;;
        *) printf 'unexpected argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

resolve_lima_stop_authority_v1() {
    [[ -n "${INSTALL_PREFIX}" && -n "${INSTALL_BOOTSTRAP_CONTEXT_V1}" && -n "${PLATFORM_BOOTSTRAP_MAPPING_V1}" && -n "${EXECUTOR_BUILD_EVIDENCE_V1}" ]] || {
        printf '%s\n' 'mapped Lima stop requires prefix, carrier, mapping, and ExecutorBuildEvidenceV1' >&2; return 1;
    }
}

invoke_mapped_lima_stop_v1() {
    "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lima-lifecycle.sh" stop \
        --install-prefix "${INSTALL_PREFIX}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}"
}

resolve_lima_stop_authority_v1
invoke_mapped_lima_stop_v1
