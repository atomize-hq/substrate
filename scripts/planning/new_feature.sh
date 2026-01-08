#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/planning/new_feature.sh --feature <feature_dir_name> [--decision-heavy] [--cross-platform] [--behavior-platforms <csv>] [--ci-parity-platforms <csv>] [--wsl-required] [--wsl-separate] [--automation]

Example:
  scripts/planning/new_feature.sh --feature world-sync --decision-heavy --cross-platform

Creates:
  docs/project_management/next/<feature>/
    plan.md
    tasks.json
    session_log.md
    contract.md
    kickoff_prompts/
USAGE
}

FEATURE=""
DECISION_HEAVY=0
CROSS_PLATFORM=0
WSL_REQUIRED=0
WSL_SEPARATE=0
AUTOMATION=0
BEHAVIOR_PLATFORMS=""
CI_PARITY_PLATFORMS=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature)
            FEATURE="${2:-}"
            shift 2
            ;;
        --decision-heavy)
            DECISION_HEAVY=1
            shift 1
            ;;
        --cross-platform)
            CROSS_PLATFORM=1
            shift 1
            ;;
        --behavior-platforms)
            BEHAVIOR_PLATFORMS="${2:-}"
            shift 2
            ;;
        --ci-parity-platforms)
            CI_PARITY_PLATFORMS="${2:-}"
            shift 2
            ;;
        --wsl-required)
            WSL_REQUIRED=1
            shift 1
            ;;
        --wsl-separate)
            WSL_SEPARATE=1
            shift 1
            ;;
        --automation)
            AUTOMATION=1
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

if [[ -z "${FEATURE}" ]]; then
    echo "Missing --feature" >&2
    usage >&2
    exit 2
fi

if [[ "${WSL_REQUIRED}" -eq 0 && "${WSL_SEPARATE}" -eq 1 ]]; then
    echo "--wsl-separate requires --wsl-required" >&2
    exit 2
fi

if [[ "${CROSS_PLATFORM}" -eq 0 && ( "${WSL_REQUIRED}" -eq 1 || "${WSL_SEPARATE}" -eq 1 ) ]]; then
    echo "--wsl-required/--wsl-separate require --cross-platform" >&2
    exit 2
fi

if [[ "${CROSS_PLATFORM}" -eq 0 && ( -n "${BEHAVIOR_PLATFORMS}" || -n "${CI_PARITY_PLATFORMS}" ) ]]; then
    echo "--behavior-platforms/--ci-parity-platforms require --cross-platform" >&2
    exit 2
fi

if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    if [[ -z "${CI_PARITY_PLATFORMS}" ]]; then
        CI_PARITY_PLATFORMS="linux,macos,windows"
    fi
    if [[ -z "${BEHAVIOR_PLATFORMS}" ]]; then
        BEHAVIOR_PLATFORMS="${CI_PARITY_PLATFORMS}"
    fi
fi

FEATURE_DIR="docs/project_management/next/${FEATURE}"
TEMPLATES_DIR="docs/project_management/standards/templates"
ORCH_BRANCH="feat/${FEATURE}"

if [[ -e "${FEATURE_DIR}" ]]; then
    echo "Refusing to overwrite existing directory: ${FEATURE_DIR}" >&2
    exit 2
fi

NOW_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

render() {
    local tmpl="$1"
    local out="$2"
    local task_id="${3:-}"
    local spec_file="${4:-C0-spec.md}"
    local branch="${5:-}"
    local worktree="${6:-}"
    local platform="${7:-}"
    local slice_id="${8:-}"

    sed \
        -e "s|{{FEATURE}}|${FEATURE}|g" \
        -e "s|{{FEATURE_DIR}}|${FEATURE_DIR}|g" \
        -e "s|{{ORCH_BRANCH}}|${ORCH_BRANCH}|g" \
        -e "s|{{NOW_UTC}}|${NOW_UTC}|g" \
        -e "s|{{TASK_ID}}|${task_id}|g" \
        -e "s|{{SPEC_FILE}}|${spec_file}|g" \
        -e "s|{{BRANCH}}|${branch}|g" \
        -e "s|{{WORKTREE}}|${worktree}|g" \
        -e "s|{{PLATFORM}}|${platform}|g" \
        -e "s|{{SLICE_ID}}|${slice_id}|g" \
        "${tmpl}" >"${out}"
}

mkdir -p "${FEATURE_DIR}/kickoff_prompts"

render "${TEMPLATES_DIR}/plan.md.tmpl" "${FEATURE_DIR}/plan.md"
render "${TEMPLATES_DIR}/session_log.md.tmpl" "${FEATURE_DIR}/session_log.md"
render "${TEMPLATES_DIR}/contract.md.tmpl" "${FEATURE_DIR}/contract.md"

cat >"${FEATURE_DIR}/C0-spec.md" <<'MD'
# C0-spec

## Scope
- None yet.

## Behavior
- None yet.

## Acceptance criteria
- None yet.

## Out of scope
- None yet.
MD

