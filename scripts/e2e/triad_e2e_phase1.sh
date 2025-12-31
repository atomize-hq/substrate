#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/e2e/triad_e2e_phase1.sh [options]

Purpose:
  Phase 1 of the end-to-end triad automation smoke:
    - scaffold an automation-enabled, cross-platform Planning Pack
    - write minimal (lintable) docs + ADR + smoke scripts + deterministic kickoff prompts
    - complete the feature start gate (F0-exec-preflight) on orchestration branch
    - start C0-code + C0-test in parallel worktrees and launch Codex headless for both
    - finish C0-code + C0-test (commit to their task branches; no merge to orchestration)

Options:
  --feature <name>            Feature dir name under docs/project_management/next/ (default: e2e-triad-smoke-<utc>)
  --remote <name>             Git remote for pushes/CI (default: origin)
  --runner-kind <kind>        github-hosted|self-hosted (default: self-hosted)
  --run-wsl                   Include WSL coverage in smoke (requires self-hosted runners)
  --wsl-separate              Scaffold a separate WSL platform-fix task (requires --run-wsl)

  --codex-profile <p>         Passed to Codex (`codex exec --profile`)
  --codex-model <m>           Passed to Codex (`codex exec --model`)
  --codex-jsonl               Capture Codex JSONL events (uses `codex exec --json`)
  --skip-codex                Do not launch Codex (still creates worktrees; you must edit manually)

  --skip-planning-lint        Skip `make planning-lint` (still runs JSON validation unless skipped)
  --skip-planning-validate    Skip `make planning-validate`
  --skip-sequencing-update    Do not add a temporary entry to docs/project_management/next/sequencing.json

  --push-orch                 Push orchestration branch to remote (recommended if you will run Phase 2 CI)

  --log-dir <dir>             Log directory (default: target/e2e/<feature>/)
  --dry-run                   Print actions; do not mutate git/worktrees

Output:
  Prints the feature directory and the log paths.
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

utc_now_compact() {
    date -u +%Y%m%dT%H%M%SZ
}

utc_now() {
    date -u +%Y-%m-%dT%H:%M:%SZ
}

python_abs_path() {
    python3 - "$1" <<'PY'
import os
import sys

p = sys.argv[1]
if os.path.isabs(p):
    print(os.path.realpath(p))
else:
    print(os.path.realpath(os.path.join(os.getcwd(), p)))
PY
}

log() {
    echo "== $*" >&2
}

run() {
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ $*" >&2
        return 0
    fi
    echo "+ $*" >&2
    "$@"
}

append_session_log() {
    local session_log="$1"
    local line="$2"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ append ${session_log}: ${line}" >&2
        return 0
    fi
    printf '%s\n' "${line}" >>"${session_log}"
}

set_task_status() {
    local tasks_json="$1"
    local task_id="$2"
    local status="$3"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ set ${tasks_json} ${task_id}.status=${status}" >&2
        return 0
    fi
    python3 - "${tasks_json}" "${task_id}" "${status}" <<'PY'
import json
import sys

path, task_id, status = sys.argv[1], sys.argv[2], sys.argv[3]
with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)

tasks = data.get("tasks")
if not isinstance(tasks, list):
    raise SystemExit("tasks.json: missing tasks[]")

found = False
for t in tasks:
    if isinstance(t, dict) and t.get("id") == task_id:
        t["status"] = status
        found = True
        break
if not found:
    raise SystemExit(f"tasks.json: task not found: {task_id}")

tmp = path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
import os
os.replace(tmp, path)
PY
}

FEATURE="e2e-triad-smoke-$(utc_now_compact)"
REMOTE="origin"
RUNNER_KIND="self-hosted"
RUN_WSL=0
WSL_SEPARATE=0

CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
SKIP_CODEX=0

SKIP_PLANNING_LINT=0
SKIP_PLANNING_VALIDATE=0
SKIP_SEQUENCING_UPDATE=0
PUSH_ORCH=0

