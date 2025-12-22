# Task R2-code (Shell decomposition & PTY channelization) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R2-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R2-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r2-shell-code
   git worktree add wt/cr-r2-shell-code cr-r2-shell-code
   cd wt/cr-r2-shell-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R2-test)
- Crate: `shell`.
- Reduce `crates/shell/src/lib.rs` to a thin (~200 lines) public surface that
  re-exports modules under `execution/`, `repl/`, `builtins/`, and `scripts/`
  per `CRATE_REFACTORING_ANALYSIS.md`.
- Replace nested `Arc<Mutex>` PTY handling with a channel/message-based manager;
  no global mutable PTY writers or new panics.
- Preserve CLI flags/help/behavior and update docs that reference module paths
  (`docs/USAGE.md`, `docs/CONFIGURATION.md`) if needed.
- Keep platform cfg-gates intact and logging/redaction consistent with existing
  patterns.

## Scope & Guardrails
- Production code only. **Do not add or modify tests** except minimal test-only
  helpers required by the refactor.
- Avoid drive-by changes; stay scoped to the module split and PTY
  channelization. Maintain public APIs and feature flags.
- Preserve behavior across platforms; document any intentional surface changes.

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor:
   decompose shell and channelize pty`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r2-shell-code
   git merge --ff-only wt/cr-r2-shell-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r2-shell-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R2-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R2-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r2-shell-code`) if done and
   hand off per instructions.
