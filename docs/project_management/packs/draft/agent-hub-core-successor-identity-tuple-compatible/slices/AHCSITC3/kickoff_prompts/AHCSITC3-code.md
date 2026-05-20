# Kickoff: AHCSITC3-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC3/AHCSITC3-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc3-code` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc3-code` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC3-spec.md`, and this prompt.
3. If the worktree metadata is missing or mismatched, stop and ask the operator to rerun `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" SLICE_ID="AHCSITC3"`.

## Requirements
- Implement exactly the behaviors required by `AC-AHCSITC3-01` through `AC-AHCSITC3-06`.
- Keep the slice boundary on parity, compatibility, and validation surfaces owned by `AHCSITC3`.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Run a targeted baseline test set before changes, then rerun the same tests after changes.
- Do not add new tests or new test files.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC3-code"`.
3. Hand off the baseline test commands and outcomes to the operator.
