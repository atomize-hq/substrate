# Kickoff: DIWAS0-integ (integration)

## Scope
- Merge DIWAS0 code+test triads and run the slice integration gates.
- Spec: `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/DIWAS0-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-service-staging-diwas0-integ` on branch `dev-install-world-service-staging-diwas0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Confirm `DIWAS0-code` and `DIWAS0-test` are complete and available to merge.

## Required commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p shell --test world_enable -- --nocapture`
- `make integ-checks`

## End Checklist
1. Merge `DIWAS0-code` and `DIWAS0-test`.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS0-integ"`.

