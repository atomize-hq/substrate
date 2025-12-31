#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/ci/dispatch_feature_smoke.sh \
    --feature-dir docs/project_management/next/<feature> \
    [--runner-kind github-hosted|self-hosted] \
    --platform linux|macos|windows|wsl|all \
    [--run-wsl] \
    [--workflow .github/workflows/feature-smoke.yml] \
    [--workflow-ref <ref>] \
    [--remote origin] \
    [--cleanup]

What it does:
  - Creates a throwaway remote branch at HEAD
  - Dispatches the workflow against the workflow ref (default: feat/policy_and_config), checking out the throwaway branch
  - Optionally waits and deletes the throwaway branch

Requirements:
  - `gh` CLI installed and authenticated
  - Push access to the configured remote
USAGE
}

FEATURE_DIR=""
PLATFORM=""
RUNNER_KIND="self-hosted"
RUN_WSL=0
WORKFLOW=".github/workflows/feature-smoke.yml"
WORKFLOW_REF="feat/policy_and_config"
REMOTE="origin"
CLEANUP=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --runner-kind)
            RUNNER_KIND="${2:-}"
            shift 2
            ;;
        --platform)
            PLATFORM="${2:-}"
            shift 2
            ;;
        --run-wsl)
            RUN_WSL=1
            shift 1
            ;;
        --workflow)
            WORKFLOW="${2:-}"
            shift 2
            ;;
        --workflow-ref)
            WORKFLOW_REF="${2:-}"
            shift 2
            ;;
        --remote)
            REMOTE="${2:-}"
            shift 2
            ;;
        --cleanup)
            CLEANUP=1
            shift 1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown arg: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -z "${FEATURE_DIR}" || -z "${PLATFORM}" ]]; then
    echo "Missing required args" >&2
    usage >&2
    exit 2
fi

case "${PLATFORM}" in
    linux|macos|windows|wsl|all) ;;
    *)
        echo "Invalid --platform: ${PLATFORM}" >&2
        usage >&2
        exit 2
        ;;
esac

case "${RUNNER_KIND}" in
    github-hosted|self-hosted) ;;
    *)
        echo "Invalid --runner-kind: ${RUNNER_KIND}" >&2
        usage >&2
        exit 2
        ;;
esac

if ! command -v gh >/dev/null 2>&1; then
    echo "Missing dependency: gh (GitHub CLI)" >&2
    exit 3
fi

gh auth status >/dev/null

if [[ -z "${WORKFLOW_REF}" ]]; then
    echo "Missing --workflow-ref" >&2
    usage >&2
    exit 2
fi

ts="$(date -u +%Y%m%dT%H%M%SZ)"
safe_feature="$(basename "${FEATURE_DIR}")"
temp_branch="tmp/feature-smoke/${safe_feature}/${PLATFORM}/${ts}"

head_sha="$(git rev-parse HEAD)"
echo "HEAD: ${head_sha}"
echo "Temp branch: ${temp_branch}"

git branch -f "${temp_branch}" "${head_sha}"
git push -u "${REMOTE}" "${temp_branch}:${temp_branch}"

echo "Dispatching workflow: ${WORKFLOW}"
echo "Workflow ref: ${WORKFLOW_REF}"
dispatch_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [[ "${RUN_WSL}" -eq 1 ]]; then
    gh workflow run "${WORKFLOW}" --ref "${WORKFLOW_REF}" -f feature_dir="${FEATURE_DIR}" -f checkout_ref="${temp_branch}" -f runner_kind="${RUNNER_KIND}" -f platform="${PLATFORM}" -f run_wsl=true
else
    gh workflow run "${WORKFLOW}" --ref "${WORKFLOW_REF}" -f feature_dir="${FEATURE_DIR}" -f checkout_ref="${temp_branch}" -f runner_kind="${RUNNER_KIND}" -f platform="${PLATFORM}" -f run_wsl=false
fi

echo "Waiting for run to start..."
sleep 5

find_run_id_for_checkout_ref() {
    local checkout_ref="$1"
    local started_after="$2"

    local candidate_ids
    candidate_ids="$(
        gh run list \
            --workflow "${WORKFLOW}" \
            --event workflow_dispatch \
            --branch "${WORKFLOW_REF}" \
            --limit 50 \
            --json databaseId,createdAt \
            -q "map(select(.createdAt >= \"${started_after}\")) | .[].databaseId"
    )"

    if [[ -z "${candidate_ids}" ]]; then
        return 1
    fi

    local candidate_id
    while IFS= read -r candidate_id; do
        [[ -z "${candidate_id}" ]] && continue

        local job_ids
        job_ids="$(gh run view "${candidate_id}" --json jobs -q '.jobs[].databaseId' 2>/dev/null || true)"
        if [[ -z "${job_ids}" ]]; then
            continue
        fi

        local job_id
        while IFS= read -r job_id; do
            [[ -z "${job_id}" ]] && continue
            if gh run view "${candidate_id}" --log --job "${job_id}" 2>/dev/null | grep -Fq "${checkout_ref}"; then
                echo "${candidate_id}"
                return 0
            fi
        done <<<"${job_ids}"
    done <<<"${candidate_ids}"

    return 1
}

run_id=""
for _attempt in $(seq 1 30); do
    if run_id="$(find_run_id_for_checkout_ref "${temp_branch}" "${dispatch_started}")"; then
        break
    fi
    sleep 2
done

if [[ -z "${run_id}" ]]; then
    run_id="$(
        gh run list \
            --workflow "${WORKFLOW}" \
            --event workflow_dispatch \
            --branch "${WORKFLOW_REF}" \
            --limit 50 \
            --json databaseId,createdAt \
            -q "map(select(.createdAt >= \"${dispatch_started}\")) | sort_by(.createdAt) | .[0].databaseId"
    )"
fi

if [[ -z "${run_id}" ]]; then
    echo "Could not find a matching workflow run for ${temp_branch}" >&2
    exit 4
fi

echo "Run: ${run_id}"
gh run watch "${run_id}"
conclusion="$(gh run view "${run_id}" --json conclusion -q '.conclusion')"
echo "Conclusion: ${conclusion}"

validate_job_id="$(gh run view "${run_id}" --json jobs -q '.jobs[] | select(.name=="validate_inputs") | .databaseId' || true)"
if [[ -n "${validate_job_id}" ]]; then
    resolved_platform="$(
        gh run view "${run_id}" --log --job "${validate_job_id}" 2>/dev/null \
            | sed -E 's/\x1b\\[[0-9;]*m//g' \
            | grep -oE 'platform="[^"]+"' \
            | head -n 1 \
            | cut -d '"' -f 2 \
            || true
    )"
    if [[ -n "${resolved_platform}" && "${resolved_platform}" != "${PLATFORM}" ]]; then
        echo "ERROR: resolved platform (${resolved_platform}) != requested platform (${PLATFORM}) for ${run_id}" >&2
        exit 5
    fi
fi

if [[ "${CLEANUP}" -eq 1 ]]; then
    echo "Cleaning up remote branch: ${temp_branch}"
    git push "${REMOTE}" ":${temp_branch}"
    git branch -D "${temp_branch}" >/dev/null 2>&1 || true
fi

if [[ "${conclusion}" != "success" ]]; then
    exit 1
fi

echo "OK: feature smoke passed"
