# Task J3b-integ (JSON input – complex workflows) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J3b-code and J3b-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J3b-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3b-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j3b-input-integ
   git worktree add wt/cs-j3b-input-integ cs-j3b-input-integ
   cd wt/cs-j3b-input-integ
   ```

## Duties
- Merge `cs-j3b-input-code` and `cs-j3b-input-test`; resolve conflicts.
- Run/log:
  ```
  cargo fmt
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ./tests/installers/install_smoke.sh   # run or note skip
  ```
- Confirm docs/backlog reflect JSON I/O completion.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate json input complex workflows`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry), capture backlog status, and commit docs (`git commit -am "docs: finish J3b-integ"`).
4. Hand off or close the effort per workflow.
