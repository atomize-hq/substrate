# Task R6-integ (Bootstrap & builtins decomposition) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R6-code` and `R6-test` are completed.
4. Set `R6-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R6-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r6-bootstrap-integ
   git worktree add wt/cr-r6-bootstrap-integ cr-r6-bootstrap-integ
   cd wt/cr-r6-bootstrap-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec
- Merge the manager_manifest/shim/builtin decomposition code/test branches and
  resolve conflicts across the new module layout.
- Ensure manifest semantics, shim exec bootstrap, and builtin outputs/logging
  remain stable; align fixtures/tests to the final structure.

## Scope & Guardrails
- No new features; focus on conflict resolution and polish for merged code/tests.
- Preserve cfg-gates, logging/redaction, and CLI/config behavior.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-common --all-targets
cargo test -p substrate-shim
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Resolve merges between R6 code/test branches inside the integration worktree.
2. Ensure fmt/clippy/tests above are green; log results and skips.
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R6 bootstrap/builtins split`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r6-bootstrap-integ
   git merge --ff-only wt/cr-r6-bootstrap-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r6-bootstrap-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and capture kickoff prompts for
   R7 code/test if not already present.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R6-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r6-bootstrap-integ`) and hand
   off per instructions.
