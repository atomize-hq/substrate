# Task R9b-test (Routing split: path/env/cwd helpers) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R9b-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R9b-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r9b-routing-test
   git worktree add wt/cr-r9b-routing-test cr-r9b-routing-test
   cd wt/cr-r9b-routing-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R9b-code)
- Crate: `crates/shell` (execution/routing).
- Update/move tests to match the path/cwd/env/world-env helper split from `routing.rs`; keep behavior identical.
- Refresh fixtures/utilities to new module paths.

## Scope & Guardrails
- Test files/fixtures/harnesses only; production code changes limited to test-only helpers.
- Preserve logging/redaction expectations and platform-gated behaviors.
- Avoid new external dependencies.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure the commands above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: align routing env helpers`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r9b-routing-test
   git merge --ff-only wt/cr-r9b-routing-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r9b-routing-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R9b-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R9b-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r9b-routing-test`) if done and
   hand off per instructions.