render "${TEMPLATES_DIR}/execution_preflight_report.md.tmpl" "${FEATURE_DIR}/execution_preflight_report.md"
render "${TEMPLATES_DIR}/slice_closeout_report.md.tmpl" "${FEATURE_DIR}/C0-closeout_report.md" "" "C0-spec.md" "" "" "" "C0"

render "${TEMPLATES_DIR}/kickoff_exec_preflight.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/F0-exec-preflight.md" "F0-exec-preflight"
if [[ "${AUTOMATION}" -eq 1 ]]; then
    render "${TEMPLATES_DIR}/kickoff_feature_cleanup.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/FZ-feature-cleanup.md" "FZ-feature-cleanup"
fi

FEATURE="${FEATURE}" FEATURE_DIR="${FEATURE_DIR}" CROSS_PLATFORM="${CROSS_PLATFORM}" BEHAVIOR_PLATFORMS="${BEHAVIOR_PLATFORMS}" CI_PARITY_PLATFORMS="${CI_PARITY_PLATFORMS}" WSL_REQUIRED="${WSL_REQUIRED}" WSL_SEPARATE="${WSL_SEPARATE}" AUTOMATION="${AUTOMATION}" \
python3 - <<'PY'
import json
import os

feature = os.environ["FEATURE"]
feature_dir = os.environ["FEATURE_DIR"]
cross_platform = os.environ["CROSS_PLATFORM"] == "1"
behavior_platforms_csv = os.environ.get("BEHAVIOR_PLATFORMS", "")
ci_parity_platforms_csv = os.environ.get("CI_PARITY_PLATFORMS", "")
wsl_required = os.environ["WSL_REQUIRED"] == "1"
wsl_separate = os.environ["WSL_SEPARATE"] == "1"
automation = os.environ.get("AUTOMATION", "0") == "1"

tasks_path = os.path.join(feature_dir, "tasks.json")

ALLOWED_REQUIRED = {"linux", "macos", "windows"}


def parse_platform_csv(raw: str, field: str) -> list:
    if not raw:
        return []
    parts = [p.strip() for p in raw.split(",") if p.strip()]
    unknown = sorted({p for p in parts if p not in ALLOWED_REQUIRED})
    if unknown:
        raise SystemExit(f"ERROR: invalid {field} platform(s): {', '.join(unknown)} (allowed: {sorted(ALLOWED_REQUIRED)})")
    duplicates = sorted({p for p in parts if parts.count(p) > 1})
    if duplicates:
        raise SystemExit(f"ERROR: duplicate {field} platform(s): {', '.join(duplicates)}")
    return parts


ci_parity_platforms = parse_platform_csv(ci_parity_platforms_csv, "ci_parity_platforms_required")
behavior_platforms = parse_platform_csv(behavior_platforms_csv, "behavior_platforms_required")

if cross_platform:
    if not ci_parity_platforms:
        ci_parity_platforms = ["linux", "macos", "windows"]
    if not behavior_platforms:
        behavior_platforms = list(ci_parity_platforms)
else:
    if ci_parity_platforms or behavior_platforms:
        raise SystemExit("ERROR: behavior/CI parity platforms require cross_platform mode")

if wsl_required and "linux" not in behavior_platforms:
    raise SystemExit("ERROR: --wsl-required requires linux in behavior_platforms_required")


def kickoff(task_id: str) -> str:
    return os.path.join(feature_dir, "kickoff_prompts", f"{task_id}.md")


def refs(*extra: str) -> list:
    base = [os.path.join(feature_dir, "plan.md"), os.path.join(feature_dir, "C0-spec.md")]
    return base + [os.path.join(feature_dir, x) for x in extra]


def _branch(suffix: str) -> str:
    return f"{feature}-{suffix}" if automation else suffix


def code_task(task_id: str, other_id: str) -> dict:
    task = {
        "id": task_id,
        "name": "C0 slice (code)",
        "type": "code",
        "phase": "C0",
        "status": "pending",
        "description": "Implement C0 spec (production code only).",
        "references": refs(),
        "acceptance_criteria": ["Meets all acceptance criteria in C0-spec.md"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start-pair FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"C0\""
                if automation
                else f"Run: git worktree add -b c0-code wt/{feature}-c0-code feat/{feature}"
            ),
        ],
        "end_checklist": [
            "cargo fmt",
            "cargo clippy --workspace --all-targets -- -D warnings",
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{task_id}\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"code: {feature} {task_id}\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-code (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-c0-code",
        "integration_task": "C0-integ-core" if cross_platform else "C0-integ",
        "kickoff_prompt": os.path.join(feature_dir, "kickoff_prompts", f"{task_id}.md"),
        "depends_on": ["F0-exec-preflight"],
        "concurrent_with": [other_id],
    }
    if automation:
        task["git_branch"] = _branch("c0-code")
        task["required_make_targets"] = ["triad-code-checks"]
    return task


