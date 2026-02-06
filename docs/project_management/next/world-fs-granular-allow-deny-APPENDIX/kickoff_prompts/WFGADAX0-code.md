# Kickoff: WFGADAX0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-appendix-wfgadax0-code` on branch `world-fs-granular-allow-deny-appendix-wfgadax0-code`.
2. Verify `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `WFGADAX0-spec.md`, this prompt.

## Requirements
- Implement behaviors and hard-error rules in the authoritative spec pack.
- Required commands (record outputs):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - Baseline compile check (before and after changes): `cargo test -p substrate-broker --tests --no-run`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGADAX0-code"` (runs `triad-code-checks`).
