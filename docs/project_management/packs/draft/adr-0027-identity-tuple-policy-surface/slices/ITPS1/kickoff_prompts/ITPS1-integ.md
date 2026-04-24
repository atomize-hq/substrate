# Kickoff: ITPS1-integ (integration)

## Scope
- Merge `ITPS1-code` and `ITPS1-test`, reconcile to spec, and make the slice green.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps1-integ` on branch `adr-0027-identity-tuple-policy-surface-itps1-integ`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. Stop if the worktree is missing `.taskmeta.json`.

## Requirements
- Merge the code and test branches for ITPS1.
- Ensure the integrated state satisfies `AC-ITPS1-01` through `AC-ITPS1-08`.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.

## End Checklist
1. Run `make triad-task-finish TASK_ID="ITPS1-integ"` inside the worktree.
2. Hand off the green commands and outcomes.
3. Do not edit planning docs inside the worktree.
