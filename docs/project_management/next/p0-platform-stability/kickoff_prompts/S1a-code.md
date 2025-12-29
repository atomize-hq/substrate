# Task S1a-code (Socket activation – agent plumbing) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `S1a-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1a-agent-code
   git worktree add wt/ps-s1a-agent-code ps-s1a-agent-code
   cd wt/ps-s1a-agent-code
   ```

## Spec
- Update `world-agent` so it consumes inherited sockets via LISTEN_FDS/FD_START and skips binding when descriptors already provided.
- Preserve the legacy direct-bind behavior for non-systemd hosts; the CLI flags/config remain unchanged.
- Emit structured telemetry/log lines indicating whether the agent started via socket activation or manual bind, including FD counts.
- Ensure both Linux and guest backends (Lima/WSL) honor the new path without regressions.

## Scope & Guardrails
- Production code + docs only; tests belong to S1a-test.
- Do not remove the existing bind configuration—add detection/fallback logic instead.
- Keep changes localized to world-agent crates/common helpers unless coordination with shell is strictly required (that work lives in S1b).

## Required Commands
```
cargo fmt
cargo clippy -p world-agent -- -D warnings
cargo test -p world-agent
```
Record outputs or skips in the session log.

## End Checklist
1. Ensure fmt/clippy/tests succeeded (note skips).
2. Commit changes (e.g., `feat: add LISTEN_FDS support to world-agent`).
3. Merge `ps-s1a-agent-code` into `feat/p0-platform-stability` (fast-forward).
4. Update `tasks.json` + `session_log.md` with END entry and mention command results.
5. Confirm S1a-integ prompt linkage (no edit expected unless requirements changed).
6. Commit doc/task/log updates (`git commit -am "docs: finish S1a-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
