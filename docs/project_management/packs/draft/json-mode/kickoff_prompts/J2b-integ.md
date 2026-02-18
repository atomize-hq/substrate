# Task J2b-integ (Structured output – world & shim) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J2b-code and J2b-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J2b-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2b-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j2b-coverage-integ
   git worktree add wt/cs-j2b-coverage-integ cs-j2b-coverage-integ
   cd wt/cs-j2b-coverage-integ
   ```

## Duties
- Merge `cs-j2b-coverage-code` and `cs-j2b-coverage-test`; resolve conflicts.
- Run/log:
  ```
  cargo fmt
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ./tests/installers/install_smoke.sh   # run or note skip
  ```
- Confirm docs/CLI help reflect structured outputs.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate structured output world/shim`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry) and create kickoff prompts for J2c tasks; commit docs (`git commit -am "docs: finish J2b-integ"`).
4. Hand off per workflow.