LOG_DIR=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature)
            FEATURE="${2:-}"
            shift 2
            ;;
        --remote)
            REMOTE="${2:-}"
            shift 2
            ;;
        --runner-kind)
            RUNNER_KIND="${2:-}"
            shift 2
            ;;
        --run-wsl)
            RUN_WSL=1
            shift 1
            ;;
        --wsl-separate)
            WSL_SEPARATE=1
            shift 1
            ;;
        --codex-profile)
            CODEX_PROFILE="${2:-}"
            shift 2
            ;;
        --codex-model)
            CODEX_MODEL="${2:-}"
            shift 2
            ;;
        --codex-jsonl)
            CODEX_JSONL=1
            shift 1
            ;;
        --skip-codex)
            SKIP_CODEX=1
            shift 1
            ;;
        --skip-planning-lint)
            SKIP_PLANNING_LINT=1
            shift 1
            ;;
        --skip-planning-validate)
            SKIP_PLANNING_VALIDATE=1
            shift 1
            ;;
        --skip-sequencing-update)
            SKIP_SEQUENCING_UPDATE=1
            shift 1
            ;;
        --push-orch)
            PUSH_ORCH=1
            shift 1
            ;;
        --log-dir)
            LOG_DIR="${2:-}"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=1
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

if [[ "${RUN_WSL}" -eq 0 && "${WSL_SEPARATE}" -eq 1 ]]; then
    die "--wsl-separate requires --run-wsl"
fi

case "${RUNNER_KIND}" in
    github-hosted|self-hosted) ;;
    *) die "Invalid --runner-kind: ${RUNNER_KIND}" ;;
esac

require_cmd git
require_cmd jq
require_cmd rg
require_cmd python3
require_cmd make

if [[ "${SKIP_CODEX}" -eq 0 ]]; then
    require_cmd codex
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

ORCH_BRANCH="feat/${FEATURE}"
FEATURE_DIR="docs/project_management/next/${FEATURE}"
FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"

if [[ -z "${LOG_DIR}" ]]; then
    LOG_DIR="target/e2e/${FEATURE}"
fi
LOG_DIR_ABS="$(python_abs_path "${LOG_DIR}")"
mkdir -p "${LOG_DIR_ABS}"
LOG_PATH="${LOG_DIR_ABS}/phase1.log"

exec > >(tee -a "${LOG_PATH}") 2>&1

log "Repo: ${REPO_ROOT}"
log "Feature: ${FEATURE_DIR}"
log "Orchestration branch: ${ORCH_BRANCH}"
log "Log: ${LOG_PATH}"

if [[ "${DRY_RUN}" -ne 1 ]]; then
    if ! git diff --quiet || ! git diff --cached --quiet; then
        die "Working tree is not clean; commit/stash before running e2e"
    fi
fi

if [[ -e "${FEATURE_DIR}" ]]; then
    die "Feature dir already exists: ${FEATURE_DIR} (choose a different --feature)"
fi

if git show-ref --verify --quiet "refs/heads/${ORCH_BRANCH}"; then
    die "Branch already exists: ${ORCH_BRANCH} (choose a different --feature)"
fi

log "Creating orchestration branch: ${ORCH_BRANCH}"
run git checkout -b "${ORCH_BRANCH}"

log "Scaffolding Planning Pack (cross-platform + automation)"
scaffold_args=(scripts/planning/new_feature.sh --feature "${FEATURE}" --cross-platform --automation)
if [[ "${RUN_WSL}" -eq 1 ]]; then
    scaffold_args+=(--wsl-required)
    if [[ "${WSL_SEPARATE}" -eq 1 ]]; then
        scaffold_args+=(--wsl-separate)
    fi
fi
run "${scaffold_args[@]}"

TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
SESSION_LOG="${FEATURE_DIR_ABS}/session_log.md"

log "Writing minimal C0 spec"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ write ${FEATURE_DIR}/C0-spec.md" >&2
else
    cat >"${FEATURE_DIR_ABS}/C0-spec.md" <<'MD'
# C0-spec (E2E triad smoke)

## Scope
- Add a new workspace member crate `crates/triad_e2e_smoke_demo/`.
- Expose `pub fn answer() -> u32` returning `42`.
- Add a minimal test proving `answer() == 42`.

## Behavior
- The crate builds on Linux/macOS/Windows.
- `cargo test -p triad_e2e_smoke_demo` passes.

## Acceptance criteria
- `crates/triad_e2e_smoke_demo/Cargo.toml` exists and is a valid Rust crate.
- `crates/triad_e2e_smoke_demo/src/lib.rs` defines `answer()` returning `42`.
- `crates/triad_e2e_smoke_demo/tests/answer.rs` asserts `answer() == 42`.
- `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` succeed.

