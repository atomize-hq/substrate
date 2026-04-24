# Kickoff: ITPS3-integ-windows (integration platform-fix — windows)

## Scope
- Validate and fix Windows compile-parity issues once checkpoint evidence identifies Windows as still needing fixes.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps3-integ-windows` on branch `adr-0027-identity-tuple-policy-surface-itps3-integ-windows`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, `pre-planning/ci_checkpoint_plan.md`, and this prompt.
3. Confirm `ITPS3-integ-core` is complete and checkpoint evidence shows Windows still needs fixes; `CP1-ci-checkpoint` may still be `in_progress` while platform-fix work is landing.

## Requirements
- Merge the `ITPS3-integ-core` branch into this worktree before validating Windows.
- Keep fixes scoped to Windows compile parity.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and targeted tests as needed for Windows parity.

## End Checklist
1. Record the Windows compile-parity run id/URL or equivalent evidence.
2. Run `make triad-task-finish TASK_ID="ITPS3-integ-windows"` inside the worktree.
3. Hand off Windows-specific notes to the final aggregator task.
