# Task R5-code (Shell execution decomposition) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R5-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R5-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r5-exec-code
   git worktree add wt/cr-r5-exec-code cr-r5-exec-code
   cd wt/cr-r5-exec-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R5-test)
- Crate: `crates/shell` (execution stack).
- Break `execution/mod.rs` (~7.5k lines) into routing, invocation planning, and
  platform adapter modules with a thin public surface.
- Split `pty_exec.rs` into control/data-plane modules, keeping the channel-based
  pattern and resize/write/close semantics intact.
- Extract settings/manager initialization helpers into focused modules; preserve
  CLI flags, logging/redaction, and cfg-gates.

## Scope & Guardrails
- Production code and documentation only; avoid adding tests beyond doc examples
  needed to explain new module boundaries.
- No CLI/config behavior changes. Keep tracing/telemetry hooks unchanged and
  maintain platform-specific gates.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: split shell execution stack`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r5-exec-code
   git merge --ff-only wt/cr-r5-exec-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r5-exec-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R5-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R5-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r5-exec-code`) if done and
   hand off per instructions.
