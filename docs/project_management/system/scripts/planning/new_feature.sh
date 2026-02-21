#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  make planning-new-feature FEATURE=<feature_dir_name> [DECISION_HEAVY=1] [CROSS_PLATFORM=1] [AUTOMATION=1] [...]

Example:
  make planning-new-feature FEATURE=world-sync DECISION_HEAVY=1 CROSS_PLATFORM=1

Creates:
  docs/project_management/packs/active/<feature>/   (canonical)
    plan.md
    tasks.json
    session_log.md
    contract.md
    spec_manifest.md
    impact_map.md
    slices/<SLICE_ID>/
      <SLICE_ID>-spec.md
      <SLICE_ID>-closeout_report.md
    kickoff_prompts/
USAGE
}

FEATURE=""
SLICE_PREFIX=""
DECISION_HEAVY=0
CROSS_PLATFORM=0
WSL_REQUIRED=0
WSL_SEPARATE=0
AUTOMATION=0
BEHAVIOR_PLATFORMS=""
CI_PARITY_PLATFORMS=""
SLICE_DIRS=1

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature)
            FEATURE="${2:-}"
            shift 2
            ;;
        --slice-prefix)
            SLICE_PREFIX="${2:-}"
            shift 2
            ;;
        --slice-dirs)
            SLICE_DIRS=1
            shift 1
            ;;
        --flat-slice-files)
            SLICE_DIRS=0
            shift 1
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null)" || {
    echo "ERROR: failed to locate repo root via git" >&2
    exit 2
}
cd "${REPO_ROOT}"

derive_slice_prefix() {
    local raw="$1"
    local -a parts=()
    IFS='-_.' read -r -a parts <<<"${raw}"

    local -a words=()
    for w in "${parts[@]}"; do
        w="$(echo "$w" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+//g')"
        [[ -z "$w" ]] && continue
        case "$w" in
            and|or|the|a|an|of|to|for|in|on|with|via|vs|by) continue ;;
        esac
        words+=("$w")
    done

    if [[ "${#words[@]}" -eq 0 ]]; then
        echo "X"
        return 0
    fi

    # NOTE: macOS ships bash 3.2 which does not support negative array indices.
    local last_index=$(( ${#words[@]} - 1 ))
    local last="${words[${last_index}]}"
    case "$last" in
        simplification|refactor|cleanup|hardening|migration|parity|stability|integration|reliability|improvement|improvements|fix|fixes|update|updates)
            if [[ "${#words[@]}" -ge 3 ]]; then
                printf "%s%s%s" "${words[0]:0:1}" "${words[1]:0:1}" "${words[2]:0:1}" | tr '[:lower:]' '[:upper:]'
                return 0
            fi
            ;;
    esac

    if [[ "${#words[@]}" -ge 3 ]]; then
        printf "%s%s%s" "${words[0]:0:1}" "${words[1]:0:1}" "${words[${last_index}]:0:1}" | tr '[:lower:]' '[:upper:]'
        return 0
    fi

    if [[ "${#words[@]}" -eq 2 ]]; then
        printf "%s%s" "${words[0]:0:1}" "${words[1]:0:1}" | tr '[:lower:]' '[:upper:]'
        return 0
    fi

    echo "${words[0]:0:3}" | tr '[:lower:]' '[:upper:]'
}

if [[ -z "${SLICE_PREFIX}" ]]; then
    SLICE_PREFIX="$(derive_slice_prefix "${FEATURE}")"
fi

if ! [[ "${SLICE_PREFIX}" =~ ^[A-Za-z][A-Za-z0-9]*$ ]]; then
    echo "Invalid --slice-prefix: ${SLICE_PREFIX} (expected alnum, starting with a letter)" >&2
    exit 2
fi

SLICE_ID="${SLICE_PREFIX}0"
SLICE_ID_LOWER="$(echo "${SLICE_ID}" | tr '[:upper:]' '[:lower:]')"

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

pm_roots_json="$(python3 "${SCRIPT_DIR}/pm_paths.py" print-roots 2>/dev/null)" || {
    echo "ERROR: failed to resolve PM roots (pm_paths.py print-roots)" >&2
    exit 2
}
PM_PACKS_ROOT="$(
    python3 -c 'import json,sys; print(json.load(sys.stdin)["pm_packs_root"])' <<<"${pm_roots_json}" 2>/dev/null || true
)"
if [[ -z "${PM_PACKS_ROOT}" ]]; then
    echo "ERROR: pm_paths.py print-roots returned empty pm_packs_root" >&2
    exit 2
