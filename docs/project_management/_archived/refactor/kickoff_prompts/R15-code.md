# Task R15-code (Shell integration test split) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R15-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R15-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r15-shell-integration-code
   git worktree add wt/cr-r15-shell-integration-code cr-r15-shell-integration-code
   cd wt/cr-r15-shell-integration-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R15-test)
- Crate: `crates/shell`.
- Split `tests/integration.rs` into logical suites (e.g., world enable/disable, shim deployment, logging/rotation, env inheritance) with a small shared support module; keep assertions and fixtures intact.

## Scope & Guardrails
- Test structure refactor only; production code untouched.
- Preserve logging/redaction expectations and platform skips.
- Maintain existing coverage and behaviors.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure the commands above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: split shell integration suites`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r15-shell-integration-code
   git merge --ff-only wt/cr-r15-shell-integration-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r15-shell-integration-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R15-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R15-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r15-shell-integration-code`) if done and
   hand off per instructions.
