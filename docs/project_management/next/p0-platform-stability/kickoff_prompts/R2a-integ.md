# Task R2a-integ (Replay agent-backed execution) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2a-code/test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2a-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2a-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2a-replay-agent-integ
   git worktree add wt/ps-r2a-replay-agent-integ ps-r2a-replay-agent-integ
   cd wt/ps-r2a-replay-agent-integ
   ```

## Scope
- Merge ps-r2a-replay-agent-code/test, resolve conflicts across replay/shell/world-agent/docs.
- Ensure replay defaults to agent-backed execution with clear fallbacks; verify warning/telemetry output and doc updates from code/test branches.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Document any platform skips or manual `substrate --replay ...` validation.

## End Checklist
1. Merge code/test branches, resolve conflicts, fast-forward to feat/p0-platform-stability-follow-up.
2. Run required fmt/lint/tests; capture results.
3. Update `tasks.json` + `session_log.md` END entry (commands run, outcomes).
4. Commit doc/task/log updates (`git commit -am "docs: finish R2a-integ"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
