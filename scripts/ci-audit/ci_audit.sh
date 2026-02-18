#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
ci_audit.sh (advisory)

Purpose:
  Recommend whether to SKIP or RUN multi-OS CI based on:
    - required platforms for the audit kind
    - last successful GH Actions run coverage (platforms that actually passed)
    - whether changes since that run are docs/planning-only

Usage:
  scripts/ci-audit/ci_audit.sh \
    --kind <ci-testing|feature-smoke> \
    --orch-branch <ref> \
    [--feature-dir <path>] \
    [--required-platforms <csv>] \
    [--head-sha <sha>] \
    [--baseline-sha <sha>] \
    [--ledger-path <path>] \
    [--remote <name>] \
    [--repo <owner/repo>]

Examples:
  scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch feat/my-feature
  scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch feat/my-feature --feature-dir docs/project_management/packs/active/my-feature

Notes:
  - Advisory only: does not dispatch CI.
  - Docs/planning-only changes (anything under docs/) are recommended to SKIP all CI per policy.
  - If no last-green run exists, diff baseline falls back to merge-base with origin/testing (if available).
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

KIND=""
ORCH_BRANCH=""
FEATURE_DIR=""
REQUIRED_PLATFORMS_OVERRIDE=""
HEAD_SHA=""
BASELINE_SHA_OVERRIDE=""
LEDGER_PATH=""
REMOTE="origin"
REPO=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --kind)
            KIND="${2:-}"; shift 2 ;;
        --orch-branch)
            ORCH_BRANCH="${2:-}"; shift 2 ;;
        --feature-dir)
            FEATURE_DIR="${2:-}"; shift 2 ;;
        --required-platforms)
            REQUIRED_PLATFORMS_OVERRIDE="${2:-}"; shift 2 ;;
        --head-sha)
            HEAD_SHA="${2:-}"; shift 2 ;;
        --baseline-sha)
            BASELINE_SHA_OVERRIDE="${2:-}"; shift 2 ;;
        --ledger-path)
            LEDGER_PATH="${2:-}"; shift 2 ;;
        --remote)
            REMOTE="${2:-}"; shift 2 ;;
        --repo)
            REPO="${2:-}"; shift 2 ;;
        -h|--help)
            usage; exit 0 ;;
        *)
            die "Unknown arg: $1 (use --help)" ;;
    esac
done

[[ -n "${KIND}" ]] || die "Missing --kind"
[[ -n "${ORCH_BRANCH}" ]] || die "Missing --orch-branch"

case "${KIND}" in
    ci-testing|feature-smoke) ;;
    *) die "Invalid --kind: ${KIND} (expected ci-testing or feature-smoke)" ;;
esac

require_cmd gh
require_cmd jq
require_cmd git

if [[ -z "${HEAD_SHA}" ]]; then
    HEAD_SHA="$(git rev-parse HEAD)"
fi

if [[ -n "${REPO}" ]]; then
    export GH_REPO="${REPO}"
fi

WORKFLOW_FILE=""
case "${KIND}" in
    ci-testing) WORKFLOW_FILE=".github/workflows/ci-testing.yml" ;;
    feature-smoke) WORKFLOW_FILE=".github/workflows/feature-smoke.yml" ;;
esac

git fetch -q "${REMOTE}" testing "${ORCH_BRANCH}" || true

required_platforms_csv=""
if [[ -n "${REQUIRED_PLATFORMS_OVERRIDE}" ]]; then
    required_platforms_csv="${REQUIRED_PLATFORMS_OVERRIDE}"
elif [[ "${KIND}" == "ci-testing" ]]; then
    required_platforms_csv="linux,macos,windows"
else
    if [[ -n "${FEATURE_DIR}" ]]; then
        if [[ ! -f "${FEATURE_DIR}/tasks.json" ]]; then
            die "Missing ${FEATURE_DIR}/tasks.json"
        fi
        required_platforms_csv="$(jq -r '.meta.behavior_platforms_required // [] | join(",")' "${FEATURE_DIR}/tasks.json")"
    else
        die "--feature-dir or --required-platforms is required for --kind feature-smoke"
    fi
    if [[ -z "${required_platforms_csv}" ]]; then
        required_platforms_csv="linux,macos,windows"
    fi
fi

to_set_lines() {
    tr ',' '\n' | sed '/^$/d' | sort -u
}

required_platforms_sorted="$(printf '%s' "${required_platforms_csv}" | to_set_lines)"

last_green_run_id=""
last_green_run_url=""
last_green_head_sha=""
last_passed_platforms_sorted=""
last_green_tested_sha=""

