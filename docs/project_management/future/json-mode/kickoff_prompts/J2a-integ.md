# Task J2a-integ (Structured output – core commands) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J2a-code and J2a-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J2a-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2a-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j2a-coverage-integ
   git worktree add wt/cs-j2a-coverage-integ cs-j2a-coverage-integ
   cd wt/cs-j2a-coverage-integ
   ```

## Duties
- Merge `cs-j2a-coverage-code` and `cs-j2a-coverage-test` branches; resolve conflicts.
- Run/log:
  ```
  cargo fmt
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ```
- Spot-check docs/help for the updated commands.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate structured output core coverage`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry) and create kickoff prompts for J2b tasks; commit docs (`git commit -am "docs: finish J2a-integ"`).
4. Hand off per workflow.
