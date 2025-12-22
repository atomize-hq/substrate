# Task R2-integ (Integrate shell decomposition) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R2-code` and `R2-test` are completed.
4. Set `R2-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update
   (`git commit -am "docs: start R2-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r2-shell-integ
   git worktree add wt/cr-r2-shell-integ cr-r2-shell-integ
   cd wt/cr-r2-shell-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared baseline)
- Crate: `shell`.
- Integrate the decomposed execution/repl/builtins/scripts modules and the
  channel-based PTY manager from `R2-code` with the extracted/expanded tests
  from `R2-test`.
- Ensure PTY handling remains channel-driven (resize/write/close), CLI behavior
  and logging/redaction remain unchanged, and fixtures/docs referenced by the
  shell crate stay aligned with the new module layout.

## Scope & Guardrails
- Merge only the R2 code and test branches; do not expand coverage or modify
  production behavior beyond the shared R2 spec.
- Keep shell-specific cfg-gates/feature flags intact; no drive-by changes in
  other crates.
- Update tasks/session log/docs on `feat/crate-refactor` only (not in worktree).

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
./tests/installers/install_smoke.sh
```
Capture outputs for the END log entry.

## End Checklist
1. Resolve merges between R2 code/test branches inside the integration worktree.
2. Ensure fmt/clippy/tests (above) are green and installer smoke results are
   recorded.
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R2 shell refactor`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r2-shell-integ
   git merge --ff-only wt/cr-r2-shell-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r2-shell-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and create/confirm next
   kickoff prompts for R3 code/test.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R2-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r2-shell-integ`) and hand
   off per instructions.
