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

Stdout contract (machine-parseable):
  HEAD=<sha>
  TEMP_BRANCH=<branch>
  RUN_ID=<id>
  RUN_URL=<url or empty>
  CONCLUSION=<conclusion>
  SMOKE_PASSED_PLATFORMS=<csv or empty>
  SMOKE_FAILED_PLATFORMS=<csv or empty>
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

run_with_timeout() {
    local timeout_secs="$1"
    shift

    if command -v timeout >/dev/null 2>&1; then
        timeout -k 10s "${timeout_secs}s" "$@"
        return $?
    fi
    if command -v gtimeout >/dev/null 2>&1; then
        gtimeout -k 10s "${timeout_secs}s" "$@"
        return $?
    fi

    python3 - "$timeout_secs" "$@" <<'PY'
import os
import signal
import subprocess
import sys

timeout_secs = float(sys.argv[1])
cmd = sys.argv[2:]

proc = subprocess.Popen(cmd, start_new_session=True)
try:
    raise SystemExit(proc.wait(timeout=timeout_secs))
except subprocess.TimeoutExpired:
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        raise SystemExit(124)
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    raise SystemExit(124)
PY
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
    usage >&2
    die "Missing required args: --feature-dir and --platform"
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

require_cmd git
require_cmd gh
require_cmd python3

if ! gh api user >/dev/null 2>&1; then
    die "GitHub CLI auth is not usable (token invalid or missing). Fix with: gh auth login -h github.com (or set GH_TOKEN for non-interactive runs)."
fi

if [[ -z "${WORKFLOW_REF}" ]]; then
    usage >&2
    die "Missing --workflow-ref"
fi

GIT_PUSH_TIMEOUT_SECS="${FEATURE_SMOKE_GIT_PUSH_TIMEOUT_SECS:-300}"
GH_TIMEOUT_SECS="${FEATURE_SMOKE_GH_TIMEOUT_SECS:-120}"
WATCH_INTERVAL_SECS="${FEATURE_SMOKE_WATCH_INTERVAL_SECS:-15}"
WATCH_TIMEOUT_SECS="${FEATURE_SMOKE_WATCH_TIMEOUT_SECS:-21600}" # 6h
WATCH_MAX_CONSECUTIVE_ERRORS="${FEATURE_SMOKE_WATCH_MAX_CONSECUTIVE_ERRORS:-20}"
RUN_LOOKUP_TIMEOUT_SECS="${FEATURE_SMOKE_RUN_LOOKUP_TIMEOUT_SECS:-120}"

ts="$(date -u +%Y%m%dT%H%M%SZ)"
safe_feature="$(basename "${FEATURE_DIR}")"
temp_branch="tmp/feature-smoke/${safe_feature}/${PLATFORM}/${ts}"

head_sha="$(git rev-parse HEAD)"
echo "HEAD: ${head_sha}" >&2
echo "Temp branch: ${temp_branch}" >&2

git branch -f "${temp_branch}" "${head_sha}"
if ! run_with_timeout "${GIT_PUSH_TIMEOUT_SECS}" git push -u "${REMOTE}" "${temp_branch}:${temp_branch}" >&2; then
    die "git push timed out or failed (branch=${temp_branch})"
fi

echo "Dispatching workflow: ${WORKFLOW}" >&2
echo "Workflow ref: ${WORKFLOW_REF}" >&2
dispatch_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
run_wsl_flag="false"
run_integ_checks_flag="false"
if [[ "${RUN_WSL}" -eq 1 ]]; then
    run_wsl_flag="true"
fi
if [[ "${RUN_INTEG_CHECKS}" -eq 1 ]]; then
    run_integ_checks_flag="true"
fi
if ! run_with_timeout "${GH_TIMEOUT_SECS}" gh workflow run "${WORKFLOW}" --ref "${WORKFLOW_REF}" \
    -f feature_dir="${FEATURE_DIR}" \
    -f checkout_ref="${temp_branch}" \
    -f runner_kind="${RUNNER_KIND}" \
    -f platform="${PLATFORM}" \
    -f run_wsl="${run_wsl_flag}" \
    -f run_integ_checks="${run_integ_checks_flag}"; then
    die "failed to dispatch workflow via gh (workflow=${WORKFLOW} ref=${WORKFLOW_REF})"
fi

echo "Waiting for run to start..." >&2
started_lookup_at="$(date +%s)"
run_id=""
while [[ -z "${run_id}" ]]; do
    run_id="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run list --workflow "${WORKFLOW}" --event workflow_dispatch --branch "${WORKFLOW_REF}" --limit 20 --json databaseId,createdAt -q "map(select(.createdAt >= \"${dispatch_started}\")) | .[0].databaseId" 2>/dev/null || true)"
    if [[ -n "${run_id}" ]]; then
        break
    fi
    now="$(date +%s)"
    if [[ $((now - started_lookup_at)) -ge "${RUN_LOOKUP_TIMEOUT_SECS}" ]]; then
        die "Could not find a matching workflow run for ${temp_branch} after ${RUN_LOOKUP_TIMEOUT_SECS}s"
    fi
    sleep 5
done
if [[ -z "${run_id}" ]]; then
    die "Could not find a matching workflow run for ${temp_branch}"
fi

echo "Run: ${run_id}" >&2
run_url="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json url -q '.url' 2>/dev/null || true)"
started_watch_at="$(date +%s)"
next_heartbeat_at="$((started_watch_at + 60))"
consecutive_errors=0

echo "Watching run status (interval=${WATCH_INTERVAL_SECS}s timeout=${WATCH_TIMEOUT_SECS}s)..." >&2
while true; do
    now="$(date +%s)"
    elapsed="$((now - started_watch_at))"
    if [[ "${elapsed}" -ge "${WATCH_TIMEOUT_SECS}" ]]; then
        die "Timed out waiting for smoke run ${run_id} to complete after ${elapsed}s"
    fi

    status="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json status -q '.status' 2>/dev/null || true)"
    if [[ -z "${status}" ]]; then
        consecutive_errors="$((consecutive_errors + 1))"
        if [[ "${consecutive_errors}" -ge "${WATCH_MAX_CONSECUTIVE_ERRORS}" ]]; then
            die "Repeated failures querying GitHub run status (run=${run_id})"
        fi
        status="unknown"
    else
        consecutive_errors=0
    fi
    if [[ "${status}" == "completed" ]]; then
        break
    fi

    if [[ "${now}" -ge "${next_heartbeat_at}" ]]; then
        echo "  status=${status} elapsed_s=${elapsed} run=${run_id}" >&2
        next_heartbeat_at="$((now + 60))"
    fi

    sleep "${WATCH_INTERVAL_SECS}"
done
conclusion="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json conclusion -q '.conclusion' 2>/dev/null || true)"

platform_summary="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json jobs 2>/dev/null || true)"
passed_csv=""
failed_csv=""
if [[ -n "${platform_summary}" ]]; then
    parsed="$(python3 - "${platform_summary}" <<'PY' || true
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
)"
    passed_csv="$(printf '%s\n' "${parsed}" | awk -F= '$1=="SMOKE_PASSED_PLATFORMS"{sub($1"=","",$0); print $0}')"
    failed_csv="$(printf '%s\n' "${parsed}" | awk -F= '$1=="SMOKE_FAILED_PLATFORMS"{sub($1"=","",$0); print $0}')"
fi

printf 'HEAD=%s\n' "${head_sha}"
printf 'TEMP_BRANCH=%s\n' "${temp_branch}"
printf 'RUN_ID=%s\n' "${run_id}"
printf 'RUN_URL=%s\n' "${run_url}"
printf 'CONCLUSION=%s\n' "${conclusion}"
printf 'SMOKE_PASSED_PLATFORMS=%s\n' "${passed_csv}"
printf 'SMOKE_FAILED_PLATFORMS=%s\n' "${failed_csv}"

if [[ "${CLEANUP}" -eq 1 ]]; then
    echo "Cleaning up remote branch: ${temp_branch}" >&2
    if ! run_with_timeout "${GIT_PUSH_TIMEOUT_SECS}" git push "${REMOTE}" ":${temp_branch}" >&2; then
        echo "WARN: failed to delete remote branch (continuing): ${temp_branch}" >&2
    fi
    git branch -D "${temp_branch}" >/dev/null 2>&1 || true
fi

if [[ "${conclusion}" != "success" ]]; then
    exit 1
fi

echo "OK: feature smoke passed" >&2