def test_task(task_id: str, other_id: str) -> dict:
    task = {
        "id": task_id,
        "name": "C0 slice (test)",
        "type": "test",
        "phase": "C0",
        "status": "pending",
        "description": "Add/modify tests for C0 spec (tests only).",
        "references": refs(),
        "acceptance_criteria": ["Tests enforce C0 acceptance criteria"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start-pair FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"C0\""
                if automation
                else f"Run: git worktree add -b c0-test wt/{feature}-c0-test feat/{feature}"
            ),
        ],
        "end_checklist": [
            "cargo fmt",
            "Run the targeted tests you add/touch",
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{task_id}\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"test: {feature} {task_id}\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-test (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-c0-test",
        "integration_task": "C0-integ-core" if cross_platform else "C0-integ",
        "kickoff_prompt": os.path.join(feature_dir, "kickoff_prompts", f"{task_id}.md"),
        "depends_on": ["F0-exec-preflight"],
        "concurrent_with": [other_id],
    }
    if automation:
        task["git_branch"] = _branch("c0-test")
        task["required_make_targets"] = ["triad-test-checks"]
    return task


def integ_core_task() -> dict:
    task: dict

    platform_to_smoke = {
        "linux": "smoke/linux-smoke.sh",
        "macos": "smoke/macos-smoke.sh",
        "windows": "smoke/windows-smoke.ps1",
    }
    smoke_refs = [platform_to_smoke[p] for p in behavior_platforms if p in platform_to_smoke]

    dispatch_base = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --workflow-ref "feat/{feature}"'
    dispatches = []
    if set(behavior_platforms) == {"linux", "macos", "windows"}:
        dispatches.append(dispatch_base + " --platform all" + (" --run-wsl" if wsl_required else "") + " --cleanup")
    else:
        for p in behavior_platforms:
            dispatches.append(dispatch_base + f" --platform {p}" + (" --run-wsl" if (p == "linux" and wsl_required) else "") + " --cleanup")

    end_checklist = [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
    ]
    for dispatch in dispatches:
        end_checklist.append(f"Dispatch behavioral smoke via CI: {dispatch} (record run ids/URLs)")
    end_checklist.extend(
        [
            f"If any platform smoke fails: start only failing platform-fix tasks via: make triad-task-start-platform-fixes-from-smoke FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"C0\" SMOKE_RUN_ID=\"<run-id>\"",
            f"After all failing platforms are green: start final aggregator via: make triad-task-start-integ-final FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"C0\"",
            (
                "From inside the worktree: make triad-task-finish TASK_ID=\"C0-integ-core\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} C0-integ-core\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-integ-core (per plan.md)"
            ),
        ]
    )

    task = {
        "id": "C0-integ-core",
        "name": "C0 slice (integration core)",
        "type": "integration",
        "phase": "C0",
        "status": "pending",
        "description": "Merge C0 code+tests and make the slice green on the primary dev platform.",
        "references": refs(*smoke_refs),
        "acceptance_criteria": ["Core slice is green under make integ-checks and matches the spec"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"C0-integ-core\""
                if automation
                else f"Run: git worktree add -b c0-integ-core wt/{feature}-c0-integ-core feat/{feature}"
            ),
        ],
        "end_checklist": end_checklist,
        "worktree": f"wt/{feature}-c0-integ-core",
        "integration_task": "C0-integ-core",
        "kickoff_prompt": kickoff("C0-integ-core"),
        "depends_on": ["C0-code", "C0-test"],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch("c0-integ-core")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = False
    return task


def integ_platform_task(platform: str) -> dict:
    smoke_required = platform in behavior_platforms or platform == "wsl"
    if platform == "linux":
        smoke_refs = ("smoke/linux-smoke.sh",)
        name = "C0 slice (integration Linux)"
        desc = "Linux platform-fix integration task (may be a no-op if already green)."
        if wsl_required and not wsl_separate:
            dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform linux --run-wsl --workflow-ref "feat/{feature}" --cleanup'
        else:
            dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform linux --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "macos":
        smoke_refs = ("smoke/macos-smoke.sh",)
        name = "C0 slice (integration macOS)"
        desc = "macOS platform-fix integration task (may be a no-op if already green)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform macos --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "windows":
        smoke_refs = ("smoke/windows-smoke.ps1",)
        name = "C0 slice (integration Windows)"
        desc = "Windows platform-fix integration task (may be a no-op if already green)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform windows --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "wsl":
        smoke_refs = ("smoke/linux-smoke.sh",)
        name = "C0 slice (integration WSL)"
        desc = "WSL platform-fix integration task (Linux-in-WSL)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform wsl --workflow-ref "feat/{feature}" --cleanup'
    else:
        raise SystemExit(f"unexpected platform: {platform}")

    if not smoke_required:
        smoke_refs = ()
        name = f"C0 slice (integration CI parity: {platform})"
        desc = f"{platform} CI parity fix task (compile/test/lint only; no behavioral smoke required for this platform)."
        dispatch = f"make ci-compile-parity CI_WORKFLOW_REF=\"feat/{feature}\" CI_REMOTE=origin CI_CLEANUP=1"

    task_id = f"C0-integ-{platform}"
    task = {
        "id": task_id,
        "name": name,
        "type": "integration",
        "phase": "C0",
        "status": "pending",
        "description": desc,
        "references": refs(*smoke_refs),
        "acceptance_criteria": [
            (f"{platform} smoke is green for this slice" if smoke_required else f"{platform} CI parity is green for this slice (no behavioral smoke required)")
        ],
        "start_checklist": [
            f"Run on {platform} host if possible",
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"{task_id}\""
                if automation
                else f"Run: git worktree add -b c0-integ-{platform} wt/{feature}-c0-integ-{platform} feat/{feature}"
            ),
        ],
        "end_checklist": [
            (f"Dispatch platform smoke via CI: {dispatch}" if smoke_required else f"Dispatch CI parity via: {dispatch}"),
            "If needed: fix + fmt/clippy + targeted tests",
            ("Ensure smoke is green; record run id/URL" if smoke_required else "Ensure CI parity is green; record run id/URL"),
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{task_id}\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} {task_id}\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-integ-{platform} (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-c0-integ-{platform}",
        "integration_task": task_id,
        "kickoff_prompt": kickoff(task_id),
        "depends_on": ["C0-integ-core"],
        "concurrent_with": [],
        "platform": platform,
        "runner": "github-actions",
        "workflow": (".github/workflows/feature-smoke.yml" if smoke_required else ".github/workflows/ci-compile-parity.yml"),
    }
    if automation:
        task["git_branch"] = _branch(f"c0-integ-{platform}")
        task["required_make_targets"] = ["triad-code-checks"]
        task["merge_to_orchestration"] = False
    return task