## Out of scope
- Any behavior changes to existing Substrate functionality.
MD
fi

log "Writing minimal ADR (with Executive Summary hash)"
ADR_PATH="${FEATURE_DIR_ABS}/ADR-0001-e2e-triad-smoke.md"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ write ${ADR_PATH}" >&2
else
    cat >"${ADR_PATH}" <<MD
# ADR-0001: E2E triad automation smoke

## Executive Summary (Operator)

ADR_BODY_SHA256: placeholder

- Existing: No single scripted end-to-end run proves planning + triad automation + CI smoke wiring works together.
- New: Add a temporary Planning Pack + triad execution run that exercises worktrees, Codex headless launch, CI smoke dispatch, and final FF merge-back.
- Why: Catch workflow/automation bugs early with a deterministic, repeatable smoke scenario.

## Decision
- Use an automation-enabled Planning Pack (tasks.json schema v3 + meta.automation.enabled=true).
- Use the cross-platform integration model (integ-core + platform-fix + final aggregator).

## Notes
- This feature is intended for workflow validation and can be removed after debugging.
MD
    python3 scripts/planning/check_adr_exec_summary.py --adr "${ADR_PATH}" --fix
fi

log "Writing quality gate report (ACCEPT) and execution preflight report (ACCEPT)"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ write ${FEATURE_DIR}/quality_gate_report.md" >&2
    echo "+ write ${FEATURE_DIR}/execution_preflight_report.md" >&2
else
    cat >"${FEATURE_DIR_ABS}/quality_gate_report.md" <<MD
# Quality Gate Report

RECOMMENDATION: ACCEPT

Notes:
- E2E smoke Planning Pack; minimal but execution-ready.
MD
    cat >"${FEATURE_DIR_ABS}/execution_preflight_report.md" <<MD
# Execution Preflight Report

RECOMMENDATION: ACCEPT

Notes:
- E2E smoke run; smoke scripts validate the demo crate builds/tests on all platforms.
MD
fi

