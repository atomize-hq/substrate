# Task R10-test (PTY IO module slicing) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R10-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R10-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r10-pty-test
   git worktree add wt/cr-r10-pty-test cr-r10-pty-test
   cd wt/cr-r10-pty-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R10-code)
- Crate: `crates/shell` (execution/pty/io).
- Update/move tests to mirror the pty/io module split; assert resize/write/close/channel flows remain correct.
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
2. Commit worktree changes with a descriptive message (e.g., `test: cover pty io split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r10-pty-test
   git merge --ff-only wt/cr-r10-pty-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r10-pty-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R10-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R10-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r10-pty-test`) if done and
   hand off per instructions.