fi

FEATURE_DIR="${PM_PACKS_ROOT%/}/active/${FEATURE}"
PLANNING_TEMPLATES_DIR="docs/project_management/system/templates/planning_pack"
KICKOFF_TEMPLATES_DIR="docs/project_management/system/templates/kickoff"
ORCH_BRANCH="feat/${FEATURE}"

if [[ -e "${FEATURE_DIR}" ]]; then
    echo "Refusing to overwrite existing directory: ${FEATURE_DIR}" >&2
    exit 2
fi

NOW_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

SLICE_SPEC_REL="${SLICE_ID}-spec.md"
SLICE_CLOSEOUT_REL="${SLICE_ID}-closeout_report.md"
SLICE_KICKOFF_DIR_REL="kickoff_prompts"
if [[ "${SLICE_DIRS}" -eq 1 ]]; then
    SLICE_SPEC_REL="slices/${SLICE_ID}/${SLICE_ID}-spec.md"
    SLICE_CLOSEOUT_REL="slices/${SLICE_ID}/${SLICE_ID}-closeout_report.md"
    SLICE_KICKOFF_DIR_REL="slices/${SLICE_ID}/kickoff_prompts"
fi

render() {
    local tmpl="$1"
    local out="$2"
    local task_id="${3:-}"
    local spec_file="${4:-${SLICE_SPEC_REL}}"
    local branch="${5:-}"
    local worktree="${6:-}"
    local platform="${7:-}"
    local slice_id="${8:-${SLICE_ID}}"
    local closeout_file="${9:-${SLICE_CLOSEOUT_REL}}"

    sed \
        -e "s|{{FEATURE}}|${FEATURE}|g" \
        -e "s|{{FEATURE_DIR}}|${FEATURE_DIR}|g" \
        -e "s|{{ORCH_BRANCH}}|${ORCH_BRANCH}|g" \
        -e "s|{{NOW_UTC}}|${NOW_UTC}|g" \
        -e "s|{{SLICE_PREFIX}}|${SLICE_PREFIX}|g" \
        -e "s|{{TASK_ID}}|${task_id}|g" \
        -e "s|{{SPEC_FILE}}|${spec_file}|g" \
        -e "s|{{CLOSEOUT_FILE}}|${closeout_file}|g" \
        -e "s|{{BRANCH}}|${branch}|g" \
        -e "s|{{WORKTREE}}|${worktree}|g" \
        -e "s|{{PLATFORM}}|${platform}|g" \
        -e "s|{{SLICE_ID}}|${slice_id}|g" \
        "${tmpl}" >"${out}"
}

mkdir -p "${FEATURE_DIR}/kickoff_prompts"
if [[ "${SLICE_DIRS}" -eq 1 ]]; then
    mkdir -p "${FEATURE_DIR}/slices/${SLICE_ID}"
fi
mkdir -p "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}"

render "${PLANNING_TEMPLATES_DIR}/plan.md.tmpl" "${FEATURE_DIR}/plan.md"
render "${PLANNING_TEMPLATES_DIR}/session_log.md.tmpl" "${FEATURE_DIR}/session_log.md"
render "${PLANNING_TEMPLATES_DIR}/contract.md.tmpl" "${FEATURE_DIR}/contract.md"
render "${PLANNING_TEMPLATES_DIR}/spec_manifest.md.tmpl" "${FEATURE_DIR}/spec_manifest.md"
render "${PLANNING_TEMPLATES_DIR}/impact_map.md.tmpl" "${FEATURE_DIR}/impact_map.md"
if [[ "${CROSS_PLATFORM}" -eq 1 && "${AUTOMATION}" -eq 1 ]]; then
    render "${PLANNING_TEMPLATES_DIR}/ci_checkpoint_plan.md.tmpl" "${FEATURE_DIR}/ci_checkpoint_plan.md"
fi

render "${PLANNING_TEMPLATES_DIR}/slice_spec.v2.md.tmpl" "${FEATURE_DIR}/${SLICE_SPEC_REL}"