log "Writing deterministic kickoff prompts (E2E)"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ write ${FEATURE_DIR}/kickoff_prompts/* (override)" >&2
else
    cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-code.md" <<'MD'
# Kickoff: C0-code (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Create a new workspace member crate at `crates/triad_e2e_smoke_demo/`.
- Implement `pub fn answer() -> u32 { 42 }` in `src/lib.rs`.
- Add the crate to the workspace (root `Cargo.toml` members).

Constraints:
- Production code only; do not add tests in this task.
- Keep changes minimal and deterministic.

Required:
- Run `cargo fmt`
- Run `cargo clippy --workspace --all-targets -- -D warnings`

Finish:
- From inside this worktree run: `make triad-task-finish TASK_ID="C0-code"`
MD

    cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-test.md" <<'MD'
# Kickoff: C0-test (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Add `crates/triad_e2e_smoke_demo/tests/answer.rs` that asserts `triad_e2e_smoke_demo::answer() == 42`.

Constraints:
- Tests only; do not modify production code.
- Keep changes minimal and deterministic.

Finish:
- From inside this worktree run: `make triad-task-finish TASK_ID="C0-test"`
MD

    cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-integ-core.md" <<MD
# Kickoff: C0-integ-core (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Merge C0-code + C0-test branches, make the slice green, and dispatch cross-platform smoke via CI.

Steps:
1) Merge the task branches:
   - Merge the code branch and the test branch into this worktree.
2) Run required checks:
   - \`make integ-checks\`
3) Dispatch smoke:
   - \`make feature-smoke FEATURE_DIR="${FEATURE_DIR}" PLATFORM=all WORKFLOW_REF="${ORCH_BRANCH}"\`
4) If any platform smoke fails, start only failing platform-fix tasks:
   - \`make triad-task-start-platform-fixes FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="C0" PLATFORMS="linux,macos,windows"\`
5) After all failing platforms are green, start the final aggregator:
   - \`make triad-task-start-integ-final FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="C0"\`

Finish:
- From inside this worktree run: \`make triad-task-finish TASK_ID="C0-integ-core"\`
MD

    for p in linux macos windows; do
        cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-integ-${p}.md" <<MD
# Kickoff: C0-integ-${p} (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Confirm the merged slice is green on ${p} and fix if needed.

Steps:
1) Merge C0-integ-core into this branch.
2) Run CI smoke for ${p} until green (repeat after fixes):
   - \`make feature-smoke FEATURE_DIR="${FEATURE_DIR}" PLATFORM=${p} WORKFLOW_REF="${ORCH_BRANCH}"\`
3) Finish (commits to this task branch; does not merge back to orchestration):
   - \`make triad-task-finish TASK_ID="C0-integ-${p}"\`
MD
    done

    if jq -e '.meta.wsl_required == true and .meta.wsl_task_mode == "separate"' "${TASKS_JSON}" >/dev/null 2>&1; then
        cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-integ-wsl.md" <<MD
# Kickoff: C0-integ-wsl (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Confirm the merged slice is green in WSL and fix if needed.

Steps:
1) Merge C0-integ-core into this branch.
2) Run CI smoke for wsl until green:
   - \`make feature-smoke FEATURE_DIR="${FEATURE_DIR}" PLATFORM=wsl RUNNER_KIND=self-hosted WORKFLOW_REF="${ORCH_BRANCH}"\`
3) Finish:
   - \`make triad-task-finish TASK_ID="C0-integ-wsl"\`
MD
    fi

    cat >"${FEATURE_DIR_ABS}/kickoff_prompts/C0-integ.md" <<MD
# Kickoff: C0-integ (E2E smoke final aggregator)

Do not edit planning docs inside the worktree.

Goal:
- Merge C0-integ-core and any platform-fix branches, re-run checks + smoke, and fast-forward merge back to orchestration.

Required:
- \`make integ-checks\`
- \`make feature-smoke FEATURE_DIR="${FEATURE_DIR}" PLATFORM=all WORKFLOW_REF="${ORCH_BRANCH}"\`

Finish:
- From inside this worktree run: \`make triad-task-finish TASK_ID="C0-integ"\`
MD
fi

log "Writing smoke scripts that validate the demo crate (these must pass on all platforms)"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ write ${FEATURE_DIR}/smoke/*" >&2
else
    cat >"${FEATURE_DIR_ABS}/smoke/linux-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "== E2E smoke: cargo test -p triad_e2e_smoke_demo =="
cargo test -p triad_e2e_smoke_demo
SH
    chmod +x "${FEATURE_DIR_ABS}/smoke/linux-smoke.sh"

    cat >"${FEATURE_DIR_ABS}/smoke/macos-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "== E2E smoke: cargo test -p triad_e2e_smoke_demo =="
cargo test -p triad_e2e_smoke_demo
SH
    chmod +x "${FEATURE_DIR_ABS}/smoke/macos-smoke.sh"

    cat >"${FEATURE_DIR_ABS}/smoke/windows-smoke.ps1" <<'PS1'
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "== E2E smoke: cargo test -p triad_e2e_smoke_demo =="
cargo test -p triad_e2e_smoke_demo
PS1
fi

if [[ "${SKIP_SEQUENCING_UPDATE}" -eq 0 ]]; then
    log "Adding temporary sequencing.json entry (required for planning-lint)"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ update docs/project_management/next/sequencing.json" >&2
    else
        python3 - "${FEATURE_DIR}" "${FEATURE}" "${ORCH_BRANCH}" <<'PY'
import json
import sys
from pathlib import Path

path = Path("docs/project_management/next/sequencing.json")
data = json.loads(path.read_text(encoding="utf-8"))
sprints = data.get("sprints", [])
feat_dir = sys.argv[1]
feat_id = f"e2e_{sys.argv[2]}"
branch = sys.argv[3]

if any(s.get("directory") == feat_dir for s in sprints if isinstance(s, dict)):
    raise SystemExit("sequencing.json already has an entry for this directory")

order = max([s.get("order", 0) for s in sprints if isinstance(s, dict) and isinstance(s.get("order"), int)] + [0]) + 1
sprints.append(
    {
        "order": order,
        "id": feat_id,
        "title": "E2E triad automation smoke",
        "branch": branch,
        "directory": feat_dir,
        "plan": f"{feat_dir}/plan.md",
        "status": "not_started",
        "sequence": [{"id": "C0", "name": "E2E smoke slice"}],
    }
)
data["sprints"] = sprints
path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
    fi
fi

log "Planning validation/lint (optional via flags)"
if [[ "${SKIP_PLANNING_VALIDATE}" -eq 0 ]]; then
    run make planning-validate FEATURE_DIR="${FEATURE_DIR}"
fi
if [[ "${SKIP_PLANNING_LINT}" -eq 0 ]]; then
    run make planning-lint FEATURE_DIR="${FEATURE_DIR}"
fi

log "Committing Planning Pack on orchestration branch"
run git add "${FEATURE_DIR}" docs/project_management/next/sequencing.json || true
run git commit -m "docs: scaffold e2e triad smoke (${FEATURE})"

if [[ "${PUSH_ORCH}" -eq 1 ]]; then
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ git push -u ${REMOTE} ${ORCH_BRANCH}" >&2
    else
        if ! git remote get-url "${REMOTE}" >/dev/null 2>&1; then
            die "Remote not configured: ${REMOTE}"
        fi
        run git push -u "${REMOTE}" "${ORCH_BRANCH}"
    fi
fi

log "Completing feature start gate task (F0-exec-preflight) on orchestration branch"
set_task_status "${TASKS_JSON}" "F0-exec-preflight" "in_progress"
append_session_log "${SESSION_LOG}" ""
append_session_log "${SESSION_LOG}" "START $(utc_now) F0-exec-preflight"
set_task_status "${TASKS_JSON}" "F0-exec-preflight" "completed"
append_session_log "${SESSION_LOG}" "END   $(utc_now) F0-exec-preflight (ACCEPT)"
run git add "${TASKS_JSON}" "${SESSION_LOG}" "${FEATURE_DIR_ABS}/execution_preflight_report.md"
run git commit -m "docs: complete F0-exec-preflight (${FEATURE})"

log "Starting C0 code+test in parallel (worktrees + optional Codex headless)"
pair_cmd=(make triad-task-start-pair FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="C0")
if [[ "${SKIP_CODEX}" -eq 0 ]]; then pair_cmd+=(LAUNCH_CODEX=1); fi
if [[ -n "${CODEX_PROFILE}" ]]; then pair_cmd+=(CODEX_PROFILE="${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then pair_cmd+=(CODEX_MODEL="${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then pair_cmd+=(CODEX_JSONL=1); fi

pair_out="$("${pair_cmd[@]}")"
echo "${pair_out}"

parse_kv() {
    local key="$1"
    local text="$2"
    printf '%s' "${text}" | awk -F= -v k="${key}" '$1==k {sub($1"=","",$0); print $0}'
}

CODE_WORKTREE="$(parse_kv CODE_WORKTREE "${pair_out}")"
TEST_WORKTREE="$(parse_kv TEST_WORKTREE "${pair_out}")"

if [[ -z "${CODE_WORKTREE}" || -z "${TEST_WORKTREE}" ]]; then
    die "Could not parse worktree paths from triad-task-start-pair output"
fi

log "Finishing C0-code (commit only; no merge back)"
run bash -lc "cd \"${CODE_WORKTREE}\" && make triad-task-finish TASK_ID=\"C0-code\""

log "Finishing C0-test (commit only; no merge back)"
run bash -lc "cd \"${TEST_WORKTREE}\" && make triad-task-finish TASK_ID=\"C0-test\""

log "Marking C0-code and C0-test completed in tasks.json (orchestration branch)"
run git checkout "${ORCH_BRANCH}"
set_task_status "${TASKS_JSON}" "C0-code" "completed"
set_task_status "${TASKS_JSON}" "C0-test" "completed"
append_session_log "${SESSION_LOG}" ""
append_session_log "${SESSION_LOG}" "END   $(utc_now) C0-code (e2e smoke)"
append_session_log "${SESSION_LOG}" "END   $(utc_now) C0-test (e2e smoke)"
run git add "${TASKS_JSON}" "${SESSION_LOG}"
run git commit -m "docs: complete C0 code+test (${FEATURE})"

echo ""
echo "PHASE1_OK=1"
echo "FEATURE_DIR=${FEATURE_DIR}"
echo "ORCH_BRANCH=${ORCH_BRANCH}"
echo "LOG=${LOG_PATH}"
