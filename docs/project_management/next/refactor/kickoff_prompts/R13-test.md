# Task R13-test (Broker/lib slimming & cleanup) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R13-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R13-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r13-broker-test
   git worktree add wt/cr-r13-broker-test cr-r13-broker-test
   cd wt/cr-r13-broker-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R13-code)
- Crate: `crates/broker`.
- Update/move tests to match the broker/lib.rs split (profiles, policy loading, watch plumbing); ensure behavior/logging unchanged.
- Refresh fixtures/utilities to new module paths.

## Scope & Guardrails
- Test files/fixtures/harnesses only; production code changes limited to test-only helpers.
- Preserve logging/redaction expectations and cfg gates; avoid new external dependencies.

## Suggested Commands
```
cargo fmt
cargo test -p broker
```

## End Checklist
1. Ensure the commands above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: cover broker lib split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r13-broker-test
   git merge --ff-only wt/cr-r13-broker-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r13-broker-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R13-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R13-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r13-broker-test`) if done and
   hand off per instructions.
