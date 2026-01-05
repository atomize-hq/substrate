#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/ci/dispatch_ci_testing.sh \
    [--checkout-ref <git-ref>] \
    [--workflow .github/workflows/ci-testing.yml] \
    [--workflow-ref <ref>] \
    [--remote origin] \
    [--cleanup]

What it does:
  - Creates a throwaway remote branch at the target commit (default: HEAD)
  - Dispatches the CI Testing workflow (workflow_dispatch) from --workflow-ref, checking out the throwaway branch
  - Waits for completion, prints run metadata, and optionally deletes the throwaway branch

Notes:
  - This is meant to catch issues that Feature Smoke won't (fmt/clippy -D warnings/full workspace tests).
  - Requires the workflow to support workflow_dispatch input: checkout_ref.

Requirements:
  - `gh` CLI installed and authenticated
  - Push access to the configured remote

Stdout contract (machine-parseable):
  HEAD=<sha>
  TEMP_BRANCH=<branch>
  RUN_ID=<id>
  RUN_URL=<url or empty>
  CONCLUSION=<conclusion>
  CI_PASSED_OSES=<csv or empty>
  CI_FAILED_OSES=<csv or empty>
  CI_FAILED_JOBS=<csv or empty>
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Missing dependency: $1"
    fi
}

WORKFLOW=".github/workflows/ci-testing.yml"
WORKFLOW_REF="testing"
REMOTE="origin"
CLEANUP=0
CHECKOUT_REF=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --checkout-ref)
            CHECKOUT_REF="${2:-}"
            shift 2
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
            die "Unknown arg: $1"
            ;;
    esac
done

require_cmd git
require_cmd gh
require_cmd python3

if ! gh api user >/dev/null 2>&1; then
    die "GitHub CLI auth is not usable (token invalid or missing). Fix with: gh auth login -h github.com (or set GH_TOKEN)."
fi

if [[ -z "${WORKFLOW_REF}" ]]; then
    die "Missing --workflow-ref"
fi

if [[ -z "${CHECKOUT_REF}" ]]; then
    CHECKOUT_REF="$(git rev-parse HEAD)"
fi

head_sha="$(git rev-parse "${CHECKOUT_REF}")"
ts="$(date -u +%Y%m%dT%H%M%SZ)"
temp_branch="tmp/ci-testing/${ts}"

echo "HEAD: ${head_sha}" >&2
echo "Temp branch: ${temp_branch}" >&2

git branch -f "${temp_branch}" "${head_sha}"
git push -u "${REMOTE}" "${temp_branch}:${temp_branch}" >&2

echo "Dispatching workflow: ${WORKFLOW}" >&2
echo "Workflow ref: ${WORKFLOW_REF}" >&2
dispatch_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

gh workflow run "${WORKFLOW}" --ref "${WORKFLOW_REF}" -f checkout_ref="${temp_branch}" >&2

echo "Waiting for run to start..." >&2
sleep 5

run_id="$(gh run list --workflow "${WORKFLOW}" --event workflow_dispatch --branch "${WORKFLOW_REF}" --limit 30 --json databaseId,createdAt -q "map(select(.createdAt >= \"${dispatch_started}\")) | .[0].databaseId")"
if [[ -z "${run_id}" ]]; then
    die "Could not find a matching workflow run for ${temp_branch}"
fi

run_url="$(gh run view "${run_id}" --json url -q '.url' 2>/dev/null || true)"

watch_interval_secs="${CI_TESTING_WATCH_INTERVAL_SECS:-15}"
watch_timeout_secs="${CI_TESTING_WATCH_TIMEOUT_SECS:-21600}" # 6h
started_watch_at="$(date +%s)"

echo "Watching run status (interval=${watch_interval_secs}s timeout=${watch_timeout_secs}s)..." >&2
while true; do
    status="$(gh run view "${run_id}" --json status -q '.status' 2>/dev/null || true)"
    if [[ -z "${status}" ]]; then
        # Transient gh/API failures are common; keep polling.
        sleep "${watch_interval_secs}"
        continue
    fi
    if [[ "${status}" == "completed" ]]; then
        break
    fi

    now="$(date +%s)"
    elapsed="$((now - started_watch_at))"
    if [[ "${elapsed}" -ge "${watch_timeout_secs}" ]]; then
        die "Timed out waiting for CI Testing run ${run_id} to complete after ${elapsed}s"
    fi

    sleep "${watch_interval_secs}"
done
conclusion="$(gh run view "${run_id}" --json conclusion -q '.conclusion' 2>/dev/null || true)"

jobs_json="$(gh run view "${run_id}" --json jobs 2>/dev/null || true)"
passed_oses=""
failed_oses=""
failed_jobs=""
if [[ -n "${jobs_json}" ]]; then
    parsed="$(python3 - "${jobs_json}" <<'PY' || true
import json
import sys

raw = sys.argv[1]
data = json.loads(raw)
jobs = data.get("jobs") or []

passed_oses = set()
failed_oses = set()
failed_jobs = set()

def os_from_job(name: str):
    prefix = "Lint & Test ("
    if name.startswith(prefix) and name.endswith(")"):
        return name[len(prefix):-1].strip()
    return None

for j in jobs:
    if not isinstance(j, dict):
        continue
    name = str(j.get("name") or "")
    concl = j.get("conclusion")
    if concl in (None, "skipped"):
        continue
    os_name = os_from_job(name)
    if concl == "success":
        if os_name:
            passed_oses.add(os_name)
    else:
        failed_jobs.add(name)
        if os_name:
            failed_oses.add(os_name)

def csv(xs):
    return ",".join(sorted(xs))

print(f"CI_PASSED_OSES={csv(passed_oses)}")
print(f"CI_FAILED_OSES={csv(failed_oses)}")
print(f"CI_FAILED_JOBS={csv(failed_jobs)}")
PY
)"
    passed_oses="$(printf '%s\n' "${parsed}" | awk -F= '$1=="CI_PASSED_OSES"{sub($1"=","",$0); print $0}')"
    failed_oses="$(printf '%s\n' "${parsed}" | awk -F= '$1=="CI_FAILED_OSES"{sub($1"=","",$0); print $0}')"
    failed_jobs="$(printf '%s\n' "${parsed}" | awk -F= '$1=="CI_FAILED_JOBS"{sub($1"=","",$0); print $0}')"
fi

printf 'HEAD=%s\n' "${head_sha}"
printf 'TEMP_BRANCH=%s\n' "${temp_branch}"
printf 'RUN_ID=%s\n' "${run_id}"
printf 'RUN_URL=%s\n' "${run_url}"
printf 'CONCLUSION=%s\n' "${conclusion}"
printf 'CI_PASSED_OSES=%s\n' "${passed_oses}"
printf 'CI_FAILED_OSES=%s\n' "${failed_oses}"
printf 'CI_FAILED_JOBS=%s\n' "${failed_jobs}"

if [[ "${CLEANUP}" -eq 1 ]]; then
    echo "Cleaning up remote branch: ${temp_branch}" >&2
    git push "${REMOTE}" ":${temp_branch}" >&2 || true
    git branch -D "${temp_branch}" >/dev/null 2>&1 || true
fi

if [[ "${conclusion}" != "success" ]]; then
    exit 1
fi
