# Kickoff: WFGAD3-integ-macos (integration platform-fix — macos)

## Scope
- Fix macOS failures detected at checkpoint boundary for slice WFGAD3.
- This is a CI parity platform-fix task: compile/test/lint parity is required; smoke is not required.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD3-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Run this task on a macOS machine.
2. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-wfgad3-integ-macos` on branch `world-fs-granular-allow-deny-wfgad3-integ-macos` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/_archived/world-fs-granular-allow-deny/plan.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/_archived/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.

## Requirements
- Merge the slice’s core integration branch into this worktree before fixing:
  - core integration branch is `world-fs-granular-allow-deny-wfgad3-integ-core`.
- Fix macOS failures only; do not make unrelated refactors.
- Run local parity gates on macOS:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGAD3-integ-macos"`
2. Hand off evidence (what failed, what was fixed) to the operator (do not edit planning docs inside the worktree).

