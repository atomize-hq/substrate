# Task R13-code (Broker/lib slimming & cleanup) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R13-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R13-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r13-broker-code
   git worktree add wt/cr-r13-broker-code cr-r13-broker-code
   cd wt/cr-r13-broker-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R13-test)
- Crate: `crates/broker`.
- Split `broker/src/lib.rs` into focused modules (profiles, policy loading, watch plumbing, API surface) with thin re-exports; keep behavior and logging unchanged.
- Preserve public API, tracing/redaction, and cfg gates; add brief rustdoc module headers.

## Scope & Guardrails
- Production code and docs only; no tests in this code task.
- Keep behavior identical; maintain API compatibility via re-exports.
- Preserve logging/telemetry hooks and platform guards.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p broker
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: split broker lib`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r13-broker-code
   git merge --ff-only wt/cr-r13-broker-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r13-broker-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R13-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R13-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r13-broker-code`) if done and
   hand off per instructions.
