# Kickoff: AHCSITC1-test (test)

## Scope
- Tests only; no production code changes.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC1/AHCSITC1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc1-test` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc1-test` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC1-spec.md`, and this prompt.
3. If the worktree metadata is missing or mismatched, stop and ask the operator to rerun `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" SLICE_ID="AHCSITC1"`.

## Requirements
- Add or adjust tests that enforce `AC-AHCSITC1-01` through `AC-AHCSITC1-06`.
- Keep the test scope reviewable and limited to the AHCSITC1 session protocol and placement surface.
- Run `cargo fmt` and the targeted tests you add or touch.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC1-test"`.
3. Hand off the targeted test commands and outcomes to the operator.
