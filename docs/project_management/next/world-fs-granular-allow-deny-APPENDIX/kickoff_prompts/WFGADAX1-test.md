# Kickoff: WFGADAX1-test (test)

## Scope
- Tests only; no production refactors.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-appendix-wfgadax1-test` on branch `world-fs-granular-allow-deny-appendix-wfgadax1-test`.
2. Verify `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `WFGADAX1-spec.md`, this prompt.

## Required commands (record outputs)
- `cargo fmt`
- `cargo test --workspace --tests --no-run`
- `cargo test -p shell wfgadax1 -- --nocapture`

## End Checklist
1. Run the required commands above.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGADAX1-test"` (runs `triad-test-checks`).