def integ_final_task(platform_tasks: list) -> dict:
    task: dict
    platform_to_smoke = {
        "linux": "smoke/linux-smoke.sh",
        "macos": "smoke/macos-smoke.sh",
        "windows": "smoke/windows-smoke.ps1",
    }
    smoke_refs = [platform_to_smoke[p] for p in behavior_platforms if p in platform_to_smoke]

    dispatch_base = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --workflow-ref "feat/{feature}"'
    dispatches = []
    if set(behavior_platforms) == {"linux", "macos", "windows"}:
        dispatches.append(dispatch_base + " --platform all" + (" --run-wsl" if wsl_required else "") + " --cleanup")
    else:
        for p in behavior_platforms:
            dispatches.append(dispatch_base + f" --platform {p}" + (" --run-wsl" if (p == "linux" and wsl_required) else "") + " --cleanup")

    end_checklist = [
        "Merge platform-fix branches (if any) + resolve conflicts",
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
    ]
    for dispatch in dispatches:
        end_checklist.append(f"Re-run behavioral smoke via CI: {dispatch}")
    end_checklist.extend(
        [
            f"Complete slice closeout gate report: {os.path.join(feature_dir, 'C0-closeout_report.md')}",
            (
                "From inside the worktree: make triad-task-finish TASK_ID=\"C0-integ\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} C0-integ\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-integ (per plan.md)"
            ),
        ]
    )

    task = {
        "id": "C0-integ",
        "name": "C0 slice (integration final)",
        "type": "integration",
        "phase": "C0",
        "status": "pending",
        "description": "Final integration: merge any platform fixes and confirm behavioral smoke + CI parity are green.",
        "references": refs(*smoke_refs, "C0-closeout_report.md"),
        "acceptance_criteria": ["All required platforms are green and the slice matches the spec"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"C0-integ\""
                if automation
                else f"Run: git worktree add -b c0-integ wt/{feature}-c0-integ feat/{feature}"
            ),
        ],
        "end_checklist": end_checklist,
        "worktree": f"wt/{feature}-c0-integ",
        "integration_task": "C0-integ",
        "kickoff_prompt": kickoff("C0-integ"),
        "depends_on": ["C0-integ-core"] + [f"C0-integ-{p}" for p in platform_tasks],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch("c0-integ")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = True
    return task


