# Task J2c-integ (Structured output – advanced) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J2c-code and J2c-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J2c-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2c-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j2c-coverage-integ
   git worktree add wt/cs-j2c-coverage-integ cs-j2c-coverage-integ
   cd wt/cs-j2c-coverage-integ
   ```

## Duties
- Merge `cs-j2c-coverage-code` and `cs-j2c-coverage-test`; resolve conflicts.
- Run/log:
  ```
  cargo fmt
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ./tests/installers/install_smoke.sh   # run or note skip
  ```
- Verify docs cover all schemas.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate structured output advanced commands`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry) and create kickoff prompts for J3a tasks; commit docs (`git commit -am "docs: finish J2c-integ"`).
4. Hand off per workflow.
