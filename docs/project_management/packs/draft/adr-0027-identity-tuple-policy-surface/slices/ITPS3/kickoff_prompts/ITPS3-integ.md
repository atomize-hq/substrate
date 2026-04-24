# Kickoff: ITPS3-integ (integration final)

## Scope
- Merge the ITPS3 platform-fix branches, reconcile to spec, and close the boundary slice after checkpoint CI.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps3-integ` on branch `adr-0027-identity-tuple-policy-surface-itps3-integ`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, `pre-planning/ci_checkpoint_plan.md`, and this prompt.
3. Confirm `ITPS3-integ-core`, `ITPS3-integ-linux`, `ITPS3-integ-macos`, and `ITPS3-integ-windows` are complete before proceeding.

## Requirements
- Merge any platform-fix branches into the final aggregator worktree.
- Ensure the final merged state satisfies `AC-ITPS3-01` through `AC-ITPS3-07`.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.

## End Checklist
1. Confirm `CP1-ci-checkpoint` evidence is already recorded in `session_log.md`.
2. Run `make triad-task-finish TASK_ID="ITPS3-integ"` inside the worktree.
3. Hand off the merged result for feature cleanup.
