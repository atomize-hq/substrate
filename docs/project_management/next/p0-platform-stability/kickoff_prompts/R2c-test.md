# Task R2c-test (Replay agent-backed execution) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2a/R2b outputs, and this prompt.
3. Set `R2c-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2c-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2c-replay-agent-test
   git worktree add wt/ps-r2c-replay-agent-test ps-r2c-replay-agent-test
   cd wt/ps-r2c-replay-agent-test
   ```

## Spec
- Add tests for replay choosing the agent path when the socket probe succeeds (mock agent client or fixture). Assert telemetry/warning output, cwd/anchor propagation, and scope lines match expectations.
- Cover agent-unavailable fallback to local backend/copy-diff, including ENOSPC retry behavior and single warnings. Include fixtures for caged vs uncaged spans.
- Exercise `--no-world`/`SUBSTRATE_REPLAY_USE_WORLD=disabled` host-only mode to ensure logs/warnings skip agent/local noise.
- Update test docs/fixtures and ensure warnings remain concise (no duplicate lines).

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Document any additional manual `substrate --replay ...` used for assertions (note platform skips).

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit worktree changes (tests/fixtures).
3. Merge `ps-r2c-replay-agent-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include command results).
5. Confirm R2c-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2c-test"`), remove worktree, hand off.
