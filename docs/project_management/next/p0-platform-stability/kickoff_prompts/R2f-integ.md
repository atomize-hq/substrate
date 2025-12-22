# Task R2f-integ (Host replay span parity) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2f-code/test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2f-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2f-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2f-host-replay-integ
   git worktree add wt/ps-r2f-host-replay-integ ps-r2f-host-replay-integ
   cd wt/ps-r2f-host-replay-integ
   ```

## Scope
- Merge R2f code/test branches, resolve conflicts across shell/shim/replay/trace/docs, and ensure host-only spans replay without world/socket dependencies.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```

## End Checklist
1. Merge code/test branches; resolve conflicts.
2. Run required fmt/lint/tests; capture output (note skips if world socket absent).
3. Fast-forward merge into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Commit doc/task/log updates (`git commit -am "docs: finish R2f-integ"`), remove worktree, hand off.
