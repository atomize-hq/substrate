#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/planning/new_feature.sh --feature <feature_dir_name> [--decision-heavy] [--cross-platform]

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

FEATURE_DIR="docs/project_management/next/${FEATURE}"
TEMPLATES_DIR="docs/project_management/standards/templates"

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

    sed \
        -e "s|{{FEATURE}}|${FEATURE}|g" \
        -e "s|{{FEATURE_DIR}}|${FEATURE_DIR}|g" \
        -e "s|{{NOW_UTC}}|${NOW_UTC}|g" \
        -e "s|{{TASK_ID}}|${task_id}|g" \
        -e "s|{{SPEC_FILE}}|${spec_file}|g" \
        -e "s|{{BRANCH}}|${branch}|g" \
        -e "s|{{WORKTREE}}|${worktree}|g" \
        -e "s|{{PLATFORM}}|${platform}|g" \
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

if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    cat >"${FEATURE_DIR}/tasks.json" <<JSON
{
  "meta": {
    "schema_version": 2,
    "feature": "${FEATURE}",
    "cross_platform": true,
    "platforms_required": ["linux", "macos", "windows"]
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
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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
        "Dispatch cross-platform smoke via scripts/ci/dispatch_feature_smoke.sh (record run ids/URLs)",
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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
        "Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform linux",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure Linux smoke is green; record run id/URL",
        "Commit worktree changes (if any); merge back ff-only; update docs; remove worktree"
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
        "Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform macos",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure macOS smoke is green; record run id/URL",
        "Commit worktree changes (if any); merge back ff-only; update docs; remove worktree"
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
        "Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform windows",
        "If needed: fix + fmt/clippy + targeted tests",
        "Ensure Windows smoke is green; record run id/URL",
        "Commit worktree changes (if any); merge back ff-only; update docs; remove worktree"
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
        "Dispatch cross-platform smoke via scripts/ci/dispatch_feature_smoke.sh (record run ids/URLs)",
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
      ],
      "worktree": "wt/${FEATURE}-c0-integ",
      "integration_task": "C0-integ",
      "kickoff_prompt": "${FEATURE_DIR}/kickoff_prompts/C0-integ.md",
      "depends_on": ["C0-integ-core", "C0-integ-linux", "C0-integ-macos", "C0-integ-windows"],
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
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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
        "Commit worktree changes; merge back ff-only; update docs; remove worktree"
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

render "${TEMPLATES_DIR}/kickoff_code.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-code.md" "C0-code" "C0-spec.md" "c0-code" "wt/${FEATURE}-c0-code"
render "${TEMPLATES_DIR}/kickoff_test.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-test.md" "C0-test" "C0-spec.md" "c0-test" "wt/${FEATURE}-c0-test"
if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    render "${TEMPLATES_DIR}/kickoff_integ_core.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-core.md" "C0-integ-core" "C0-spec.md" "c0-integ-core" "wt/${FEATURE}-c0-integ-core"
    render "${TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-linux.md" "C0-integ-linux" "C0-spec.md" "c0-integ-linux" "wt/${FEATURE}-c0-integ-linux" "linux"
    render "${TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-macos.md" "C0-integ-macos" "C0-spec.md" "c0-integ-macos" "wt/${FEATURE}-c0-integ-macos" "macos"
    render "${TEMPLATES_DIR}/kickoff_integ_platform.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ-windows.md" "C0-integ-windows" "C0-spec.md" "c0-integ-windows" "wt/${FEATURE}-c0-integ-windows" "windows"
    render "${TEMPLATES_DIR}/kickoff_integ_final.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ.md" "C0-integ" "C0-spec.md" "c0-integ" "wt/${FEATURE}-c0-integ"
else
    render "${TEMPLATES_DIR}/kickoff_integ.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ.md" "C0-integ" "C0-spec.md" "c0-integ" "wt/${FEATURE}-c0-integ"
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

    cat >"${FEATURE_DIR}/manual_testing_playbook.md" <<'MD'
# Manual Testing Playbook

This playbook must contain runnable commands and expected exit codes/output.
MD
fi

if [[ "${CROSS_PLATFORM}" -eq 1 ]]; then
    mkdir -p "${FEATURE_DIR}/smoke"
    cat >"${FEATURE_DIR}/smoke/linux-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "Smoke script scaffold (linux) - replace with feature checks"
exit 1
SH
    cat >"${FEATURE_DIR}/smoke/macos-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "Smoke script scaffold (macos) - replace with feature checks"
exit 1
SH
    cat >"${FEATURE_DIR}/smoke/windows-smoke.ps1" <<'PS1'
param()
$ErrorActionPreference = "Stop"
Write-Host "Smoke script scaffold (windows) - replace with feature checks"
exit 1
PS1
fi

echo "OK: created ${FEATURE_DIR}"
