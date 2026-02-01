# Kickoff: WFGAD1-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny/WFGAD1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-wfgad1-integ-core` on branch `world-fs-granular-allow-deny-wfgad1-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-fs-granular-allow-deny/plan.md`, `docs/project_management/next/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/next/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny" TASK_ID="WFGAD1-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run local integration gates (must be green before finishing this task):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Cross-platform CI is not run from this task. Cross-platform CI runs only via `CP1-ci-checkpoint` per `docs/project_management/next/world-fs-granular-allow-deny/ci_checkpoint_plan.md`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGAD1-integ-core"`
2. Update tasks/session_log on orchestration branch (do not edit planning docs inside the worktree).
