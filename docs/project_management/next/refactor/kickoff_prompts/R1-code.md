# Task R1-code (Remove library panics) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R1-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R1-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r1-panics-code
   git worktree add wt/cr-r1-panics-code cr-r1-panics-code
   cd wt/cr-r1-panics-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R1-test)
- Crates: `broker`, `world`, `telemetry-lib`, `forwarder`.
- Replace all library `.unwrap()` panic paths with `Result`-based flows using
  `anyhow::Context` for rich errors; no new panics introduced.
- Preserve public API behavior (or deprecate/document if unavoidable) and keep
  logging/redaction consistent with `crates/common/src/log_schema.rs`.
- Ensure panic-focused tests can validate poisoned locks/error paths (test agent
  will author tests separately); keep behavior consistent across platforms.
- Update docs/CHANGELOG only as needed to reflect error-surface changes.

## Scope & Guardrails
- Production code only. **Do not add or modify tests** (test agent owns tests).
- Keep changes scoped to the four crates plus minimal doc/CHANGELOG updates tied
  to the refactor; avoid drive-by cleanups.
- Maintain existing feature flags/cfg-gates and avoid altering public types
  unless documented.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
# Targeted tests if needed for confidence (log in END entry):
cargo test -p broker
cargo test -p world
cargo test -p telemetry-lib
cargo test -p forwarder
```

## End Checklist
1. Ensure fmt/clippy (and any executed tests) are green; capture outputs for the
   END log entry.
2. Commit worktree changes with a descriptive message (e.g., `fix: remove panics in broker/world/telemetry/forwarder`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r1-panics-code
   git merge --ff-only wt/cr-r1-panics-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r1-panics-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R1-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R1-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r1-panics-code`) if done and
   hand off per instructions.
