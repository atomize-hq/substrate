# Kickoff: DIWAS1-test (test)

## Scope
- Tests only (add/update tests to lock the DIWAS1 contract and installer regression coverage).
- Spec: `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/DIWAS1-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-service-staging-diwas1-test` on branch `dev-install-world-service-staging-diwas1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `contract.md`, the slice spec, and this prompt.

## Requirements
- Add coverage that proves:
  - staging behavior for debug and release,
  - refresh rule (`ln -sfn`) for profile switching,
  - no provisioning/systemd mutation when `--no-world` is set.
- Keep changes scoped to DIWAS1 surfaces.

## Validation (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash tests/installers/install_smoke.sh`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-test"`.