select_last_green_from_ledger() {
    local ledger_path="$1"
    local kind="$2"
    local orch_branch="$3"
    local required_csv="$4"

    [[ -f "${ledger_path}" ]] || return 1

    local required_json
    required_json="$(printf '%s' "${required_csv}" | tr ',' '\n' | sed '/^$/d' | sort -u | jq -R -s 'split("\n") | map(select(length>0))')"

    local candidate
    candidate="$(
        jq -s \
          --arg kind "${kind}" \
          --arg orch "${orch_branch}" \
          --argjson required "${required_json}" \
          '
            map(
              select(.kind==$kind)
              | select(.conclusion=="success")
              | select((.orch_branch==$orch) or (.orch_branch|not))
              | select((($required - (.passed_platforms // [])) | length) == 0)
            )
            | sort_by(.timestamp // .created_at // "")
            | reverse
            | .[0] // empty
          ' "${ledger_path}" 2>/dev/null || true
    )"

    [[ -n "${candidate}" ]] || return 1

    last_green_run_id="$(jq -r '.run_id // empty' <<<"${candidate}")"
    last_green_run_url="$(jq -r '.run_url // empty' <<<"${candidate}")"
    last_green_tested_sha="$(jq -r '.tested_sha // empty' <<<"${candidate}")"
    last_green_head_sha="$(jq -r '.head_sha // empty' <<<"${candidate}")"
    last_passed_platforms_sorted="$(jq -r '.passed_platforms // [] | .[]' <<<"${candidate}" | sort -u || true)"
    return 0
}

if [[ -n "${LEDGER_PATH}" ]]; then
    select_last_green_from_ledger "${LEDGER_PATH}" "${KIND}" "${ORCH_BRANCH}" "${required_platforms_csv}" || true
fi

run_list_json="$(gh run list --workflow "${WORKFLOW_FILE}" --branch "${ORCH_BRANCH}" --limit 50 --json databaseId,conclusion,headSha,url,createdAt || true)"
if [[ -n "${run_list_json}" ]] && jq -e 'type=="array"' >/dev/null 2>&1 <<<"${run_list_json}"; then
    if [[ -z "${last_green_run_id}" ]]; then
        last_green_run_id="$(jq -r 'map(select(.conclusion=="success")) | sort_by(.createdAt) | reverse | .[0].databaseId // empty' <<<"${run_list_json}")"
        last_green_run_url="$(jq -r 'map(select(.conclusion=="success")) | sort_by(.createdAt) | reverse | .[0].url // empty' <<<"${run_list_json}")"
        last_green_head_sha="$(jq -r 'map(select(.conclusion=="success")) | sort_by(.createdAt) | reverse | .[0].headSha // empty' <<<"${run_list_json}")"
    fi
fi

derive_passed_platforms_from_jobs() {
    local jobs_json="$1"
    local kind="$2"
    local platforms=()

    if [[ "${kind}" == "ci-testing" ]]; then
        while IFS= read -r job_name; do
            case "${job_name}" in
                "Lint & Test (ubuntu-"*")") platforms+=("linux") ;;
                "Lint & Test (macos-"*")") platforms+=("macos") ;;
                "Lint & Test (windows-"*")") platforms+=("windows") ;;
            esac
        done < <(jq -r '.[] | select(.conclusion=="success") | .name' <<<"${jobs_json}")
    else
        while IFS= read -r job_name; do
            case "${job_name}" in
                linux_*) platforms+=("linux") ;;
                macos_*) platforms+=("macos") ;;
                windows_*) platforms+=("windows") ;;
                wsl) platforms+=("wsl") ;;
            esac
        done < <(jq -r '.[] | select(.conclusion=="success") | .name' <<<"${jobs_json}")
    fi

    if [[ "${#platforms[@]}" -eq 0 ]]; then
        return 0
    fi
    printf '%s\n' "${platforms[@]}" | sort -u
}

if [[ -n "${last_green_run_id}" ]]; then
    run_jobs_json="$(gh run view "${last_green_run_id}" --json jobs | jq -c '.jobs')"
    last_passed_platforms_sorted="$(derive_passed_platforms_from_jobs "${run_jobs_json}" "${KIND}" || true)"
fi

baseline_sha=""
baseline_source=""
if [[ -n "${BASELINE_SHA_OVERRIDE}" ]]; then
    baseline_sha="${BASELINE_SHA_OVERRIDE}"
    baseline_source="baseline_sha_override"
elif [[ -n "${last_green_tested_sha}" ]]; then
    baseline_sha="${last_green_tested_sha}"
    baseline_source="ledger_tested_sha"
elif [[ -n "${last_green_head_sha}" ]]; then
    baseline_sha="${last_green_head_sha}"
    baseline_source="last_green_head_sha"
