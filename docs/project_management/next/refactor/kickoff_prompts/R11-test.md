# Task R11-test (Routing dispatch modularization) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R11-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R11-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r11-routing-test
   git worktree add wt/cr-r11-routing-test cr-r11-routing-test
   cd wt/cr-r11-routing-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R11-code)
- Crate: `crates/shell` (execution/routing).
- Update/move tests to match the dispatch modularization (registry + category modules); keep behavior identical for world enable/disable, builtins, exec/launch paths.
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
2. Commit worktree changes with a descriptive message (e.g., `test: align routing dispatch modules`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r11-routing-test
   git merge --ff-only wt/cr-r11-routing-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r11-routing-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R11-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R11-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r11-routing-test`) if done and
   hand off per instructions.
