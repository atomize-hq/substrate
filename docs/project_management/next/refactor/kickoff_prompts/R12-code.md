# Task R12-code (Routing builtin/world runner slimming) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R12-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R12-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r12-routing-builtin-code
   git worktree add wt/cr-r12-routing-builtin-code cr-r12-routing-builtin-code
   cd wt/cr-r12-routing-builtin-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R12-test)
- Crate: `crates/shell` (execution/routing + builtins).
- Split `execution/routing/builtin.rs` into smaller category modules (world/deps, shim actions, utility) with thin re-exports.
- Slim `builtins/world_enable/runner.rs` into helpers/modules without changing outputs or logging.
- Preserve CLI/config behavior, tracing/redaction, and cfg gates.

## Scope & Guardrails
- Production code and docs only; no tests in this code task.
- Keep behavior identical; maintain API compatibility via re-exports.
- Preserve logging/telemetry hooks and platform guards.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: slim routing builtins`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r12-routing-builtin-code
   git merge --ff-only wt/cr-r12-routing-builtin-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r12-routing-builtin-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R12-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R12-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r12-routing-builtin-code`) if done and
   hand off per instructions.