def integ_single_task() -> dict:
    task = {
        "id": "C0-integ",
        "name": "C0 slice (integration)",
        "type": "integration",
        "phase": "C0",
        "status": "pending",
        "description": "Integrate C0 code+tests, reconcile to spec, and run integration gate.",
        "references": refs("C0-closeout_report.md"),
        "acceptance_criteria": ["Slice is green under make integ-checks and matches the spec"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"C0-integ\""
                if automation
                else f"Run: git worktree add -b c0-integ wt/{feature}-c0-integ feat/{feature}"
            ),
        ],
        "end_checklist": [
            "cargo fmt",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "Run relevant tests",
            "make integ-checks",
            f"Complete slice closeout gate report: {os.path.join(feature_dir, 'C0-closeout_report.md')}",
            (
                "From inside the worktree: make triad-task-finish TASK_ID=\"C0-integ\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} C0-integ\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-c0-integ (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-c0-integ",
        "integration_task": "C0-integ",
        "kickoff_prompt": kickoff("C0-integ"),
        "depends_on": ["C0-code", "C0-test"],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch("c0-integ")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = True
    return task


meta = {
    "schema_version": 3 if automation else (2 if cross_platform else 1),
    "feature": feature,
    "cross_platform": cross_platform,
    "execution_gates": True,
}
if automation:
    meta["automation"] = {"enabled": True, "orchestration_branch": f"feat/{feature}"}

tasks = []

tasks.append(
    {
        "id": "F0-exec-preflight",
        "name": "Execution preflight gate (feature start)",
        "type": "ops",
        "phase": "F0",
        "status": "pending",
        "description": "Run the execution preflight gate to confirm smoke/manual/CI plans are adequate before starting triads.",
        "references": [
            os.path.join(feature_dir, "plan.md"),
            os.path.join(feature_dir, "tasks.json"),
            os.path.join(feature_dir, "session_log.md"),
            os.path.join(feature_dir, "execution_preflight_report.md"),
        ],
        "acceptance_criteria": ["Execution preflight recommendation recorded (ACCEPT or REVISE)"],
        "start_checklist": [
            f"Run: make triad-orch-ensure FEATURE_DIR=\"{feature_dir}\"" if automation else f"git checkout feat/{feature} && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, specs, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
        ],
        "end_checklist": [
            "Complete execution_preflight_report.md with ACCEPT/REVISE and required fixes",
            "Set status to completed; add END entry; commit docs",
        ],
        "worktree": None,
        "integration_task": None,
        "kickoff_prompt": os.path.join(feature_dir, "kickoff_prompts", "F0-exec-preflight.md"),
        "depends_on": [],
        "concurrent_with": [],
    }
)
tasks.append(code_task("C0-code", "C0-test"))
tasks.append(test_task("C0-test", "C0-code"))

if cross_platform:
    meta["behavior_platforms_required"] = behavior_platforms
    meta["ci_parity_platforms_required"] = ci_parity_platforms
    if wsl_required:
        meta["wsl_required"] = True
        meta["wsl_task_mode"] = "separate" if wsl_separate else "bundled"

    tasks.append(integ_core_task())

    platforms = list(ci_parity_platforms)
    if wsl_required and wsl_separate:
        platforms.append("wsl")

    for platform in platforms:
        tasks.append(integ_platform_task(platform))

    tasks.append(integ_final_task(platforms))
else:
    tasks.append(integ_single_task())

if automation:
    tasks.append(
        {
            "id": "FZ-feature-cleanup",
            "name": "Feature cleanup (worktrees + branches)",
            "type": "ops",
            "phase": "FZ",
            "status": "pending",
            "description": "At feature end, remove retained worktrees and optionally prune task branches via scripts/triad/feature_cleanup.sh.",
            "references": [
                os.path.join(feature_dir, "plan.md"),
                os.path.join(feature_dir, "tasks.json"),
                os.path.join(feature_dir, "session_log.md"),
                "scripts/triad/feature_cleanup.sh",
            ],
            "acceptance_criteria": [
                "All worktrees removed (or intentionally retained) and cleanup summary recorded in session_log.md",
            ],
            "start_checklist": [
                f"git checkout feat/{feature} && git pull --ff-only",
                "Confirm all tasks are completed and merged as intended",
                "Set status to in_progress; add START entry; commit docs",
            ],
            "end_checklist": [
                f"Run: make triad-feature-cleanup FEATURE_DIR=\"{feature_dir}\" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1",
                f"Then run: make triad-feature-cleanup FEATURE_DIR=\"{feature_dir}\" REMOVE_WORKTREES=1 PRUNE_LOCAL=1",
                "Set status to completed; add END entry with script summary; commit docs",
            ],
            "worktree": None,
            "integration_task": None,
            "kickoff_prompt": os.path.join(feature_dir, "kickoff_prompts", "FZ-feature-cleanup.md"),
            "depends_on": ["C0-integ"],
            "concurrent_with": [],
        }
    )

data = {"meta": meta, "tasks": tasks}
with open(tasks_path, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2)
    handle.write("\n")
PY

: <<'LEGACY_TASKS_JSON'
if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    cat >"${FEATURE_DIR}/tasks.json" <<JSON
{
  "meta": {
    "schema_version": 2,
    "feature": "${FEATURE}",
    "cross_platform": true,
    "platforms_required": ["linux", "macos", "windows"]$( [[ "${WSL_REQUIRED}" -eq 1 ]] && echo "," || echo "" )
    $( [[ "${WSL_REQUIRED}" -eq 1 ]] && echo "\"wsl_required\": true," || echo "" )
    $( [[ "${WSL_REQUIRED}" -eq 1 ]] && echo "\"wsl_task_mode\": \"$( [[ \"${WSL_SEPARATE}\" -eq 1 ]] && echo "separate" || echo "bundled" )\"" || echo "" )
  },
  "tasks": [
    {
      "id": "C0-code",
      "name": "C0 slice (code)",
      "type": "code",
      "phase": "C0",
      "status": "pending",
      "description": "Implement C0 spec (production code only).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Meets all acceptance criteria in C0-spec.md"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-code and worktree wt/${FEATURE}-c0-code; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Commit changes to the task branch; do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-code",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-code.md",
      "depends_on": [],
      "concurrent_with": ["C0-test"]
    },
    {
      "id": "C0-test",
      "name": "C0 slice (test)",
      "type": "test",
      "phase": "C0",
      "status": "pending",
      "description": "Add/modify tests for C0 spec (tests only).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Tests enforce C0 acceptance criteria"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-test and worktree wt/${FEATURE}-c0-test; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "Run the targeted tests you add/touch",
        "Commit changes to the task branch; do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-test",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-test.md",
      "depends_on": [],
      "concurrent_with": ["C0-code"]
    },
    {
      "id": "C0-integ-core",
      "name": "C0 slice (integration core)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "Merge C0 code+tests and make the slice green on the primary dev platform.",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Core slice is green under make integ-checks and matches the spec"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ-core and worktree wt/${FEATURE}-c0-integ-core; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Dispatch cross-platform smoke via CI (record run ids/URLs): scripts/ci/dispatch_feature_smoke.sh --feature-dir \"${FEATURE_DIR}\" --runner-kind self-hosted --platform all$( [[ \"${WSL_REQUIRED}\" -eq 1 ]] && echo \" --run-wsl\" || echo \"\" ) --workflow-ref \"feat/${FEATURE}\" --cleanup",
        "Commit worktree changes; do not merge to orchestration yet; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ-core",
      "integration_task": "C0-integ-core",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ-core.md",
      "depends_on": ["C0-code", "C0-test"],
      "concurrent_with": []
    },
    {
      "id": "C0-integ-linux",
      "name": "C0 slice (integration linux)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "Linux platform-fix integration task (may be a no-op if already green).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Linux smoke is green for this slice"],
      "start_checklist": [
        "Run on Linux host if possible",
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ-linux and worktree wt/${FEATURE}-c0-integ-linux; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "Dispatch platform smoke via CI: scripts/ci/dispatch_feature_smoke.sh --feature-dir \"${FEATURE_DIR}\" --runner-kind self-hosted --platform linux$( [[ \"${WSL_REQUIRED}\" -eq 1 && \"${WSL_SEPARATE}\" -eq 0 ]] && echo \" --run-wsl\" || echo \"\" ) --workflow-ref \"feat/${FEATURE}\" --cleanup",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure Linux smoke is green; record run id/URL",
        "Commit changes to the platform-fix branch (if any); do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ-linux",
      "integration_task": "C0-integ-linux",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ-linux.md",
      "depends_on": ["C0-integ-core"],
      "concurrent_with": [],
      "platform": "linux",
      "runner": "github-actions",
      "workflow": ".github/workflows/feature-smoke.yml"
    },
    {
      "id": "C0-integ-macos",
      "name": "C0 slice (integration macOS)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "macOS platform-fix integration task (may be a no-op if already green).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["macOS smoke is green for this slice"],
      "start_checklist": [
        "Run on macOS host if possible",
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ-macos and worktree wt/${FEATURE}-c0-integ-macos; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "Dispatch platform smoke via CI: scripts/ci/dispatch_feature_smoke.sh --feature-dir \"${FEATURE_DIR}\" --runner-kind self-hosted --platform macos --workflow-ref \"feat/${FEATURE}\" --cleanup",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure macOS smoke is green; record run id/URL",
        "Commit changes to the platform-fix branch (if any); do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ-macos",
      "integration_task": "C0-integ-macos",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ-macos.md",
      "depends_on": ["C0-integ-core"],
      "concurrent_with": [],
      "platform": "macos",
      "runner": "github-actions",
      "workflow": ".github/workflows/feature-smoke.yml"
    },
    {
      "id": "C0-integ-windows",
      "name": "C0 slice (integration Windows)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "Windows platform-fix integration task (may be a no-op if already green).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Windows smoke is green for this slice"],
      "start_checklist": [
        "Run on Windows host if possible",
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ-windows and worktree wt/${FEATURE}-c0-integ-windows; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "Dispatch platform smoke via CI: scripts/ci/dispatch_feature_smoke.sh --feature-dir \"${FEATURE_DIR}\" --runner-kind self-hosted --platform windows --workflow-ref \"feat/${FEATURE}\" --cleanup",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure Windows smoke is green; record run id/URL",
        "Commit changes to the platform-fix branch (if any); do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ-windows",
      "integration_task": "C0-integ-windows",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ-windows.md",
      "depends_on": ["C0-integ-core"],
      "concurrent_with": [],
      "platform": "windows",
      "runner": "github-actions",
      "workflow": ".github/workflows/feature-smoke.yml"
    },
    $( [[ "${WSL_REQUIRED}" -eq 1 && "${WSL_SEPARATE}" -eq 1 ]] && cat <<'WSLTASK'
    {
      "id": "C0-integ-wsl",
      "name": "C0 slice (integration WSL)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "WSL platform-fix integration task (Linux-in-WSL).",
      "references": ["{{FEATURE_DIR}}/plan.md", "{{FEATURE_DIR}}/C0-spec.md"],
      "acceptance_criteria": ["WSL smoke is green for this slice"],
      "start_checklist": [
        "Run on WSL runner host if possible",
        "git checkout feat/{{FEATURE}} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ-wsl and worktree wt/{{FEATURE}}-c0-integ-wsl; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "Dispatch WSL smoke via CI: scripts/ci/dispatch_feature_smoke.sh --feature-dir \"{{FEATURE_DIR}}\" --runner-kind self-hosted --platform wsl --workflow-ref \"feat/{{FEATURE}}\" --cleanup",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure WSL smoke is green; record run id/URL",
        "Commit changes to the platform-fix branch (if any); do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/{{FEATURE}}-c0-integ-wsl",
      "integration_task": "C0-integ-wsl",
      "kickoff_prompt": "{{FEATURE_DIR}}/kickoff_prompts/C0-integ-wsl.md",
      "depends_on": ["C0-integ-core"],
      "concurrent_with": [],
      "platform": "wsl",
      "runner": "github-actions",
      "workflow": ".github/workflows/feature-smoke.yml"
    },
WSLTASK
    )
    {
      "id": "C0-integ",
      "name": "C0 slice (integration final)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "Final cross-platform integration: merge any platform fixes and confirm all platforms are green.",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["All required platforms are green and the slice matches the spec"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ and worktree wt/${FEATURE}-c0-integ; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "Merge platform-fix branches (if any) + resolve conflicts",
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Dispatch cross-platform smoke via CI (record run ids/URLs): scripts/ci/dispatch_feature_smoke.sh --feature-dir \"${FEATURE_DIR}\" --runner-kind self-hosted --platform all$( [[ \"${WSL_REQUIRED}\" -eq 1 ]] && echo \" --run-wsl\" || echo \"\" ) --workflow-ref \"feat/${FEATURE}\" --cleanup",
        "Commit worktree changes; fast-forward merge this branch into feat/${FEATURE}; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ.md",
      "depends_on": ["C0-integ-core", "C0-integ-linux", "C0-integ-macos", "C0-integ-windows"$( [[ \"${WSL_REQUIRED}\" -eq 1 && \"${WSL_SEPARATE}\" -eq 1 ]] && echo ", \"C0-integ-wsl\"" || echo "" )],
      "concurrent_with": []
    }
  ]
}
JSON
else
    cat >"${FEATURE_DIR}/tasks.json" <<JSON
{
  "meta": {
    "schema_version": 2,
    "feature": "${FEATURE}",
    "cross_platform": false
  },
  "tasks": [
    {
      "id": "C0-code",
      "name": "C0 slice (code)",
      "type": "code",
      "phase": "C0",
      "status": "pending",
      "description": "Implement C0 spec (production code only).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Meets all acceptance criteria in C0-spec.md"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-code and worktree wt/${FEATURE}-c0-code; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Commit changes to the task branch; do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-code",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-code.md",
      "depends_on": [],
      "concurrent_with": ["C0-test"]
    },
    {
      "id": "C0-test",
      "name": "C0 slice (test)",
      "type": "test",
      "phase": "C0",
      "status": "pending",
      "description": "Add/modify tests for C0 spec (tests only).",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Tests enforce C0 acceptance criteria"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-test and worktree wt/${FEATURE}-c0-test; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "Run the targeted tests you add/touch",
        "Commit changes to the task branch; do not merge to orchestration; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-test",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-test.md",
      "depends_on": [],
      "concurrent_with": ["C0-code"]
    },
    {
      "id": "C0-integ",
      "name": "C0 slice (integration)",
      "type": "integration",
      "phase": "C0",
      "status": "pending",
      "description": "Integrate C0 code+tests, reconcile to spec, and run integration gate.",
      "references": ["${FEATURE_DIR}/plan.md", "${FEATURE_DIR}/C0-spec.md"],
      "acceptance_criteria": ["Slice is green under make integ-checks and matches the spec"],
      "start_checklist": [
        "git checkout feat/${FEATURE} && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        "Create branch c0-integ and worktree wt/${FEATURE}-c0-integ; do not edit planning docs inside the worktree"
      ],
      "end_checklist": [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Commit worktree changes; fast-forward merge this branch into feat/${FEATURE}; update docs; do not delete the worktree (cleanup at feature end)"
      ],
      "worktree": "wt/${FEATURE}-c0-integ",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ.md",
      "depends_on": ["C0-code", "C0-test"],
      "concurrent_with": []
    }
  ]
}
JSON
fi
LEGACY_TASKS_JSON

if [[ "${AUTOMATION}" -eq 1 ]]; then
    C0_CODE_BRANCH="${FEATURE}-c0-code"
    C0_TEST_BRANCH="${FEATURE}-c0-test"
    C0_INTEG_BRANCH="${FEATURE}-c0-integ"
    C0_INTEG_CORE_BRANCH="${FEATURE}-c0-integ-core"
else
    C0_CODE_BRANCH="c0-code"
    C0_TEST_BRANCH="c0-test"
    C0_INTEG_BRANCH="c0-integ"
    C0_INTEG_CORE_BRANCH="c0-integ-core"
fi

render "${TEMPLATES_DIR}/kickoff_code.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-code.md" "C0-code" "C0-spec.md" "${C0_CODE_BRANCH}" "wt/${FEATURE}-c0-code"
render "${TEMPLATES_DIR}/kickoff_test.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-test.md" "C0-test" "C0-spec.md" "${C0_TEST_BRANCH}" "wt/${FEATURE}-c0-test"
if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    render "${TEMPLATES_DIR}/kickoff_integ_core.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-core.md" "C0-integ-core" "C0-spec.md" "${C0_INTEG_CORE_BRANCH}" "wt/${FEATURE}-c0-integ-core"
    IFS=',' read -r -a ci_platforms <<<"${CI_PARITY_PLATFORMS}"
    for p in "${ci_platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        render "${TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-${p}.md" "C0-integ-${p}" "C0-spec.md" "${FEATURE}-c0-integ-${p}" "wt/${FEATURE}-c0-integ-${p}" "${p}"
    done
    if [[ "${WSL_REQUIRED}" -eq 1 && "${WSL_SEPARATE}" -eq 1 ]]; then
        render "${TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-wsl.md" "C0-integ-wsl" "C0-spec.md" "${FEATURE}-c0-integ-wsl" "wt/${FEATURE}-c0-integ-wsl" "wsl"
    fi
    render "${TEMPLATES_DIR}/kickoff_integ_final.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ.md" "C0-integ" "C0-spec.md" "${C0_INTEG_BRANCH}" "wt/${FEATURE}-c0-integ"
else
    render "${TEMPLATES_DIR}/kickoff_integ.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ.md" "C0-integ" "C0-spec.md" "${C0_INTEG_BRANCH}" "wt/${FEATURE}-c0-integ"
fi

if [[ "${DECISION_HEAVY}" -eq 1 || "${CROSS_PLATFORM}" -eq 1 ]]; then
    cat >"${FEATURE_DIR}/decision_register.md" <<'MD'
# Decision Register

Use the template in:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
MD

    cat >"${FEATURE_DIR}/integration_map.md" <<'MD'
# Integration Map

Use the standard in:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
MD

    if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
        {
            cat <<'MD'
# Manual Testing Playbook

This playbook must contain runnable commands and expected exit codes/output.

## Behavioral Smoke Scripts

These scripts define the behavioral platform contract for this feature. Keep them deterministic and fast.
MD
            IFS=',' read -r -a behavior_platforms <<<"${BEHAVIOR_PLATFORMS}"
            for p in "${behavior_platforms[@]}"; do
                p="$(echo "${p}" | xargs)"
                [[ -z "${p}" ]] && continue
                case "${p}" in
                    linux) echo "- Linux: \`bash smoke/linux-smoke.sh\` (expected exit: 0)" ;;
                    macos) echo "- macOS: \`bash smoke/macos-smoke.sh\` (expected exit: 0)" ;;
                    windows) echo "- Windows: \`pwsh -File smoke/windows-smoke.ps1\` (expected exit: 0)" ;;
                    *) echo "ERROR: invalid behavior platform: ${p}" >&2; exit 2 ;;
                esac
            done
            cat <<MD

## CI Parity (compile/test)

CI parity platforms (may be broader than behavioral scope): \`${CI_PARITY_PLATFORMS}\`

Recommended gates:
- \`make ci-compile-parity CI_WORKFLOW_REF="feat/${FEATURE}" CI_REMOTE=origin CI_CLEANUP=1\`
- \`scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/${FEATURE}" --remote origin --cleanup\`
MD
        } >"${FEATURE_DIR}/manual_testing_playbook.md"
    else
        cat >"${FEATURE_DIR}/manual_testing_playbook.md" <<'MD'
# Manual Testing Playbook

This playbook must contain runnable commands and expected exit codes/output.
MD
    fi
fi

if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    mkdir -p "${FEATURE_DIR}/smoke"
    IFS=',' read -r -a behavior_platforms <<<"${BEHAVIOR_PLATFORMS}"
    for p in "${behavior_platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        case "${p}" in
            linux)
                cat >"${FEATURE_DIR}/smoke/linux-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "Smoke script scaffold (linux) - replace with feature checks"
exit 1
SH
                ;;
            macos)
                cat >"${FEATURE_DIR}/smoke/macos-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "Smoke script scaffold (macos) - replace with feature checks"
exit 1
SH
                ;;
            windows)
                cat >"${FEATURE_DIR}/smoke/windows-smoke.ps1" <<'PS1'
param()
$ErrorActionPreference = "Stop"
Write-Host "Smoke script scaffold (windows) - replace with feature checks"
exit 1
PS1
                ;;
            *)
                echo "ERROR: invalid behavior platform: ${p}" >&2
                exit 2
                ;;
        esac
    done
fi

echo "OK: created ${FEATURE_DIR}"
