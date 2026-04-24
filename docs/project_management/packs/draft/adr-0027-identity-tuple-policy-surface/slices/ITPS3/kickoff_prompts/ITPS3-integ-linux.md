# Kickoff: ITPS3-integ-linux (integration platform-fix — linux)

## Scope
- Validate and fix Linux behavior or parity issues after the `CP1-ci-checkpoint` result is known.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps3-integ-linux` on branch `adr-0027-identity-tuple-policy-surface-itps3-integ-linux`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, `pre-planning/ci_checkpoint_plan.md`, and this prompt.
3. Confirm both `ITPS3-integ-core` and `CP1-ci-checkpoint` are complete before proceeding.

## Requirements
- Merge the `ITPS3-integ-core` branch into this worktree before validating Linux.
- Keep fixes scoped to Linux behavior or parity.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted tests as needed, and Linux smoke for `ITPS3`.

## End Checklist
1. Record the Linux smoke run id/URL or equivalent evidence.
2. Run `make triad-task-finish TASK_ID="ITPS3-integ-linux"` inside the worktree.
3. Hand off Linux-specific notes to the final aggregator task.
