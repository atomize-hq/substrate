# Task J1-integ (Integrate structured mode scaffold) – INTEGRATION

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Confirm J1-code and J1-test completed.
3. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `J1-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J1-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-j1-structured-integ
   git worktree add wt/cs-j1-structured-integ cs-j1-structured-integ
   cd wt/cs-j1-structured-integ
   ```

## Duties
- Merge `cs-j1-structured-code` and `cs-j1-structured-test`, resolving conflicts while preserving the shared spec.
- Execute/log:
  ```
  cargo fmt
  cargo clippy -p substrate-shell -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ```
  (Installer smoke optional; document if run/skipped.)
- Spot-check CLI help/docs to ensure structured-mode flags documented.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate structured mode scaffold`).
2. Merge branch into `feat/json-mode`, remove worktree.
3. Update `tasks.json` + `session_log.md` (END entry) and create kickoff prompts for J2a-code/test; commit docs (`git commit -am "docs: finish J1-integ"`).
4. Hand off per workflow.
