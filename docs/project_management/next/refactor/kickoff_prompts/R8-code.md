# Task R8-code (Shell execution file slicing) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R8-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R8-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r8-shell-slim-code
   git worktree add wt/cr-r8-shell-slim-code cr-r8-shell-slim-code
   cd wt/cr-r8-shell-slim-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R8-test)
- Crate: `crates/shell` (execution stack follow-up).
- Split oversized files into focused modules while preserving behavior:
  - `execution/pty/io.rs` (~1,328 LOC): separate IO traits/state from sinks/sources; keep channel-based PTY flow intact.
  - `execution/invocation.rs` (~1,080 LOC): isolate invocation planning vs env/PATH/cwd prep; maintain CLI/env semantics.
  - `execution/settings.rs` (~763 LOC) and `execution/manager_init.rs` (~668 LOC): extract builders/validation vs runtime helpers with thin surfaces.
  - `execution/platform.rs` (~721 LOC): isolate platform adapters/world doctor helpers behind cfg gates without changing outputs.
- Maintain thin public surfaces (re-export as needed), tracing/redaction, and platform guards.

## Scope & Guardrails
- Production code and docs only; tests remain unchanged in this code task.
- No CLI/config behavior changes; keep logging/telemetry intact.
- Preserve existing channel/PTY semantics and cfg(target_os) boundaries.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: split shell execution files`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r8-shell-slim-code
   git merge --ff-only wt/cr-r8-shell-slim-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r8-shell-slim-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R8-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R8-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r8-shell-slim-code`) if done and
   hand off per instructions.
