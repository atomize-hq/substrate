# Task R14-integ (Routing dispatch module trimming) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R14-code` and `R14-test` are completed.
4. Set `R14-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R14-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r14-routing-dispatch-integ
   git worktree add wt/cr-r14-routing-dispatch-integ cr-r14-routing-dispatch-integ
   cd wt/cr-r14-routing-dispatch-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec
- Merge the R14 code and test branches for routing dispatch trimming.
- Resolve conflicts, ensure module paths/exports align, and keep behavior unchanged.
- Run required fmt/lint/tests and document results.

## Scope & Guardrails
- Integration only: no new production features; minimal touch-ups to reconcile code/test branches.
- Preserve platform cfg gates, tracing/logging, and redaction helpers.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Merge code/test branches into the integration branch/worktree and resolve conflicts.
2. Run the commands above; capture outputs for the END log entry.
3. Return to repo root, merge the integration branch:
   ```
   git checkout cr-r14-routing-dispatch-integ
   git merge --ff-only wt/cr-r14-routing-dispatch-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r14-routing-dispatch-integ
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers).
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R14-integ"`).
6. Remove the worktree (`git worktree remove wt/cr-r14-routing-dispatch-integ`) if done and
   push or hand off per instructions.
