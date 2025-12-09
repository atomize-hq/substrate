# Task R10-code (PTY IO module slicing) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R10-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R10-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r10-pty-code
   git worktree add wt/cr-r10-pty-code cr-r10-pty-code
   cd wt/cr-r10-pty-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R10-test)
- Crate: `crates/shell` (execution/pty/io).
- Split `execution/pty/io/mod.rs` into focused modules (types/traits, reader path, writer path, test utilities) with thin re-exports.
- Preserve channel/resize/write/close semantics, tracing/logging, redaction, and cfg gates.

## Scope & Guardrails
- Production code and docs only; no tests in this code task.
- Maintain API compatibility via re-exports; keep behavior identical.
- Preserve platform-gated branches and telemetry hooks.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: split pty io modules`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r10-pty-code
   git merge --ff-only wt/cr-r10-pty-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r10-pty-code
   ```
3. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R10-test.md`)
   is referenced.
4. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R10-code"`).
5. Remove the worktree (`git worktree remove wt/cr-r10-pty-code`) if done and
   hand off per instructions.
