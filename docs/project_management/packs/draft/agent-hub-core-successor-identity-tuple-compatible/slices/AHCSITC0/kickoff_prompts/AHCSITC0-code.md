# Kickoff: AHCSITC0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC0/AHCSITC0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc0-code` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc0-code` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC0-spec.md`, and this prompt.
3. If the worktree metadata is missing or mismatched, stop and ask the operator to rerun `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" SLICE_ID="AHCSITC0"`.

## Requirements
- Implement exactly the behaviors required by `AC-AHCSITC0-01` through `AC-AHCSITC0-06`.
- Keep the slice boundary on the command surface and identity projection surfaces owned by `AHCSITC0`.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Run a targeted baseline test set before changes, then rerun the same tests after changes.
- Do not add new tests or new test files.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC0-code"`.
3. Hand off the baseline test commands and outcomes to the operator.
