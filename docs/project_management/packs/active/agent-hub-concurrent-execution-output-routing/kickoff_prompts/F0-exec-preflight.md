# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing"`
2. Read: ADR + Executive Summary, `plan.md`, `tasks.json`, `session_log.md`, specs, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `tasks.json`; add START entry to `session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `execution_preflight_report.md` with a concrete recommendation:
- ACCEPT: triads may begin.
- REVISE: do not start triads until the listed issues are fixed and the preflight is re-run.

Verify at minimum:
- `tasks.json` meta: schema v4, cross_platform=true, execution_gates=true, automation.enabled=true, platforms lists correct.
- `ci_checkpoint_plan.md` machine-readable JSON is valid and boundaries match:
  - `tasks.json` `meta.checkpoint_boundaries == ["OR1"]`
  - checkpoint task exists: `CP1-ci-checkpoint` depends on `OR1-integ-core`
  - boundary-only platform-fix tasks exist only for OR1 (`OR1-integ-core`, `OR1-integ-linux`, `OR1-integ-macos`, `OR1-integ-windows`, `OR1-integ`)
- Manual playbook and smoke scripts mirror the same assertions.
- Kickoff prompts exist for every task id and include the exact rule line above.

## End Checklist

1. Update `execution_preflight_report.md`.
2. Set `F0-exec-preflight` status to `completed` in `tasks.json`; add END entry to `session_log.md`; commit docs (`docs: finish F0-exec-preflight`).
