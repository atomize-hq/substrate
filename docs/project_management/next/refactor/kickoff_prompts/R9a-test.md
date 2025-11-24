# Task R9a-test (Routing split: dispatch & builtins) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R9a-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R9a-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r9a-routing-test
   git worktree add wt/cr-r9a-routing-test cr-r9a-routing-test
   cd wt/cr-r9a-routing-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R9a-code)
- Crate: `crates/shell` (execution/routing).
- Update/move tests to match the routing dispatch/builtin module split; ensure behavior stays the same (world enable/disable flows, builtin selection).
- Keep fixtures/utilities aligned with new module paths.

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
2. Commit worktree changes with a descriptive message (e.g., `test: align routing dispatch split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r9a-routing-test
   git merge --ff-only wt/cr-r9a-routing-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r9a-routing-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R9a-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R9a-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r9a-routing-test`) if done and
   hand off per instructions.
