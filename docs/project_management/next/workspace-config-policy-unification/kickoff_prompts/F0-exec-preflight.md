# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/next/workspace-config-policy-unification/execution_preflight_report.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification"`
2. Read end-to-end:
   - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
   - `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
   - `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
   - `docs/project_management/next/workspace-config-policy-unification/plan.md`
   - `docs/project_management/next/workspace-config-policy-unification/tasks.json`
   - `docs/project_management/next/workspace-config-policy-unification/session_log.md`
   - `docs/project_management/next/workspace-config-policy-unification/integration_map.md`
   - `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md`
3. Set `F0-exec-preflight` status to `in_progress` in `tasks.json`; add START entry to `session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements
- Confirm Phase A/B gates (ADR-0012) are explicitly owned by slice acceptance criteria and validation artifacts.
- Confirm smoke scripts mirror the manual playbook and contain real contract assertions (not “command ran” checks).
- Confirm integration tasks reference required smoke scripts and closeout reports.

## End Checklist
1. Fill `execution_preflight_report.md` with a concrete recommendation (ACCEPT or REVISE) and any required fixes.
2. Set `F0-exec-preflight` status to `completed` in `tasks.json`; add END entry to `session_log.md`; commit docs (`docs: finish F0-exec-preflight`).
