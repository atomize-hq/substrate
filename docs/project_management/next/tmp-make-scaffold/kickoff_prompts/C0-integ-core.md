# Kickoff: C0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/tmp-make-scaffold-c0-integ-core` on branch `tmp-make-scaffold-c0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" TASK_ID="C0-integ-core" LAUNCH_CODEX=1`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Merge the task branches for this slice into this worktree (resolve conflicts/drift):
  - `tmp-make-scaffold-c0-code`
  - `tmp-make-scaffold-c0-test`
- Run integration gates and ensure the state you intend to validate is committed on `HEAD`:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Preferred sequencing (matches the e2e flow): finish this task (runs gates + commits) before dispatching smoke:
  - From inside this worktree: `make triad-task-finish TASK_ID="C0-integ-core"`

### Cross-platform smoke via CI (validation-only)

Dispatch from this **integ-core worktree**, because the smoke dispatcher validates the current `HEAD` by creating/pushing a throwaway branch at that commit.

- `make feature-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" PLATFORM=all RUN_WSL=1 RUNNER_KIND=self-hosted WORKFLOW_REF="feat/tmp-make-scaffold" REMOTE=origin CLEANUP=1`

If any platform fails:
- Do not attempt platform-specific fixes in integ-core.
- Ask the operator to start only the failing platform-fix tasks from the emitted `RUN_ID` (operator runs this from the orchestration checkout, not a task worktree):
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" SLICE_ID="C0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

If `PLATFORM=all` smoke is green:
- Platform-fix tasks may still be required for CI-only failures (e.g., clippy warnings on macOS/Windows), so do not mark them no-op yet.
- The wrapper/final gate should run CI Testing; if CI Testing is green, then mark `C0-integ-linux|macos|windows` as `completed` no-ops to unblock `C0-integ`:
  - `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir "docs/project_management/next/tmp-make-scaffold" --slice-id "C0" --from-smoke-run "<run-id>"`

After all required platforms are green (and platform-fix tasks are completed), ask the operator to start `C0-integ` (integration final):
- `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" SLICE_ID="C0" LAUNCH_CODEX=1`

## End Checklist
1. Ensure this worktree is finished via `make triad-task-finish TASK_ID="C0-integ-core"`.
2. Dispatch cross-platform smoke and include these exact key/value lines in your handoff (wrapper agents parse them):
   - `RUN_ID=<id>`
   - `RUN_URL=<url>` (if printed)
   - `SMOKE_PASSED_PLATFORMS=<csv>`
   - `SMOKE_FAILED_PLATFORMS=<csv>` (empty means success)
3. Hand off key outputs + smoke run ids/URLs to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
