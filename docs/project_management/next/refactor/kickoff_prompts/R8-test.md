# Task R8-test (Shell execution file slicing) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R8-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R8-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r8-shell-slim-test
   git worktree add wt/cr-r8-shell-slim-test cr-r8-shell-slim-test
   cd wt/cr-r8-shell-slim-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R8-code)
- Crate: `crates/shell` (execution stack follow-up).
- Add/reshape tests to mirror the new module layout after splitting:
  - PTY IO modules: resize/write/close/channel flows and error cases.
  - Invocation planning/env/PATH/cwd resolution.
  - Settings builder/validation and manager initialization helpers.
  - Platform adapter selection/world doctor behavior under cfg gates.
- Update fixtures/utilities to the new module paths; keep behavior assertions intact.

## Scope & Guardrails
- Test files/fixtures/harnesses only; production code changes limited to test-only helpers.
- Preserve existing expectations for logging/redaction and platform skips.
- Keep tests self-contained; avoid introducing new external dependencies.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure the commands above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: cover shell execution splits`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r8-shell-slim-test
   git merge --ff-only wt/cr-r8-shell-slim-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r8-shell-slim-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R8-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R8-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r8-shell-slim-test`) if done and
   hand off per instructions.
