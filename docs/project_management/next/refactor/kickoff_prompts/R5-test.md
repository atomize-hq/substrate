# Task R5-test (Shell execution decomposition) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R5-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R5-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r5-exec-test
   git worktree add wt/cr-r5-exec-test cr-r5-exec-test
   cd wt/cr-r5-exec-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R5-code)
- Crate: `crates/shell` execution stack.
- Add/reshape tests for the split execution modules covering routing and
  invocation planning seams.
- Cover PTY control/data-plane interactions (resize/write/close) with channel
  assertions and mocks where useful.
- Update fixtures to reflect new module layout; keep CLI/config semantics and
  logging expectations unchanged.

## Scope & Guardrails
- Test-only changes: fixtures, helpers, doctests/property tests as needed.
- No production code edits except tiny test-only helpers; keep platform cfg
  guards and tracing expectations intact.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/tests above are green; capture outputs (and skips) for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `test: cover shell execution split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r5-exec-test
   git merge --ff-only wt/cr-r5-exec-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r5-exec-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R5-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R5-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r5-exec-test`) if done and
   hand off per instructions.
