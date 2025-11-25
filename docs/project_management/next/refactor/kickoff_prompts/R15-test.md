# Task R15-test (Shell integration test split fixtures) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R15-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R15-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r15-shell-integration-test
   git worktree add wt/cr-r15-shell-integration-test cr-r15-shell-integration-test
   cd wt/cr-r15-shell-integration-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R15-code)
- Crate: `crates/shell`.
- Refactor integration test fixtures/utilities to support the split suites; keep behavior/coverage identical.
- Consolidate shared helpers into a support module; update imports accordingly.

## Scope & Guardrails
- Test files/fixtures/harnesses only; production code changes limited to test-only helpers.
- Preserve logging/redaction expectations and platform skips; avoid new external dependencies.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure the commands above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: refactor shell integration fixtures`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r15-shell-integration-test
   git merge --ff-only wt/cr-r15-shell-integration-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r15-shell-integration-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R15-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R15-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r15-shell-integration-test`) if done and
   hand off per instructions.
