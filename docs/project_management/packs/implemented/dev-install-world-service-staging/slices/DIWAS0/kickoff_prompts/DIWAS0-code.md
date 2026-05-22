# Kickoff: DIWAS0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/DIWAS0-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-service-staging-diwas0-code` on branch `dev-install-world-service-staging-diwas0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `contract.md`, the slice spec, and this prompt.
3. If the worktree is missing or mismatched, stop and ask the operator to start the triads:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-service-staging" SLICE_ID="DIWAS0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-service-staging" TASK_ID="DIWAS0-code"`

## Requirements
- Implement only the DIWAS0 behavior delta for the standard version-dir flow (`SUBSTRATE_WORLD_ENABLE_SCRIPT` unset).
- Keep `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior unchanged.
- Ensure the missing-artifact preflight happens before helper launch and before any privileged/provisioning/systemd work.

## Validation (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p shell --test world_enable -- --nocapture`

## End Checklist
1. Capture a short before/after note for the manual playbook (Cases 3 and 4).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS0-code"`.

