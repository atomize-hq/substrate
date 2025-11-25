# Task R14-code (Routing dispatch module trimming) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R14-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R14-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r14-routing-dispatch-code
   git worktree add wt/cr-r14-routing-dispatch-code cr-r14-routing-dispatch-code
   cd wt/cr-r14-routing-dispatch-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R14-test)
- Crate: `crates/shell` (execution/routing/dispatch).
- Trim `dispatch/mod.rs` by moving shared helpers/state/types into submodules and keeping a thin orchestrator with re-exports to preserve API stability.
- Keep CLI/config behavior unchanged; tracing/redaction and cfg gates intact.

## Scope & Guardrails
- Production code and docs only; no tests in this code task.
- Preserve logging/telemetry hooks; maintain existing dispatch semantics.
- Re-export as needed to avoid breaking callers/tests.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: slim routing dispatch mod`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r14-routing-dispatch-code
   git merge --ff-only wt/cr-r14-routing-dispatch-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r14-routing-dispatch-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R14-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R14-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r14-routing-dispatch-code`) if done and
   hand off per instructions.