render "${PLANNING_TEMPLATES_DIR}/execution_preflight_report.md.tmpl" "${FEATURE_DIR}/execution_preflight_report.md"
render "${PLANNING_TEMPLATES_DIR}/slice_closeout_report.md.tmpl" "${FEATURE_DIR}/${SLICE_CLOSEOUT_REL}" "" "${SLICE_SPEC_REL}" "" "" "" "${SLICE_ID}"

render "${KICKOFF_TEMPLATES_DIR}/kickoff_exec_preflight.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/F0-exec-preflight.md" "F0-exec-preflight"
if [[ "${AUTOMATION}" -eq 1 ]]; then
    render "${KICKOFF_TEMPLATES_DIR}/kickoff_feature_cleanup.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/FZ-feature-cleanup.md" "FZ-feature-cleanup"
fi
if [[ "${CROSS_PLATFORM}" -eq 1 && "${AUTOMATION}" -eq 1 ]]; then
    render "${KICKOFF_TEMPLATES_DIR}/kickoff_ci_checkpoint.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/CP1-ci-checkpoint.md" "CP1-ci-checkpoint"
fi

FEATURE="${FEATURE}" FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="${SLICE_ID}" SLICE_ID_LOWER="${SLICE_ID_LOWER}" SLICE_SPEC_REL="${SLICE_SPEC_REL}" SLICE_CLOSEOUT_REL="${SLICE_CLOSEOUT_REL}" SLICE_KICKOFF_DIR_REL="${SLICE_KICKOFF_DIR_REL}" CROSS_PLATFORM="${CROSS_PLATFORM}" BEHAVIOR_PLATFORMS="${BEHAVIOR_PLATFORMS}" CI_PARITY_PLATFORMS="${CI_PARITY_PLATFORMS}" WSL_REQUIRED="${WSL_REQUIRED}" WSL_SEPARATE="${WSL_SEPARATE}" AUTOMATION="${AUTOMATION}" \
python3 - <<'PY'
import json
import os

feature = os.environ["FEATURE"]
feature_dir = os.environ["FEATURE_DIR"]
slice_id = os.environ["SLICE_ID"]
slice_id_lower = os.environ.get("SLICE_ID_LOWER", slice_id.lower())
slice_spec = os.environ.get("SLICE_SPEC_REL") or f"{slice_id}-spec.md"
slice_closeout = os.environ.get("SLICE_CLOSEOUT_REL") or f"{slice_id}-closeout_report.md"
slice_kickoff_dir_rel = os.environ.get("SLICE_KICKOFF_DIR_REL") or "kickoff_prompts"
cross_platform = os.environ["CROSS_PLATFORM"] == "1"
behavior_platforms_csv = os.environ.get("BEHAVIOR_PLATFORMS", "")
ci_parity_platforms_csv = os.environ.get("CI_PARITY_PLATFORMS", "")
wsl_required = os.environ["WSL_REQUIRED"] == "1"
wsl_separate = os.environ["WSL_SEPARATE"] == "1"
automation = os.environ.get("AUTOMATION", "0") == "1"
seed_ac_ids = [f"AC-{slice_id}-01", f"AC-{slice_id}-02", f"AC-{slice_id}-03"]

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


def kickoff_feature(task_id: str) -> str:
    return os.path.join(feature_dir, "kickoff_prompts", f"{task_id}.md")


def kickoff_slice(task_id: str) -> str:
    return os.path.join(feature_dir, slice_kickoff_dir_rel, f"{task_id}.md")


def refs(*extra: str) -> list:
    base = [
        os.path.join(feature_dir, "plan.md"),
        os.path.join(feature_dir, "spec_manifest.md"),
        os.path.join(feature_dir, "impact_map.md"),
        os.path.join(feature_dir, slice_spec),
    ]
    if cross_platform and automation:
        base.insert(3, os.path.join(feature_dir, "ci_checkpoint_plan.md"))
    return base + [os.path.join(feature_dir, x) for x in extra]


def _branch(suffix: str) -> str:
    return f"{feature}-{suffix}" if automation else suffix


