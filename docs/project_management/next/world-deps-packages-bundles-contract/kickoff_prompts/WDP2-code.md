# Kickoff: WDP2-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP2-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp2-code` on branch `world-deps-packages-bundles-contract-wdp2-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-deps-packages-bundles-contract/plan.md`, `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json`, `docs/project_management/next/world-deps-packages-bundles-contract/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract" SLICE_ID="WDP2"`

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- Keep changes scoped to this slice; do not introduce install/sync mutation behavior in WDP2.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP2-code"`
3. Hand off the baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree.

