# Task C1-integ (Integrate config init) – INTEGRATION

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Confirm `C1-code` and `C1-test` are marked `completed` with branches pushed.
3. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `C1-integ` to `in_progress`, add a START entry to the session log, and
   commit the doc update (`git commit -am "docs: start C1-integ"`).
5. Create the integration branch/worktree:
   ```
   git checkout -b cs-c1-config-integ
   git worktree add wt/cs-c1-config-integ cs-c1-config-integ
   cd wt/cs-c1-config-integ
   ```

## Duties
- Merge `cs-c1-config-code` and `cs-c1-config-test` into the integration branch,
  resolving conflicts (docs/tests/code) while preserving the agreed spec.
- Run required commands and record outputs:
  ```
  cargo fmt
  cargo clippy -p substrate-shell -- -D warnings
  cargo test -p substrate-shell world_root
  ./tests/installers/install_smoke.sh   # or document skip if platform-restricted
  ```
- Spot-check docs/help output for the new `config init` command.
- Ensure net result still hints users when config is missing.

## Guardrails
- No new functionality or tests; focus on merging and validation.
- If conflicts arise, coordinate solutions that honor both branches’ intent;
  document any follow-up tasks if gaps remain.

## End Checklist
1. After fmt/clippy/tests succeed, commit integration fixes in the worktree
   (e.g., `chore: integrate config init code+tests`).
2. Merge the integration branch back into `feat/config-subcommand`
   (fast-forward) and remove the worktree when finished.
3. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` with command results, and create kickoff prompts for
   `C2-code` and `C2-test`.
4. Commit the doc updates on `feat/config-subcommand`
   (`git commit -am "docs: finish C1-integ"`). Hand off per workflow.
