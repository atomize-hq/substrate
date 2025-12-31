# Kickoff: C0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/tmp-make-scaffold-c0-integ-core` on branch `tmp-make-scaffold-c0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" TASK_ID="C0-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run required integration gates (must be green before any CI smoke dispatch):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- If the feature directory contains `smoke/`, run cross-platform smoke via CI (validation-only):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" PLATFORM=all WORKFLOW_REF="feat/tmp-make-scaffold"`
  - This feature requires WSL coverage, so add `RUN_WSL=1`.
  - Record the emitted `RUN_ID=<id>` (and URL).
- If any platform smoke fails, start only the failing platform-fix tasks (do not attempt platform-specific fixes here):
  - Ask the operator to run from the orchestration worktree:
    - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" SLICE_ID="C0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
- Once all failing platform-fix tasks report green smoke, start the final aggregator:
  - Ask the operator to run from the orchestration worktree:
    - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" SLICE_ID="C0" LAUNCH_CODEX=1`

## End Checklist
1. Run required commands; capture outputs (including any smoke run ids/URLs).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-core"`
3. On the orchestration branch, update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-integ-core`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
