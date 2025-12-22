# Task R9c-code (Routing split: world/agent flows) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R9c-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R9c-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r9c-routing-code
   git worktree add wt/cr-r9c-routing-code cr-r9c-routing-code
   cd wt/cr-r9c-routing-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R9c-test)
- Crate: `crates/shell` (execution/routing).
- Extract world enable/disable flows, agent client wiring, and platform bridging from `routing.rs` into focused modules with thin re-exports.
- Preserve CLI behavior, tracing, redaction, and cfg gates.

## Scope & Guardrails
- Production code and docs only; no tests in this code task.
- Keep world/agent behavior unchanged; maintain API compatibility via re-exports.
- Preserve logging/telemetry hooks and platform-gated branches.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: split routing world flows`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r9c-routing-code
   git merge --ff-only wt/cr-r9c-routing-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r9c-routing-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R9c-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R9c-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r9c-routing-code`) if done and
   hand off per instructions.
