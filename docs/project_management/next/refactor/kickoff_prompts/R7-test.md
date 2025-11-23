# Task R7-test (Service module slimming) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R7-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R7-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r7-services-test
   git worktree add wt/cr-r7-services-test cr-r7-services-test
   cd wt/cr-r7-services-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R7-code)
- Crates: `host-proxy`, `world` (overlayfs), `replay`.
- Add tests/fixtures covering host-proxy config/transport/runtime seams for the
  new modules.
- Add overlayfs layering and cleanup coverage after the split; include property
  or table-driven cases where helpful.
- Extend replay planner/executor tests to match the decomposed modules and
  ensure CLI/API semantics remain stable.

## Scope & Guardrails
- Test-only changes: fixtures, helpers, doctests/property tests as needed.
- No production code edits beyond test-only helpers; keep cfg-gates, logging,
  and performance expectations intact.

## Suggested Commands
```
cargo fmt
cargo test -p host-proxy
cargo test -p world
cargo test -p substrate-replay --all-targets
```

## End Checklist
1. Ensure fmt/tests above are green; capture outputs (and skips) for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `test: cover service module split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r7-services-test
   git merge --ff-only wt/cr-r7-services-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r7-services-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R7-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R7-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r7-services-test`) if done
   and hand off per instructions.
