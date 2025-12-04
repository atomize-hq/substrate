# Task R2d-integ (Replay origin-aware defaults & agent routing) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2d-code/test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2d-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2d-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2d-replay-origin-integ
   git worktree add wt/ps-r2d-replay-origin-integ ps-r2d-replay-origin-integ
   cd wt/ps-r2d-replay-origin-integ
   ```

## Scope
- Merge ps-r2d-replay-origin-code/test, resolve conflicts across replay/shell/world-agent/trace/docs.
- Verify origin-aware defaults, flip flag, agent-first world path with fallback, and copy-diff override/warning behavior all pass together.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Capture any manual `substrate --replay ...` validation (agent healthy vs missing, flip on/off); note skips.

## End Checklist
1. Merge code/test branches, resolve conflicts, fast-forward to feat/p0-platform-stability-follow-up.
2. Run required fmt/lint/tests; capture results.
3. Update `tasks.json` + `session_log.md` END entry with commands/outcomes.
4. Commit doc/task/log updates (`git commit -am "docs: finish R2d-integ"`), remove worktree, hand off.
