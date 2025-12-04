# Task R2a-test (Replay agent-backed execution) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2a-code scope, and this prompt.
3. Set `R2a-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2a-replay-agent-test
   git worktree add wt/ps-r2a-replay-agent-test ps-r2a-replay-agent-test
   cd wt/ps-r2a-replay-agent-test
   ```

## Spec
- Add tests that exercise replay choosing the agent path when the socket is healthy (mock/probe), asserting telemetry/warning output matches spec.
- Cover agent-unavailable fallback to local backend/copy-diff with a single warning; include ENOSPC retry behavior.
- Ensure replay honors recorded cwd/anchor/caging/env so path alignment matches live runs; include fixtures for both caged and uncaged modes.
- Update test docs/fixtures for existing flags/env controls (no new flags yet).

## Scope & Guardrails
- Focus on replay/shell/world-agent tests and fixtures; code changes belong to R2a-code.
- Keep warnings concise (no repeated lines) and assert copy-diff retry behavior.
- Note platform skips for macOS/WSL if applicable.

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Capture any manual `substrate --replay ...` assertions if used.

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit worktree changes (tests/fixtures).
3. Merge `ps-r2a-replay-agent-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include command results).
5. Confirm R2a-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2a-test"`), remove worktree, hand off.
