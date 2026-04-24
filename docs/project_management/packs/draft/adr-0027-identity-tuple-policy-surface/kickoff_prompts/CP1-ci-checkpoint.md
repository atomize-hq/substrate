# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Validate the single checkpoint boundary after `ITPS3-integ-core`.
- Run from the orchestration checkout, not a task worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Read `plan.md`, `tasks.json`, `session_log.md`, and `pre-planning/ci_checkpoint_plan.md`.
2. Confirm the checkpoint boundary slice is `ITPS3`.
3. Confirm this task waits for `ITPS3-integ-core`.

## Requirements
- Use the checkpoint plan as the source of truth for compile-parity and behavioral-smoke dispatch.
- Record run ids, URLs, or ci-audit skip evidence in `session_log.md`.
- Do not change slice ordering, checkpoint boundaries, or automation posture from this task.

## End Checklist
1. Mark `CP1-ci-checkpoint` completed in `tasks.json`.
2. Add the checkpoint evidence to `session_log.md`.
3. Keep follow-on platform-fix work scoped to `ITPS3`.
