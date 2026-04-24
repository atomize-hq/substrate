# Kickoff: AHCSITC2-integ (integration final)

## Scope
- Merge the AHCSITC2 core-integration and platform-fix branches into one final checkpoint-boundary slice result.
- Spec: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ` on branch `agent-hub-core-successor-identity-tuple-compatible-ahcsitc2-integ` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `AHCSITC2-spec.md`, and this prompt.
3. If the worktree metadata is missing or mismatched, stop and ask the operator to rerun `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" TASK_ID="AHCSITC2-integ"`.

## Requirements
- Merge `AHCSITC2-integ-core` and any non-empty platform-fix branches for Linux, macOS, and Windows.
- Reconcile code and tests to the spec. The spec wins if drift appears.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Do not rerun checkpoint CI from this task unless the operator explicitly asks for a fresh checkpoint pass.

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="AHCSITC2-integ"`.
3. Hand off the merged parity status and any residual follow-ups to the operator.
