# Task R7-code (Service module slimming) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R7-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R7-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r7-services-code
   git worktree add wt/cr-r7-services-code cr-r7-services-code
   cd wt/cr-r7-services-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R7-test)
- Crates: `host-proxy`, `world` (overlayfs), `replay`.
- Split `host-proxy/src/lib.rs` into thin public surface plus config/transport/
  runtime modules; keep binary thin and behavior unchanged.
- Break `world/src/overlayfs.rs` into layering/utils modules, preserving
  platform guards and semantics.
- Decompose `replay/src/replay.rs` into planners/executors/helpers with stable
  API/CLI semantics and docs.

## Scope & Guardrails
- Production code and docs only; no new features or CLI/config changes.
- Preserve logging, cfg-gates, and performance characteristics; update docs for
  new module paths if referenced.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p host-proxy
cargo test -p world
cargo test -p substrate-replay --all-targets
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry (note skips if any).
2. Commit worktree changes with a descriptive message (e.g., `refactor: slim host-proxy/overlayfs/replay`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r7-services-code
   git merge --ff-only wt/cr-r7-services-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r7-services-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R7-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R7-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r7-services-code`) if done
   and hand off per instructions.
