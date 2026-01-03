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
    [--run-integ-checks] \
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
RUN_INTEG_CHECKS=0
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
        --run-integ-checks)
            RUN_INTEG_CHECKS=1
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

if ! gh api user >/dev/null 2>&1; then
    echo "GitHub CLI auth is not usable (token invalid or missing). Fix with: gh auth login -h github.com (or set GH_TOKEN for non-interactive runs)." >&2
    exit 3
fi

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
run_wsl_flag="false"
run_integ_checks_flag="false"
if [[ "${RUN_WSL}" -eq 1 ]]; then
    run_wsl_flag="true"
fi
if [[ "${RUN_INTEG_CHECKS}" -eq 1 ]]; then
    run_integ_checks_flag="true"
fi
gh workflow run "${WORKFLOW}" --ref "${WORKFLOW_REF}" \
    -f feature_dir="${FEATURE_DIR}" \
    -f checkout_ref="${temp_branch}" \
    -f runner_kind="${RUNNER_KIND}" \
    -f platform="${PLATFORM}" \
    -f run_wsl="${run_wsl_flag}" \
    -f run_integ_checks="${run_integ_checks_flag}"

echo "Waiting for run to start..."
sleep 5

run_id="$(gh run list --workflow "${WORKFLOW}" --event workflow_dispatch --branch "${WORKFLOW_REF}" --limit 20 --json databaseId,createdAt -q "map(select(.createdAt >= \"${dispatch_started}\")) | .[0].databaseId")"
if [[ -z "${run_id}" ]]; then
    echo "Could not find a matching workflow run for ${temp_branch}" >&2
    exit 4
fi

echo "Run: ${run_id}"
echo "RUN_ID=${run_id}"
run_url="$(gh run view "${run_id}" --json url -q '.url' 2>/dev/null || true)"
if [[ -n "${run_url}" ]]; then
    echo "RUN_URL=${run_url}"
fi
gh run watch "${run_id}"
conclusion="$(gh run view "${run_id}" --json conclusion -q '.conclusion')"
echo "Conclusion: ${conclusion}"

platform_summary="$(gh run view "${run_id}" --json jobs 2>/dev/null || true)"
if [[ -n "${platform_summary}" ]]; then
    python3 - "${platform_summary}" <<'PY' || true
import json
import sys

raw = sys.argv[1]
try:
    data = json.loads(raw)
except Exception:
    raise SystemExit(0)

jobs = data.get("jobs") or []
failed = set()
passed = set()

def job_platform(name):
    if name.startswith("linux_"):
        return "linux"
    if name.startswith("macos_"):
        return "macos"
    if name.startswith("windows_"):
        return "windows"
    if name == "wsl":
        return "wsl"
    return None

for j in jobs:
    if not isinstance(j, dict):
        continue
    name = j.get("name") or ""
    concl = j.get("conclusion")
    if concl in (None, "skipped"):
        continue
    p = job_platform(str(name))
    if not p:
        continue
    if concl == "success":
        passed.add(p)
    else:
        failed.add(p)

def csv(xs):
    return ",".join(sorted(xs))

print(f"SMOKE_PASSED_PLATFORMS={csv(passed)}")
print(f"SMOKE_FAILED_PLATFORMS={csv(failed)}")
PY
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
