# Kickoff: WDL2-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world_deps_selection_layer-wdl2-integ-core` on branch `world_deps_selection_layer-wdl2-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" TASK_ID="WDL2-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Assert WDL2 capability before smoke dispatch:
  - `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && substrate world deps --help 2>/dev/null | grep -Eq '\\bprovision\\b'`
- Dispatch CI compile parity (fast fail):
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1`
- Dispatch behavioral smoke via CI from this worktree’s `HEAD`:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- If smoke fails on any platform, do not fix it in integ-core. Ask the operator to start only the failing platform-fix tasks:
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" SLICE_ID="WDL2" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist
1. Ensure local integration gates are green and your merged state is committed.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDL2-integ-core"`.
3. Hand off run ids/URLs and `SMOKE_FAILED_PLATFORMS` to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

