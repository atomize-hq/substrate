# Task R5-integ (Shell execution decomposition) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R5-code` and `R5-test` are completed.
4. Set `R5-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R5-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r5-exec-integ
   git worktree add wt/cr-r5-exec-integ cr-r5-exec-integ
   cd wt/cr-r5-exec-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec
- Merge the shell execution decomposition code/test branches and resolve any
  conflicts across the new module layout.
- Ensure routing/invocation/PTY module splits preserve behavior and logging/cfg
  gates; align fixtures/tests with final layout.

## Scope & Guardrails
- No new features; only conflict resolution and polish needed for merged code +
  tests.
- Keep CLI/config semantics identical; avoid touching unrelated crates.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Resolve merges between R5 code/test branches inside the integration worktree.
2. Ensure fmt/clippy/tests above are green; log results (including any platform
   skips).
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R5 shell execution split`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r5-exec-integ
   git merge --ff-only wt/cr-r5-exec-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r5-exec-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and capture kickoff prompts for
   R6 code/test if not already present.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R5-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r5-exec-integ`) and hand off
   per instructions.
