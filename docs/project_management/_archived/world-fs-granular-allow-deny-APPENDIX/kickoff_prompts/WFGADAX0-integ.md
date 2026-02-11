# Kickoff: WFGADAX0-integ (integration)

## Scope
- Merge code + tests and validate schema behavior.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-appendix-wfgadax0-integ` on branch `world-fs-granular-allow-deny-appendix-wfgadax0-integ`.
2. Verify `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `WFGADAX0-spec.md`, this prompt.

## Required commands (record outputs)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `bash docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh`
- `make integ-checks`

## End Checklist
1. Run the required commands above and record outputs in `session_log.md`.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGADAX0-integ"` (runs `integ-checks`).
