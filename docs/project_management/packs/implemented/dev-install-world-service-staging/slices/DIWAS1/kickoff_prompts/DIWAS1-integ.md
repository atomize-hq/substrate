# Kickoff: DIWAS1-integ (integration final)

## Scope
- Finalize the boundary slice after `CP1-ci-checkpoint` and any platform follow-up tasks.
- Spec: `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/DIWAS1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-service-staging-diwas1-integ` on branch `dev-install-world-service-staging-diwas1-integ` and that `.taskmeta.json` exists at the worktree root.
2. Confirm the checkpoint evidence is complete and recorded in `session_log.md`.
3. Confirm any platform follow-up tasks are complete or explicitly no-op:
   - `DIWAS1-integ-linux`
   - `DIWAS1-integ-macos`
   - `DIWAS1-integ-windows`

## Required commands
- Merge any platform-fix branches that produced commits.
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh`
- `make integ-checks`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-integ"`.
2. Ensure the orchestration branch is updated with the final boundary slice state.

