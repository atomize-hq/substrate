# Kickoff: WFGADAX3-integ-windows (integration)

## Scope
- Windows platform-fix integration task for checkpoint boundary slice.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX3-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-appendix-wfgadax3-integ-windows` on branch `world-fs-granular-allow-deny-appendix-wfgadax3-integ-windows`.
2. Verify `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `WFGADAX3-spec.md`, this prompt.

## Required commands (record outputs)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make integ-checks`

## End Checklist
1. Run the required commands above.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGADAX3-integ-windows"` (runs `integ-checks`).
