#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/ci/dispatch_ci_testing.sh \
    [--checkout-ref <git-ref>] \
    [--mode <mode>] \
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
  - Default workflow ref is the current git branch. Do not dispatch from `main` or `testing`;
    dispatch from the feature orchestration/task ref and rely on `checkout_ref` to run on the exact commit.

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

    # Cross-platform fallback (macOS often lacks `timeout`).
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

WORKFLOW=".github/workflows/ci-testing.yml"
WORKFLOW_REF=""
REMOTE="origin"
CLEANUP=0
CHECKOUT_REF=""
MODE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --checkout-ref)
            CHECKOUT_REF="${2:-}"
            shift 2
            ;;
        --mode)
            MODE="${2:-}"
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

GIT_PUSH_TIMEOUT_SECS="${CI_TESTING_GIT_PUSH_TIMEOUT_SECS:-300}"
GH_TIMEOUT_SECS="${CI_TESTING_GH_TIMEOUT_SECS:-120}"
WATCH_INTERVAL_SECS="${CI_TESTING_WATCH_INTERVAL_SECS:-15}"
WATCH_TIMEOUT_SECS="${CI_TESTING_WATCH_TIMEOUT_SECS:-7200}" # 2h
WATCH_MAX_CONSECUTIVE_ERRORS="${CI_TESTING_WATCH_MAX_CONSECUTIVE_ERRORS:-20}"
RUN_LOOKUP_TIMEOUT_SECS="${CI_TESTING_RUN_LOOKUP_TIMEOUT_SECS:-120}"

if [[ -z "${WORKFLOW_REF}" ]]; then
    WORKFLOW_REF="$(git branch --show-current 2>/dev/null || true)"
    if [[ -z "${WORKFLOW_REF}" ]]; then
        die "Missing --workflow-ref (ref must not be main/testing; use the orchestration/task ref)"
    fi
fi

if [[ -z "${CHECKOUT_REF}" ]]; then
    CHECKOUT_REF="$(git rev-parse HEAD)"
fi

case "${MODE}" in
    ""|full|quick|compile-parity) ;;
    *) die "Invalid --mode: ${MODE} (expected full|quick|compile-parity)" ;;
esac

head_sha="$(git rev-parse "${CHECKOUT_REF}")"
ts="$(date -u +%Y%m%dT%H%M%SZ)"
temp_branch_prefix="tmp/ci-testing"
if [[ "${MODE}" == "compile-parity" ]]; then
    temp_branch_prefix="tmp/ci-compile-parity"
fi
temp_branch="${temp_branch_prefix}/${ts}"

echo "HEAD: ${head_sha}" >&2
echo "Temp branch: ${temp_branch}" >&2

git branch -f "${temp_branch}" "${head_sha}"
if ! run_with_timeout "${GIT_PUSH_TIMEOUT_SECS}" git push -u "${REMOTE}" "${temp_branch}:${temp_branch}" >&2; then
    die "git push timed out or failed (branch=${temp_branch})"
fi

echo "Dispatching workflow: ${WORKFLOW}" >&2
echo "Workflow ref: ${WORKFLOW_REF}" >&2
effective_workflow="${WORKFLOW}"
effective_ref="${WORKFLOW_REF}"

repo="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
workflow_file="$(basename "${WORKFLOW}")"
legacy_workflow=".github/workflows/ci-testing.yml"
if [[ -n "${repo}" ]] && ! gh api "repos/${repo}/actions/workflows/${workflow_file}" >/dev/null 2>&1; then
    legacy_file="$(basename "${legacy_workflow}")"
    if gh api "repos/${repo}/actions/workflows/${legacy_file}" >/dev/null 2>&1; then
        echo "WARN: ${WORKFLOW} not registered on default branch; falling back to ${legacy_workflow}" >&2
        effective_workflow="${legacy_workflow}"
    else
        echo "WARN: ${WORKFLOW} not registered on default branch, and legacy workflow is unavailable; continuing with ${WORKFLOW}" >&2
    fi
fi

dispatch_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
dispatch_err="$(mktemp)"

dispatch_args=(gh workflow run "${effective_workflow}" --ref "${effective_ref}" -f checkout_ref="${temp_branch}")
if [[ -n "${MODE}" ]]; then
    dispatch_args+=(-f mode="${MODE}")
