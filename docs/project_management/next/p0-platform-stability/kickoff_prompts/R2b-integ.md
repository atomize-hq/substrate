# Task R2b-integ (Replay fallbacks & warnings) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2b-code/test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2b-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2b-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2b-replay-fallback-integ
   git worktree add wt/ps-r2b-replay-fallback-integ ps-r2b-replay-fallback-integ
   cd wt/ps-r2b-replay-fallback-integ
   ```

## Scope
- Merge ps-r2b-replay-fallback-code/test, resolve conflicts, and ensure warning/copy-diff plumbing behaves as expected across replay + shell.
- Re-run fmt/lint/tests (replay + replay_world suite) and capture results.
- Update docs/tasks/session log; confirm R2c prompts ready.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Note any skips/platform limitations plus manual `substrate --replay ...` validation logs.

## End Checklist
1. Merge code/test branches, resolve conflicts, fast-forward to feat/p0-platform-stability-follow-up.
2. Run required fmt/lint/tests; capture results.
3. Update `tasks.json` + `session_log.md` END entry (commands + outcomes).
4. Commit doc/task/log updates (`git commit -am "docs: finish R2b-integ"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
