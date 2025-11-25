# Task J3a-integ (JSON input – simple commands) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J3a-code and J3a-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J3a-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3a-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j3a-input-integ
   git worktree add wt/cs-j3a-input-integ cs-j3a-input-integ
   cd wt/cs-j3a-input-integ
   ```

## Duties
- Merge `cs-j3a-input-code` and `cs-j3a-input-test`; resolve conflicts.
- Run/log:
  ```
  cargo fmt
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test -p substrate-shell world_root
  ```
- Confirm docs cover payload schemas/preference rules.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate json input plumbing`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry) and create kickoff prompts for J3b tasks; commit docs (`git commit -am "docs: finish J3a-integ"`).
4. Hand off per workflow.
