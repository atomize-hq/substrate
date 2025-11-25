# Task C2-integ (Integrate config show) – INTEGRATION

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Verify `C2-code` and `C2-test` are completed with branches available.
3. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
4. Set `C2-integ` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start C2-integ"`).
5. Create branch/worktree:
   ```
   git checkout -b cs-c2-show-integ
   git worktree add wt/cs-c2-show-integ cs-c2-show-integ
   cd wt/cs-c2-show-integ
   ```

## Duties
- Merge `cs-c2-show-code` and `cs-c2-show-test`; resolve conflicts and ensure
  combined behavior matches the shared spec.
- Commands to run/log:
  ```
  cargo fmt
  cargo clippy -p substrate-shell -- -D warnings
  cargo test -p substrate-shell world_root
  cargo test -p substrate-shell world_enable
  ```
  (Add installer smoke if relevant; document skips.)
- Spot-check CLI help and docs to ensure `config show` appears as expected.

## Guardrails
- No new functionality or tests—merge and validate only.
- Coordinate with prior agents if conflicts require spec interpretation.

## End Checklist
1. Commit integration fixes (e.g., `chore: integrate config show code+tests`).
2. Fast-forward merge into `feat/config-subcommand`, remove worktree afterward.
3. Update `tasks.json` (status → `completed`) and session log (END entry with
   command results). Create kickoff prompts for `C3-code` and `C3-test`.
4. Commit doc updates on `feat/config-subcommand`
   (`git commit -am "docs: finish C2-integ"`). Hand off.
