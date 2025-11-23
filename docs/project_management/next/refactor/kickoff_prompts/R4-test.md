# Task R4-test (Documentation & polish validation) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R4-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R4-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r4-polish-test
   git worktree add wt/cr-r4-polish-test cr-r4-polish-test
   cd wt/cr-r4-polish-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R4-code)
- Crates: `trace`, `world-windows-wsl`, `replay`, `common`.
- Add doctests/property tests validating the new module splits and replay
  documentation; ensure fixtures align with refactored module layouts.
- Cover `common` prelude/doc changes and confirm trace/world-windows-wsl splits
  behave as expected on supported platforms.
- Record any benchmarks or performance checks if run; note skips/guards in the
  session log.

## Scope & Guardrails
- Tests only. Avoid production code changes except minimal hooks required for
  testing. Preserve feature flags, platform cfg-gates, and redaction/logging
  expectations.
- No coverage expansion beyond the R4 spec; keep CLI/config behavior intact.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-trace
cargo test -p world-windows-wsl          # on supported platforms; note skips
cargo test -p substrate-replay --all-targets
cargo test -p substrate-common --all-targets
cargo test --doc -p substrate-replay
```

## End Checklist
1. Ensure fmt/tests above are green; capture outputs for the END log entry and
   note any platform skips/benchmarks.
2. Commit worktree changes with a descriptive message (e.g., `test: validate
   trace/replay splits and docs`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r4-polish-test
   git merge --ff-only wt/cr-r4-polish-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r4-polish-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R4-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R4-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r4-polish-test`) if done and
   hand off per instructions.
