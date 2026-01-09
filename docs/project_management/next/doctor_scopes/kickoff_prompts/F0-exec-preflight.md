# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level execution preflight gate before any DS0 triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/next/doctor_scopes/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/doctor_scopes"`
2. Read: ADR + Executive Summary, `docs/project_management/next/doctor_scopes/plan.md`, `docs/project_management/next/doctor_scopes/tasks.json`, `docs/project_management/next/doctor_scopes/session_log.md`, `docs/project_management/next/doctor_scopes/DS0-spec.md`, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `docs/project_management/next/doctor_scopes/tasks.json`; add a START entry to `docs/project_management/next/doctor_scopes/session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `docs/project_management/next/doctor_scopes/execution_preflight_report.md` with a concrete recommendation:
- **ACCEPT**: triads may begin (after the planning quality gate is also ACCEPT).
- **REVISE**: do not start triads until the listed issues are fixed and preflight is re-run.

At minimum, verify:
- `tasks.json` platform declarations are correct and match the spec.
- Smoke scripts are runnable and mirror `manual_testing_playbook.md`.
- CI dispatch commands embedded in integration tasks are correct and the `feat/doctor-scopes` ref exists on the remote.

## End Checklist

1. Update `docs/project_management/next/doctor_scopes/execution_preflight_report.md` and set `RECOMMENDATION: ACCEPT` or `RECOMMENDATION: REVISE`.
2. Set `F0-exec-preflight` status to `completed` in `docs/project_management/next/doctor_scopes/tasks.json`; add an END entry to `docs/project_management/next/doctor_scopes/session_log.md` (include the recommendation + any required fixes).
3. Commit docs (`docs: finish F0-exec-preflight`).

