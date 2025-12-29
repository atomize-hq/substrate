# Task R2c-integ (Replay agent-backed execution) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2a-code, R2b-code, and R2c-test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2c-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2c-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2c-replay-agent-integ
   git worktree add wt/ps-r2c-replay-agent-integ ps-r2c-replay-agent-integ
   cd wt/ps-r2c-replay-agent-integ
   ```

## Scope
- Merge ps-r2a-replay-agent-code, ps-r2b-replay-fallback-code, and ps-r2c-replay-agent-test into a coherent replay stack.
- Resolve any conflicts across replay/shell/world-agent/docs; ensure default agent preference, fallback warnings, copy-diff retries, and new tests all pass together.
- Update docs/tasks/session log, confirm prompts for follow-on work (if any).

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Note platform skips or additional manual `substrate --replay ...` validation.

## End Checklist
1. Merge code/test branches, resolve conflicts, fast-forward to feat/p0-platform-stability-follow-up.
2. Run required fmt/lint/tests; capture results.
3. Update `tasks.json` + `session_log.md` END entry with executed commands/outcomes.
4. Commit doc/task/log updates (`git commit -am "docs: finish R2c-integ"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
