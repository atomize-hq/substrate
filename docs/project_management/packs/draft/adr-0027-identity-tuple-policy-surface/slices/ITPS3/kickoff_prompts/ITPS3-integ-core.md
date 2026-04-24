# Kickoff: ITPS3-integ-core (integration core)

## Scope
- Merge `ITPS3-code` and `ITPS3-test`, reconcile to spec, and produce the core integration branch for checkpoint CI.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps3-integ-core` on branch `adr-0027-identity-tuple-policy-surface-itps3-integ-core`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, `pre-planning/ci_checkpoint_plan.md`, and this prompt.
3. Stop if the worktree is missing `.taskmeta.json`.

## Requirements
- Merge the code and test branches for ITPS3.
- Make the core branch green under `make integ-checks`.
- Do not dispatch cross-platform CI from this task; hand off to `CP1-ci-checkpoint` after the core branch is finished.

## End Checklist
1. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
2. Run `make triad-task-finish TASK_ID="ITPS3-integ-core"` inside the worktree.
3. Hand off the core-branch SHA and green commands for the checkpoint task.
