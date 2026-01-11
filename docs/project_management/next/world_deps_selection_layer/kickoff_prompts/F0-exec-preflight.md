# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/next/world_deps_selection_layer/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/world_deps_selection_layer"`
2. Read: ADR + Executive Summary, `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, all WDL specs, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `docs/project_management/next/world_deps_selection_layer/execution_preflight_report.md` with a concrete recommendation:
- ACCEPT: triads may begin.
- REVISE: do not start triads until the listed issues are fixed and the preflight is re-run.

Minimum checks (must be recorded in the report):
- `docs/project_management/next/world_deps_selection_layer/tasks.json` meta is explicit and correct:
  - `schema_version=3`, `behavior_platforms_required=["linux","macos","windows"]`, `ci_parity_platforms_required=["linux","macos","windows"]`
  - `automation.enabled=true`, `automation.orchestration_branch="feat/world_deps_selection_layer"`
- Smoke scripts are not “toy” checks and match the automation requirements in:
  - `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md` → “Automation hooks (required)”
- CI dispatch commands embedded in integration tasks are runnable:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1`
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist

1. Set `F0-exec-preflight` status to `completed` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add END entry to `docs/project_management/next/world_deps_selection_layer/session_log.md` (include the recommendation and any required fixes).
2. Commit docs (`docs: finish F0-exec-preflight`).

