# Kickoff: WFGADAX0-test (test)

## Scope
- Tests only; no production refactors.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-appendix-wfgadax0-test` on branch `world-fs-granular-allow-deny-appendix-wfgadax0-test`.
2. Verify `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `WFGADAX0-spec.md`, this prompt.

## Requirements
- Add tests for deterministic schema validation and hard errors.
- Required commands (record outputs):
  - `cargo fmt`
  - `cargo test -p substrate-broker --tests`
  - `cargo test -p substrate-broker wfgadax0 -- --nocapture`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGADAX0-test"` (runs `triad-test-checks`).
