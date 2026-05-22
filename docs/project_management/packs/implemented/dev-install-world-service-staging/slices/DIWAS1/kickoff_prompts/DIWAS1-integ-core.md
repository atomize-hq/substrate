# Kickoff: DIWAS1-integ-core (integration core)

## Scope
- Merge DIWAS1 code+test triads on the primary behavior platform (Linux) before the checkpoint runs.
- Spec: `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/DIWAS1-spec.md`
- Checkpoint plan: `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-service-staging-diwas1-integ-core` on branch `dev-install-world-service-staging-diwas1-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Confirm `DIWAS1-code` and `DIWAS1-test` are complete and available to merge.

## Required commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash tests/installers/install_smoke.sh`
- `bash docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh`
- `make integ-checks`

## End Checklist
1. Merge `DIWAS1-code` and `DIWAS1-test`.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-integ-core"`.
3. Hand off the commit SHA for `CP1-ci-checkpoint`.

