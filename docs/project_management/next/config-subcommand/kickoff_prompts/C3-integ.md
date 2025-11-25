# Task C3-integ (Integrate config set) – INTEGRATION

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Confirm `C3-code` and `C3-test` completed.
3. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `C3-integ` to `in_progress`, log START entry, and commit doc update
   (`git commit -am "docs: start C3-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-c3-set-integ
   git worktree add wt/cs-c3-set-integ cs-c3-set-integ
   cd wt/cs-c3-set-integ
   ```

## Duties
- Merge `cs-c3-set-code` and `cs-c3-set-test`, resolve conflicts, and ensure the
  combined CLI behaves per spec (multi-key set, validation, JSON output).
- Run/log required commands:
  ```
  cargo fmt
  cargo clippy -p substrate-shell -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ./tests/installers/install_smoke.sh   # document skip if needed
  ```
- Spot-check docs/help/backlog to ensure the global configuration UX acceptance
  criteria are now satisfied.

## Guardrails
- No new functionality or tests beyond reconciliation fixes.
- Document any remaining gaps (parking lot/backlog) if the acceptance criteria
  still need follow-up.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate config set code+tests`).
2. Fast-forward merge into `feat/config-subcommand`, remove worktree afterward.
3. Update `tasks.json` (status → `completed`), append END entry to session log
   with command outputs, and note backlog status in the log if applicable.
4. Commit doc updates on `feat/config-subcommand`
   (`git commit -am "docs: finish C3-integ"`). Hand off or close out project.
