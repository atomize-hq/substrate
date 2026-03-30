# Kickoff: DIWAS0-test (test)

## Scope
- Tests only (add/update tests to lock the DIWAS0 contract).
- Spec: `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS0/DIWAS0-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-agent-staging-diwas0-test` on branch `dev-install-world-agent-staging-diwas0-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `contract.md`, the slice spec, and this prompt.
3. If the worktree is missing or mismatched, stop and ask the operator to start the triads:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging" SLICE_ID="DIWAS0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging" TASK_ID="DIWAS0-test"`

## Requirements
- Add or update tests that prove:
  - accepted staged paths are checked in the correct order,
  - exit code and remediation content are deterministic when both paths are missing,
  - `--dry-run` performs no writes and still enforces the preflight.

## Validation (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p shell --test world_enable -- --nocapture`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS0-test"`.

