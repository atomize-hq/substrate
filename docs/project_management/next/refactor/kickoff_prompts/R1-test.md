# Task R1-test (Remove library panics) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R1-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R1-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r1-panics-test
   git worktree add wt/cr-r1-panics-test cr-r1-panics-test
   cd wt/cr-r1-panics-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R1-code)
- Crates: `broker`, `world`, `telemetry-lib`, `forwarder`.
- Replace all library `.unwrap()` panic paths with `Result`-based flows using
  `anyhow::Context` for rich errors; no new panics introduced.
- Preserve public API behavior (or deprecate/document if unavoidable) and keep
  logging/redaction consistent with `crates/common/src/log_schema.rs`.
- Ensure panic-focused tests can validate poisoned locks/error paths (test agent
  authors these tests without relying on visibility into code branches).
- Update docs/CHANGELOG only as needed to reflect error-surface changes.

## Scope & Guardrails
- **Tests only. Do not modify production code** (except minimal test-only
  helpers/fixtures if strictly required).
- Author panic-focused tests for the four crates (poisoned locks, error paths)
  that assert `Result` errors instead of panics. Keep fixtures isolated to test
  modules.
- Mirror the shared spec; assume code agent has implemented the panic removal.

## Suggested Commands
```
cargo fmt
cargo test -p broker
cargo test -p world
cargo test -p telemetry-lib
cargo test -p forwarder
# Run any additional targeted suites you add; log outputs in END entry.
```

## End Checklist
1. Ensure fmt/tests you ran are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: add panic guards for broker/world/telemetry/forwarder`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r1-panics-test
   git merge --ff-only wt/cr-r1-panics-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r1-panics-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and create/confirm the
   integration prompt path (`docs/project_management/next/refactor/kickoff_prompts/R1-integ.md`).
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R1-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r1-panics-test`) if done and
   hand off per instructions.
