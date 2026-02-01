# Kickoff: WFGAD5-integ (integration final — cross-platform merge)

## Scope
- Merge WFGAD5 core + platform-fix branches (if any) and finalize the slice with a clean merged state.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny/WFGAD5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task merges back to the orchestration branch after all required platforms are green.

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-wfgad5-integ` on branch `world-fs-granular-allow-deny-wfgad5-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-fs-granular-allow-deny/plan.md`, `docs/project_management/next/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/next/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.

## Requirements
- Merge the WFGAD5 core integration branch and any platform-fix branches that produced commits:
  - `world-fs-granular-allow-deny-wfgad5-integ-core`
  - `world-fs-granular-allow-deny-wfgad5-integ-linux` (if used)
  - `world-fs-granular-allow-deny-wfgad5-integ-macos` (if used)
  - `world-fs-granular-allow-deny-wfgad5-integ-windows` (if used)
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Cross-platform CI is not run from this task. Cross-platform CI runs only via `CP3-ci-checkpoint` per `docs/project_management/next/world-fs-granular-allow-deny/ci_checkpoint_plan.md`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGAD5-integ"`
2. Hand off evidence and any remaining follow-ups to the operator (do not edit planning docs inside the worktree).
