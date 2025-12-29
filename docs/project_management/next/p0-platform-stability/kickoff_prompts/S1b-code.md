# Task S1b-code (Socket activation – shell readiness) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1a outputs, and this prompt.
3. Set `S1b-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1b-shell-code
   git worktree add wt/ps-s1b-shell-code ps-s1b-shell-code
   cd wt/ps-s1b-shell-code
   ```

## Spec
- Build on the S1a agent changes (world-agent now consumes LISTEN_FDS/FD_START and emits socket-activation telemetry) so shell binaries respect the inherited sockets.
- Update `ensure_world_agent_ready`, `world_enable`, shim status, and related telemetry so the shell tolerates pre-existing sockets created by systemd.
- When `substrate world doctor --json` or `substrate --shim-status` detect socket activation, emit explicit messaging (text + JSON) describing the mode.
- Propagate a socket-activation flag through spans so CLI completions can differentiate the transport.

## Scope & Guardrails
- Production shell/world code and docs only; tests belong in S1b-test.
- Preserve behavior when systemd is absent—ensure new logic only activates when sockets exist or LISTEN_FDS data is available.
- Coordinate with telemetry schema (crates/common) if new fields are required; document them.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_enable
```
Capture any manual doctor/shim-status runs in the log (if privileges are missing, record `SKIP` with the reason).

## End Checklist
1. Ensure fmt/clippy/tests/manual checks completed; document skips.
2. Commit code/doc changes (e.g., `feat: handle socket-activated world agent readiness`).
3. Merge `ps-s1b-shell-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing command results and manual checks.
5. Confirm S1b-integ prompt is current (edit if new requirements emerged).
6. Commit doc/task/log updates (`git commit -am "docs: finish S1b-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
