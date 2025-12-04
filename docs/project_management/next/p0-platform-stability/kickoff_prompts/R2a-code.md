# Task R2a-code (Replay agent-backed execution) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, latest R1x outputs, and this prompt.
3. Set `R2a-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2a-replay-agent-code
   git worktree add wt/ps-r2a-replay-agent-code ps-r2a-replay-agent-code
   cd wt/ps-r2a-replay-agent-code
   ```

## Spec
- Default replay to agent-backed execution when `/run/substrate.sock` is healthy; only fall back to local backend/copy-diff when the agent is unavailable or explicitly disabled.
- Provide clear controls: default agent path; `--no-world`/`SUBSTRATE_REPLAY_USE_WORLD=disabled` for host-only; an explicit opt-in for local backend if needed. Emit a single warning when falling back.
- Preserve/forward world root + caging/env for replayed commands so cwd/path alignment matches live runs; update telemetry/log messages to indicate the chosen path and any ENOSPC/cgroup/netns issues (no spam).
- Update docs/help to describe replay path selection and the new toggles.

## Scope & Guardrails
- Code changes in replay/shell/world-agent as needed for agent-backed replay; tests live in R2a-test.
- Keep backward compatibility for existing flags/env; avoid changing default behavior beyond agent preference.
- Limit doc updates to replay/help/TRACE/WORLD notes.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-replay -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Record any manual `substrate --replay ...` smoke (agent available vs disabled).

## End Checklist
1. Ensure fmt/clippy/tests/manual replays completed; note skips.
2. Commit changes (e.g., `feat: prefer agent-backed replay`).
3. Merge `ps-r2a-replay-agent-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm R2a-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2a-code"`), remove worktree, hand off.