def code_task(task_id: str, other_id: str) -> dict:
    task = {
        "id": task_id,
        "name": f"{slice_id} slice (code)",
        "type": "code",
        "phase": slice_id,
        "status": "pending",
        "description": f"Implement {slice_id} spec (production code only).",
        "references": refs(),
        "ac_ids": seed_ac_ids,
        "acceptance_criteria": [f"Implements the behaviors required by ac_ids (see {slice_spec})"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start-pair FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"{slice_id}\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-code wt/{feature}-{slice_id_lower}-code feat/{feature}"
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
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-code (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-{slice_id_lower}-code",
        "integration_task": f"{slice_id}-integ-core" if cross_platform else f"{slice_id}-integ",
        "kickoff_prompt": os.path.join(feature_dir, slice_kickoff_dir_rel, f"{task_id}.md"),
        "depends_on": ["F0-exec-preflight"],
        "concurrent_with": [other_id],
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-code")
        task["required_make_targets"] = ["triad-code-checks"]
    return task


def test_task(task_id: str, other_id: str) -> dict:
    task = {
        "id": task_id,
        "name": f"{slice_id} slice (test)",
        "type": "test",
        "phase": slice_id,
        "status": "pending",
        "description": f"Add/modify tests for {slice_id} spec (tests only).",
        "references": refs(),
        "ac_ids": seed_ac_ids,
        "acceptance_criteria": [f"Tests enforce the behaviors required by ac_ids (see {slice_spec})"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start-pair FEATURE_DIR=\"{feature_dir}\" SLICE_ID=\"{slice_id}\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-test wt/{feature}-{slice_id_lower}-test feat/{feature}"
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
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-test (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-{slice_id_lower}-test",
        "integration_task": f"{slice_id}-integ-core" if cross_platform else f"{slice_id}-integ",
        "kickoff_prompt": os.path.join(feature_dir, slice_kickoff_dir_rel, f"{task_id}.md"),
        "depends_on": ["F0-exec-preflight"],
        "concurrent_with": [other_id],
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-test")
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

    end_checklist = [
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
    ]
    end_checklist.extend(
        [
            f"If this slice ends a CI checkpoint group: run the checkpoint task (for example, CP1-ci-checkpoint) from the orchestration checkout per {os.path.join(feature_dir, 'ci_checkpoint_plan.md')}",
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{slice_id}-integ-core\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} {slice_id}-integ-core\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-integ-core (per plan.md)"
            ),
        ]
    )

    task = {
        "id": f"{slice_id}-integ-core",
        "name": f"{slice_id} slice (integration core)",
        "type": "integration",
        "phase": slice_id,
        "status": "pending",
        "description": f"Merge {slice_id} code+tests and make the slice green on the primary dev platform.",
        "references": refs(*smoke_refs),
        "acceptance_criteria": ["Core slice is green under make integ-checks and matches the spec"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"{slice_id}-integ-core\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-integ-core wt/{feature}-{slice_id_lower}-integ-core feat/{feature}"
            ),
        ],
        "end_checklist": end_checklist,
        "worktree": f"wt/{feature}-{slice_id_lower}-integ-core",
        "integration_task": f"{slice_id}-integ-core",
        "kickoff_prompt": kickoff_slice(f"{slice_id}-integ-core"),
        "depends_on": [f"{slice_id}-code", f"{slice_id}-test"],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-integ-core")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = False
    return task


def integ_platform_task(platform: str) -> dict:
    smoke_required = platform in behavior_platforms or platform == "wsl"
    if platform == "linux":
        smoke_refs = ("smoke/linux-smoke.sh",)
        name = f"{slice_id} slice (integration Linux)"
        desc = "Linux platform-fix integration task (may be a no-op if already green)."
        if wsl_required and not wsl_separate:
            dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform linux --run-wsl --workflow-ref "feat/{feature}" --cleanup'
        else:
            dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform linux --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "macos":
        smoke_refs = ("smoke/macos-smoke.sh",)
        name = f"{slice_id} slice (integration macOS)"
        desc = "macOS platform-fix integration task (may be a no-op if already green)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform macos --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "windows":
        smoke_refs = ("smoke/windows-smoke.ps1",)
        name = f"{slice_id} slice (integration Windows)"
        desc = "Windows platform-fix integration task (may be a no-op if already green)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform windows --workflow-ref "feat/{feature}" --cleanup'
    elif platform == "wsl":
        smoke_refs = ("smoke/linux-smoke.sh",)
        name = f"{slice_id} slice (integration WSL)"
        desc = "WSL platform-fix integration task (Linux-in-WSL)."
        dispatch = f'scripts/ci/dispatch_feature_smoke.sh --feature-dir "{feature_dir}" --runner-kind self-hosted --platform wsl --workflow-ref "feat/{feature}" --cleanup'
    else:
        raise SystemExit(f"unexpected platform: {platform}")

    if not smoke_required:
        smoke_refs = ()
        name = f"{slice_id} slice (integration CI parity: {platform})"
        desc = f"{platform} CI parity fix task (compile/test/lint only; no behavioral smoke required for this platform)."
        dispatch = f"make ci-compile-parity CI_WORKFLOW_REF=\"feat/{feature}\" CI_REMOTE=origin CI_CLEANUP=1"

    task_id = f"{slice_id}-integ-{platform}"
    task = {
        "id": task_id,
        "name": name,
        "type": "integration",
        "phase": slice_id,
        "status": "pending",
        "description": desc,
        "references": refs(*smoke_refs),
        "acceptance_criteria": [
            (f"{platform} smoke is green for this slice" if smoke_required else f"{platform} CI parity is green for this slice (no behavioral smoke required)")
        ],
        "start_checklist": [
            f"Run on {platform} host if possible",
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"{task_id}\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-integ-{platform} wt/{feature}-{slice_id_lower}-integ-{platform} feat/{feature}"
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
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-integ-{platform} (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-{slice_id_lower}-integ-{platform}",
        "integration_task": task_id,
        "kickoff_prompt": kickoff_slice(task_id),
        "depends_on": [f"{slice_id}-integ-core"],
        "concurrent_with": [],
        "platform": platform,
        "runner": "github-actions",
        "workflow": (".github/workflows/feature-smoke.yml" if smoke_required else ".github/workflows/ci-compile-parity.yml"),
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-integ-{platform}")
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

    end_checklist = [
        "Merge platform-fix branches (if any) + resolve conflicts",
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
    ]
    end_checklist.extend(
        [
            f"Confirm required CI checkpoint tasks that cover this slice are completed and recorded in {os.path.join(feature_dir, 'session_log.md')}",
            f"Complete slice closeout gate report: {os.path.join(feature_dir, slice_closeout)}",
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{slice_id}-integ\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} {slice_id}-integ\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-integ (per plan.md)"
            ),
        ]
    )

    task = {
        "id": f"{slice_id}-integ",
        "name": f"{slice_id} slice (integration final)",
        "type": "integration",
        "phase": slice_id,
        "status": "pending",
        "description": "Final integration: merge any platform fixes, complete slice closeout, and confirm checkpoint evidence is recorded.",
        "references": refs(*smoke_refs, slice_closeout),
        "ac_ids": seed_ac_ids,
        "acceptance_criteria": [f"Slice closeout report completed and local integration gates are green (implements behaviors required by ac_ids; see {slice_spec})"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"{slice_id}-integ\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-integ wt/{feature}-{slice_id_lower}-integ feat/{feature}"
            ),
        ],
        "end_checklist": end_checklist,
        "worktree": f"wt/{feature}-{slice_id_lower}-integ",
        "integration_task": f"{slice_id}-integ",
        "kickoff_prompt": kickoff_slice(f"{slice_id}-integ"),
        "depends_on": [f"{slice_id}-integ-core"] + [f"{slice_id}-integ-{p}" for p in platform_tasks],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-integ")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = True
    return task


def integ_single_task() -> dict:
    task = {
        "id": f"{slice_id}-integ",
        "name": f"{slice_id} slice (integration)",
        "type": "integration",
        "phase": slice_id,
        "status": "pending",
        "description": f"Integrate {slice_id} code+tests, reconcile to spec, and run integration gate.",
        "references": refs(slice_closeout),
        "ac_ids": seed_ac_ids,
        "acceptance_criteria": [f"Slice is green under make integ-checks and implements behaviors required by ac_ids (see {slice_spec})"],
        "start_checklist": [
            f"git checkout feat/{feature} && git pull --ff-only",
            f"Read plan.md, tasks.json, session_log.md, {slice_spec}, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            (
                f"Run: make triad-task-start FEATURE_DIR=\"{feature_dir}\" TASK_ID=\"{slice_id}-integ\""
                if automation
                else f"Run: git worktree add -b {slice_id_lower}-integ wt/{feature}-{slice_id_lower}-integ feat/{feature}"
            ),
        ],
        "end_checklist": [
            "cargo fmt",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "Run relevant tests",
            "make integ-checks",
            f"Complete slice closeout gate report: {os.path.join(feature_dir, slice_closeout)}",
            (
                f"From inside the worktree: make triad-task-finish TASK_ID=\"{slice_id}-integ\""
                if automation
                else f"From inside the worktree: git add -A && git commit -m \"integ: {feature} {slice_id}-integ\""
            ),
            (
                "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
                if automation
                else f"Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/{feature}-{slice_id_lower}-integ (per plan.md)"
            ),
        ],
        "worktree": f"wt/{feature}-{slice_id_lower}-integ",
        "integration_task": f"{slice_id}-integ",
        "kickoff_prompt": kickoff_slice(f"{slice_id}-integ"),
        "depends_on": [f"{slice_id}-code", f"{slice_id}-test"],
        "concurrent_with": [],
    }
    if automation:
        task["git_branch"] = _branch(f"{slice_id_lower}-integ")
        task["required_make_targets"] = ["integ-checks"]
        task["merge_to_orchestration"] = True
    return task


meta = {
    "schema_version": (4 if (automation and cross_platform) else (3 if automation else (2 if cross_platform else 1))),
    "feature": feature,
    "cross_platform": cross_platform,
    "execution_gates": True,
    "slice_spec_version": 2,
    # Optional registries / external dependency tracking (Planning Pack-level).
    "workstream_id": None,
    "work_item_refs": [],
    "depends_on": {"adrs": [], "work_items": [], "workstreams": [], "packs": []},
    "blocks": {"adrs": [], "work_items": [], "workstreams": [], "packs": []},
}
if automation:
    meta["automation"] = {"enabled": True, "orchestration_branch": f"feat/{feature}"}
if automation and cross_platform:
    # Schema v4 cross-platform packs require explicit boundary markers to avoid per-slice platform-fix task explosions.
    # Initial scaffold has a single slice, so it is necessarily a checkpoint boundary.
    meta["checkpoint_boundaries"] = [slice_id]

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
            os.path.join(feature_dir, "spec_manifest.md"),
            os.path.join(feature_dir, "impact_map.md"),
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
code_id = f"{slice_id}-code"
test_id = f"{slice_id}-test"
tasks.append(code_task(code_id, test_id))
tasks.append(test_task(test_id, code_id))

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
    if automation:
        tasks.append(
            {
                "id": "CP1-ci-checkpoint",
                "name": "CI checkpoint (initial scaffold)",
                "type": "ops",
                "phase": "CP1",
                "status": "pending",
                "description": "Run cross-platform CI gates at the checkpoint boundary defined in ci_checkpoint_plan.md.",
                "references": [
                    os.path.join(feature_dir, "ci_checkpoint_plan.md"),
                    os.path.join(feature_dir, "impact_map.md"),
                    os.path.join(feature_dir, "tasks.json"),
                    os.path.join(feature_dir, "session_log.md"),
                ],
                "acceptance_criteria": [
                    "Checkpoint CI gates executed or skipped per ci-audit, with evidence recorded in session_log.md",
                ],
                "start_checklist": [
                    f"Run: make triad-orch-ensure FEATURE_DIR=\"{feature_dir}\"",
                    "Read ci_checkpoint_plan.md and confirm which slice id this checkpoint validates",
                    "Set status to in_progress; add START entry; commit docs",
                ],
                "end_checklist": [
                    "Run compile parity + behavioral smoke per ci_checkpoint_plan.md (use ci-audit to skip redundant dispatch)",
                    "Record run ids/URLs and ci-audit output lines in session_log.md",
                    "Set status to completed; add END entry; commit docs",
                ],
                "worktree": None,
                "integration_task": None,
                "kickoff_prompt": os.path.join(feature_dir, "kickoff_prompts", "CP1-ci-checkpoint.md"),
                "depends_on": [f"{slice_id}-integ-core"],
                "concurrent_with": [],
            }
        )
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
            "description": "At feature end, remove retained worktrees and optionally prune task branches via make triad-feature-cleanup.",
            "references": [
                os.path.join(feature_dir, "plan.md"),
                os.path.join(feature_dir, "tasks.json"),
                os.path.join(feature_dir, "session_log.md"),
                "make triad-feature-cleanup",
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
            "depends_on": [f"{slice_id}-integ"],
            "concurrent_with": [],
        }
    )

data = {"meta": meta, "tasks": tasks}
with open(tasks_path, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2)
    handle.write("\n")
PY

: <<'LEGACY_TASKS_JSON'
Legacy tasks.json generator removed; tasks.json is produced by the Python generator above.
LEGACY_TASKS_JSON

if [[ "${AUTOMATION}" -eq 1 ]]; then
    SLICE_CODE_BRANCH="${FEATURE}-${SLICE_ID_LOWER}-code"
    SLICE_TEST_BRANCH="${FEATURE}-${SLICE_ID_LOWER}-test"
    SLICE_INTEG_BRANCH="${FEATURE}-${SLICE_ID_LOWER}-integ"
    SLICE_INTEG_CORE_BRANCH="${FEATURE}-${SLICE_ID_LOWER}-integ-core"
else
    SLICE_CODE_BRANCH="${SLICE_ID_LOWER}-code"
    SLICE_TEST_BRANCH="${SLICE_ID_LOWER}-test"
    SLICE_INTEG_BRANCH="${SLICE_ID_LOWER}-integ"
    SLICE_INTEG_CORE_BRANCH="${SLICE_ID_LOWER}-integ-core"
fi

render "${KICKOFF_TEMPLATES_DIR}/kickoff_code.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-code.md" "${SLICE_ID}-code" "${SLICE_SPEC_REL}" "${SLICE_CODE_BRANCH}" "wt/${FEATURE}-${SLICE_ID_LOWER}-code" "" "${SLICE_ID}"
render "${KICKOFF_TEMPLATES_DIR}/kickoff_test.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-test.md" "${SLICE_ID}-test" "${SLICE_SPEC_REL}" "${SLICE_TEST_BRANCH}" "wt/${FEATURE}-${SLICE_ID_LOWER}-test" "" "${SLICE_ID}"
if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    render "${KICKOFF_TEMPLATES_DIR}/kickoff_integ_core.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-integ-core.md" "${SLICE_ID}-integ-core" "${SLICE_SPEC_REL}" "${SLICE_INTEG_CORE_BRANCH}" "wt/${FEATURE}-${SLICE_ID_LOWER}-integ-core" "" "${SLICE_ID}"
    IFS=',' read -r -a ci_platforms <<<"${CI_PARITY_PLATFORMS}"
    for p in "${ci_platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        render "${KICKOFF_TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-integ-${p}.md" "${SLICE_ID}-integ-${p}" "${SLICE_SPEC_REL}" "${FEATURE}-${SLICE_ID_LOWER}-integ-${p}" "wt/${FEATURE}-${SLICE_ID_LOWER}-integ-${p}" "${p}" "${SLICE_ID}"
    done
    if [[ "${WSL_REQUIRED}" -eq 1 && "${WSL_SEPARATE}" -eq 1 ]]; then
        render "${KICKOFF_TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-integ-wsl.md" "${SLICE_ID}-integ-wsl" "${SLICE_SPEC_REL}" "${FEATURE}-${SLICE_ID_LOWER}-integ-wsl" "wt/${FEATURE}-${SLICE_ID_LOWER}-integ-wsl" "wsl" "${SLICE_ID}"
    fi
    render "${KICKOFF_TEMPLATES_DIR}/kickoff_integ_final.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-integ.md" "${SLICE_ID}-integ" "${SLICE_SPEC_REL}" "${SLICE_INTEG_BRANCH}" "wt/${FEATURE}-${SLICE_ID_LOWER}-integ" "" "${SLICE_ID}"
else
    render "${KICKOFF_TEMPLATES_DIR}/kickoff_integ.md.tmpl" "${FEATURE_DIR}/${SLICE_KICKOFF_DIR_REL}/${SLICE_ID}-integ.md" "${SLICE_ID}-integ" "${SLICE_SPEC_REL}" "${SLICE_INTEG_BRANCH}" "wt/${FEATURE}-${SLICE_ID_LOWER}-integ" "" "${SLICE_ID}"
fi

if [[ "${DECISION_HEAVY}" -eq 1 || "${CROSS_PLATFORM}" -eq 1 ]]; then
	    cat >"${FEATURE_DIR}/decision_register.md" <<'MD'
# Decision Register

Use the template in:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
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