fi

set +e
run_with_timeout "${GH_TIMEOUT_SECS}" "${dispatch_args[@]}" >/dev/null 2>"${dispatch_err}"
dispatch_rc="$?"
set -e

if [[ "${dispatch_rc}" -ne 0 ]]; then
    err_msg="$(cat "${dispatch_err}" || true)"

    # Back-compat: if the workflow doesn't accept `mode`, retry once without it.
    if [[ -n "${MODE}" ]] && grep -Fq "Unexpected inputs provided" <<<"${err_msg}"; then
        echo "WARN: workflow does not accept mode input; retrying without mode" >&2
        : >"${dispatch_err}"
        set +e
        run_with_timeout "${GH_TIMEOUT_SECS}" gh workflow run "${effective_workflow}" --ref "${effective_ref}" -f checkout_ref="${temp_branch}" >/dev/null 2>"${dispatch_err}"
        dispatch_rc="$?"
        set -e
        err_msg="$(cat "${dispatch_err}" || true)"
    fi

    # If dispatching from a non-stable ref fails due to trigger discovery, retry from `testing`.
    if [[ "${dispatch_rc}" -ne 0 ]] && [[ "${effective_ref}" != "testing" ]] && grep -Fq "does not have 'workflow_dispatch' trigger" <<<"${err_msg}"; then
        echo "WARN: dispatch failed on ref=${effective_ref}; retrying with workflow ref=testing" >&2
        effective_ref="testing"
        dispatch_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        : >"${dispatch_err}"
        dispatch_args=(gh workflow run "${effective_workflow}" --ref "${effective_ref}" -f checkout_ref="${temp_branch}")
        if [[ -n "${MODE}" ]]; then
            dispatch_args+=(-f mode="${MODE}")
        fi
        set +e
        run_with_timeout "${GH_TIMEOUT_SECS}" "${dispatch_args[@]}" >/dev/null 2>"${dispatch_err}"
        dispatch_rc="$?"
        set -e
        err_msg="$(cat "${dispatch_err}" || true)"
    fi

    if [[ "${dispatch_rc}" -ne 0 ]]; then
        echo "${err_msg}" >&2
        rm -f "${dispatch_err}" >/dev/null 2>&1 || true
        die "failed to dispatch workflow via gh (workflow=${effective_workflow} ref=${effective_ref})"
    fi
fi

rm -f "${dispatch_err}" >/dev/null 2>&1 || true

echo "Waiting for run to start..." >&2
started_lookup_at="$(date +%s)"
run_id=""
while [[ -z "${run_id}" ]]; do
    run_id="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run list --workflow "${effective_workflow}" --event workflow_dispatch --branch "${effective_ref}" --limit 30 --json databaseId,createdAt -q "map(select(.createdAt >= \"${dispatch_started}\")) | .[0].databaseId" 2>/dev/null || true)"
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

run_url="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json url -q '.url' 2>/dev/null || true)"

started_watch_at="$(date +%s)"
next_heartbeat_at="$((started_watch_at + 60))"
consecutive_errors=0

echo "Watching run status (interval=${WATCH_INTERVAL_SECS}s timeout=${WATCH_TIMEOUT_SECS}s)..." >&2
while true; do
    now="$(date +%s)"
    elapsed="$((now - started_watch_at))"
    if [[ "${elapsed}" -ge "${WATCH_TIMEOUT_SECS}" ]]; then
        die "Timed out waiting for CI Testing run ${run_id} to complete after ${elapsed}s"
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

jobs_json="$(run_with_timeout "${GH_TIMEOUT_SECS}" gh run view "${run_id}" --json jobs 2>/dev/null || true)"
passed_oses=""
failed_oses=""
failed_jobs=""
if [[ -n "${jobs_json}" ]]; then
    parsed="$(
        python3 - "${jobs_json}" <<'PY'
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
    )" || true
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
    if ! run_with_timeout "${GIT_PUSH_TIMEOUT_SECS}" git push "${REMOTE}" ":${temp_branch}" >&2; then
        echo "WARN: failed to delete remote branch (continuing): ${temp_branch}" >&2
    fi
    git branch -D "${temp_branch}" >/dev/null 2>&1 || true
fi

if [[ "${conclusion}" != "success" ]]; then
    exit 1
fi
