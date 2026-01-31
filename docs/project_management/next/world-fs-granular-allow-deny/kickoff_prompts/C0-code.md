# Kickoff: C0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny/C0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-c0-code` on branch `world-fs-granular-allow-deny-c0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-fs-granular-allow-deny/plan.md`, `docs/project_management/next/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/next/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny" SLICE_ID="C0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny" TASK_ID="C0-code"`

## Requirements
- Implement exactly the behaviors and error handling in the authoritative spec pack.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Tests boundary:
  - Do not add new tests or new test files.
  - Only update existing tests if required to restore baseline expectations after the spec’s behavior change.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-code"`
3. Hand off baseline test commands and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
