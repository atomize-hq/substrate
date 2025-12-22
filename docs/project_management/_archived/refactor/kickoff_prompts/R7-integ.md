# Task R7-integ (Service module slimming) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R7-code` and `R7-test` are completed.
4. Set `R7-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R7-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r7-services-integ
   git worktree add wt/cr-r7-services-integ cr-r7-services-integ
   cd wt/cr-r7-services-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec
- Merge host-proxy/overlayfs/replay code/test branches and resolve conflicts
  across the new module layout.
- Ensure behavior, logging/cfg-gates, and performance expectations remain
  stable; align fixtures/tests to final structure.

## Scope & Guardrails
- No new features; focus on conflict resolution and polish for merged code/tests.
- Keep binaries thin and APIs unchanged.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p host-proxy
cargo test -p world
cargo test -p substrate-replay --all-targets
```

## End Checklist
1. Resolve merges between R7 code/test branches inside the integration worktree.
2. Ensure fmt/clippy/tests above are green; log results (including skips).
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R7 service module split`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r7-services-integ
   git merge --ff-only wt/cr-r7-services-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r7-services-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and capture any follow-up
   prompts/notes.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R7-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r7-services-integ`) and hand
   off per instructions.
