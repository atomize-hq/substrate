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

    sed \
        -e "s|{{FEATURE}}|${FEATURE}|g" \
        -e "s|{{FEATURE_DIR}}|${FEATURE_DIR}|g" \
        -e "s|{{NOW_UTC}}|${NOW_UTC}|g" \
        -e "s|{{TASK_ID}}|${task_id}|g" \
        -e "s|{{SPEC_FILE}}|${spec_file}|g" \
        -e "s|{{BRANCH}}|${branch}|g" \
        -e "s|{{WORKTREE}}|${worktree}|g" \
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

cat >"${FEATURE_DIR}/tasks.json" <<JSON
{
  "meta": {
    "feature": "${FEATURE}",
    "cross_platform": $( [[ "${CROSS_PLATFORM}" -eq 1 ]] && echo "true" || echo "false" )
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

render "${TEMPLATES_DIR}/kickoff_code.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-code.md" "C0-code" "C0-spec.md" "c0-code" "wt/${FEATURE}-c0-code"
render "${TEMPLATES_DIR}/kickoff_test.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-test.md" "C0-test" "C0-spec.md" "c0-test" "wt/${FEATURE}-c0-test"
render "${TEMPLATES_DIR}/kickoff_integ.md.tmpl" "${FEATURE_DIR}/kickoff_prompts/C0-integ.md" "C0-integ" "C0-spec.md" "c0-integ" "wt/${FEATURE}-c0-integ"

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