else
    if git cat-file -e "${REMOTE}/testing^{commit}" 2>/dev/null; then
        baseline_sha="$(git merge-base "${HEAD_SHA}" "${REMOTE}/testing")"
        baseline_source="merge_base_${REMOTE}_testing"
    else
        baseline_sha=""
        baseline_source="none"
    fi
fi

diff_files_count="0"
diff_insertions="0"
diff_deletions="0"
diff_class="unknown"

if [[ -n "${baseline_sha}" ]]; then
    if git cat-file -e "${baseline_sha}^{commit}" 2>/dev/null; then
        mapfile -t diff_files < <(git diff --name-only "${baseline_sha}..${HEAD_SHA}" || true)
        diff_files_count="${#diff_files[@]}"
        if [[ "${diff_files_count}" -eq 0 ]]; then
            diff_class="no_changes"
        else
            if git diff --numstat "${baseline_sha}..${HEAD_SHA}" >/dev/null 2>&1; then
                read -r diff_insertions diff_deletions < <(
                    git diff --numstat "${baseline_sha}..${HEAD_SHA}" \
                        | awk '{ins+=$1; del+=$2} END {print ins+0, del+0}'
                )
            fi

            docs_only=1
            for f in "${diff_files[@]}"; do
                if [[ "${f}" != docs/* ]]; then
                    docs_only=0
                    break
                fi
            done

            if [[ "${docs_only}" -eq 1 ]]; then
                diff_class="docs_only"
            else
                diff_class="code_affecting"
            fi
        fi
    fi
fi

missing_platforms_sorted=""
if [[ -n "${last_passed_platforms_sorted}" ]]; then
    missing_platforms_sorted="$(comm -23 <(printf '%s\n' "${required_platforms_sorted}") <(printf '%s\n' "${last_passed_platforms_sorted}") || true)"
else
    missing_platforms_sorted="${required_platforms_sorted}"
fi

recommend="run"
reason=""

if [[ "${diff_class}" == "docs_only" ]]; then
    recommend="skip"
    reason="docs_only_changes"
elif [[ "${diff_class}" == "no_changes" ]] && [[ -z "${missing_platforms_sorted}" ]]; then
    recommend="skip"
    reason="already_green_full_coverage_and_no_changes"
elif [[ -n "${missing_platforms_sorted}" ]] && [[ -n "${last_green_run_id}" ]]; then
    recommend="run"
    reason="last_green_missing_required_platform_coverage"
elif [[ -z "${last_green_run_id}" ]]; then
    recommend="run"
    reason="no_last_green_run_found"
else
    recommend="run"
    reason="changes_since_last_green"
fi

echo "CI Audit (advisory)"
echo "  kind: ${KIND}"
echo "  workflow: ${WORKFLOW_FILE}"
echo "  orch_branch: ${ORCH_BRANCH}"
echo "  head_sha: ${HEAD_SHA}"
if [[ -n "${last_green_run_id}" ]]; then
    echo "  last_green: ${last_green_run_id} (${last_green_run_url})"
    echo "  last_green_head_sha: ${last_green_head_sha}"
else
    echo "  last_green: (none found)"
fi
echo "  required_platforms: ${required_platforms_csv}"
echo "  last_passed_platforms: $(printf '%s' "${last_passed_platforms_sorted}" | paste -sd, - 2>/dev/null || true)"
echo "  baseline_source: ${baseline_source}"
echo "  baseline_sha: ${baseline_sha}"
echo "  diff_class: ${diff_class} (files=${diff_files_count}, +${diff_insertions}/-${diff_deletions})"
if [[ -n "${missing_platforms_sorted}" ]]; then
    echo "  missing_platforms: $(printf '%s' "${missing_platforms_sorted}" | paste -sd, - 2>/dev/null || true)"
else
    echo "  missing_platforms: (none)"
fi
echo "  recommend: ${recommend} (${reason})"

echo
echo "RECOMMEND=${recommend}"
echo "REASON=${reason}"
echo "REQUIRED_PLATFORMS=${required_platforms_csv}"
echo "LAST_GREEN_RUN_ID=${last_green_run_id}"
echo "LAST_GREEN_RUN_URL=${last_green_run_url}"
echo "LAST_GREEN_TESTED_SHA=${last_green_tested_sha}"
echo "LAST_GREEN_HEAD_SHA=${last_green_head_sha}"
echo "LAST_PASSED_PLATFORMS=$(printf '%s' "${last_passed_platforms_sorted}" | paste -sd, - 2>/dev/null || true)"
echo "BASELINE_SOURCE=${baseline_source}"
echo "BASELINE_SHA=${baseline_sha}"
echo "DIFF_CLASS=${diff_class}"
echo "DIFF_FILES_COUNT=${diff_files_count}"
echo "DIFF_LOC=+${diff_insertions},-${diff_deletions}"
