#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
ci_audit_record.sh

Purpose:
  Append an evidence entry to a local JSONL ledger for a completed GH Actions run.
  This improves ci-audit correctness when dispatch uses throwaway branches, because
  we explicitly record TESTED_SHA and the passed platform coverage.

Usage:
  scripts/ci-audit/ci_audit_record.sh \
    --ledger-path <path> \
    --kind <ci-testing|feature-smoke> \
    --orch-branch <ref> \
    --run-id <id> \
    --tested-sha <sha> \
    [--feature-dir <path>] \
    [--required-platforms <csv>] \
    [--mode <string>] \
    [--repo <owner/repo>] \
    [--dry-run]

Notes:
  - Does not dispatch CI. It only records evidence after the fact.
  - Intended to be run by operators/wrappers outside the triad/CI scripts.
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

LEDGER_PATH=""
KIND=""
ORCH_BRANCH=""
RUN_ID=""
TESTED_SHA=""
FEATURE_DIR=""
REQUIRED_PLATFORMS_OVERRIDE=""
MODE=""
REPO=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ledger-path) LEDGER_PATH="${2:-}"; shift 2 ;;
        --kind) KIND="${2:-}"; shift 2 ;;
        --orch-branch) ORCH_BRANCH="${2:-}"; shift 2 ;;
        --run-id) RUN_ID="${2:-}"; shift 2 ;;
        --tested-sha) TESTED_SHA="${2:-}"; shift 2 ;;
        --feature-dir) FEATURE_DIR="${2:-}"; shift 2 ;;
        --required-platforms) REQUIRED_PLATFORMS_OVERRIDE="${2:-}"; shift 2 ;;
        --mode) MODE="${2:-}"; shift 2 ;;
        --repo) REPO="${2:-}"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift 1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "Unknown arg: $1 (use --help)" ;;
    esac
done

[[ -n "${LEDGER_PATH}" ]] || die "Missing --ledger-path"
[[ -n "${KIND}" ]] || die "Missing --kind"
[[ -n "${ORCH_BRANCH}" ]] || die "Missing --orch-branch"
[[ -n "${RUN_ID}" ]] || die "Missing --run-id"
[[ -n "${TESTED_SHA}" ]] || die "Missing --tested-sha"

case "${KIND}" in
    ci-testing|feature-smoke) ;;
    *) die "Invalid --kind: ${KIND} (expected ci-testing or feature-smoke)" ;;
esac

require_cmd gh
require_cmd jq
require_cmd git

if [[ -n "${REPO}" ]]; then
    export GH_REPO="${REPO}"
fi

required_platforms_csv=""
if [[ -n "${REQUIRED_PLATFORMS_OVERRIDE}" ]]; then
    required_platforms_csv="${REQUIRED_PLATFORMS_OVERRIDE}"
elif [[ "${KIND}" == "ci-testing" ]]; then
    required_platforms_csv="linux,macos,windows"
else
    if [[ -n "${FEATURE_DIR}" ]]; then
        [[ -f "${FEATURE_DIR}/tasks.json" ]] || die "Missing ${FEATURE_DIR}/tasks.json"
        required_platforms_csv="$(jq -r '.meta.behavior_platforms_required // [] | join(",")' "${FEATURE_DIR}/tasks.json")"
        if [[ -z "${required_platforms_csv}" ]]; then
            required_platforms_csv="linux,macos,windows"
        fi
    else
        required_platforms_csv="linux,macos,windows"
    fi
fi

to_set_json() {
    tr ',' '\n' | sed '/^$/d' | sort -u | jq -R -s 'split("\n") | map(select(length>0))'
}

derive_passed_platforms() {
    local jobs_json="$1"
    local kind="$2"

    if [[ "${kind}" == "ci-testing" ]]; then
        jq -r '
          .[]
          | select(.conclusion=="success")
          | .name
        ' <<<"${jobs_json}" | while IFS= read -r name; do
            case "${name}" in
                "Lint & Test (ubuntu-"*")") echo linux ;;
                "Lint & Test (macos-"*")") echo macos ;;
                "Lint & Test (windows-"*")") echo windows ;;
            esac
        done | sort -u
    else
        jq -r '
          .[]
          | select(.conclusion=="success")
          | .name
        ' <<<"${jobs_json}" | while IFS= read -r name; do
            case "${name}" in
                linux_*) echo linux ;;
                macos_*) echo macos ;;
                windows_*) echo windows ;;
                wsl) echo wsl ;;
            esac
        done | sort -u
    fi
}

run_json="$(gh run view "${RUN_ID}" --json url,conclusion,workflowName,event,createdAt,headBranch,headSha,jobs)"
run_url="$(jq -r '.url' <<<"${run_json}")"
conclusion="$(jq -r '.conclusion' <<<"${run_json}")"
workflow_name="$(jq -r '.workflowName' <<<"${run_json}")"
event="$(jq -r '.event' <<<"${run_json}")"
created_at="$(jq -r '.createdAt' <<<"${run_json}")"
head_branch="$(jq -r '.headBranch' <<<"${run_json}")"
head_sha="$(jq -r '.headSha' <<<"${run_json}")"
jobs_json="$(jq -c '.jobs' <<<"${run_json}")"

timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
required_platforms_json="$(printf '%s' "${required_platforms_csv}" | to_set_json)"
passed_platforms_json="$(derive_passed_platforms "${jobs_json}" "${KIND}" | jq -R -s 'split("\n") | map(select(length>0))')"

entry="$(jq -n \
    --arg timestamp "${timestamp}" \
    --arg orch_branch "${ORCH_BRANCH}" \
    --arg kind "${KIND}" \
    --arg mode "${MODE}" \
    --arg tested_sha "${TESTED_SHA}" \
    --arg run_id "${RUN_ID}" \
    --arg run_url "${run_url}" \
    --arg conclusion "${conclusion}" \
    --arg workflow_name "${workflow_name}" \
    --arg event "${event}" \
    --arg created_at "${created_at}" \
    --arg head_branch "${head_branch}" \
    --arg head_sha "${head_sha}" \
    --argjson required_platforms "${required_platforms_json}" \
    --argjson passed_platforms "${passed_platforms_json}" \
    '{
      timestamp: $timestamp,
      orch_branch: $orch_branch,
      kind: $kind,
      mode: ($mode | select(length>0) // null),
      tested_sha: $tested_sha,
      required_platforms: $required_platforms,
      passed_platforms: $passed_platforms,
      run_id: $run_id,
      run_url: $run_url,
      conclusion: $conclusion,
      workflow_name: $workflow_name,
      event: $event,
      created_at: $created_at,
      head_branch: $head_branch,
      head_sha: $head_sha
    }')"

if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "${entry}"
    exit 0
fi

mkdir -p "$(dirname "${LEDGER_PATH}")"
printf '%s\n' "${entry}" >>"${LEDGER_PATH}"

echo "RECORDED=1"
echo "LEDGER_PATH=${LEDGER_PATH}"
echo "RUN_ID=${RUN_ID}"
echo "RUN_URL=${run_url}"
echo "CONCLUSION=${conclusion}"
echo "TESTED_SHA=${TESTED_SHA}"
echo "REQUIRED_PLATFORMS=${required_platforms_csv}"
echo "PASSED_PLATFORMS=$(jq -r '.passed_platforms | join(",")' <<<"${entry}")"

