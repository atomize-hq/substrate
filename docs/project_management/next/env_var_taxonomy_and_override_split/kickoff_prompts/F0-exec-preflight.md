# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/next/env_var_taxonomy_and_override_split/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split"`
2. Read: ADR + Executive Summary, `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, `integration_map.md`, `manual_testing_playbook.md`, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `tasks.json`; add START entry to `session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `execution_preflight_report.md` with a concrete recommendation:
- **ACCEPT**: triads may begin.
- **REVISE**: do not start triads until the listed issues are fixed and the preflight is re-run.

At minimum, verify:
- The cross-platform plan is explicit and matches the spec/contract (platforms + WSL mode if needed).
- Smoke scripts mirror the manual testing playbook by running real commands/workflows and validating exit codes + key output.
- Any CI dispatch commands embedded in integration tasks are runnable with the expected runners.

## End Checklist

1. Set `F0-exec-preflight` status to `completed` in `tasks.json`; add END entry to `session_log.md` (include the recommendation and any required fixes).
2. Commit docs (`docs: finish F0-exec-preflight`).

