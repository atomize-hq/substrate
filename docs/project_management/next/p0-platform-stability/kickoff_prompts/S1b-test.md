# Task S1b-test (Socket activation – shell readiness) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1b-code scope, and this prompt.
3. Set `S1b-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1b-shell-test
   git worktree add wt/ps-s1b-shell-test ps-s1b-shell-test
   cd wt/ps-s1b-shell-test
   ```

## Spec
- Extend shell/world doctor integration tests to simulate socket activation (pre-created UDS) and confirm readiness succeeds only when the agent responds.
- Update shim status tests to expect the new socket-activation messaging (text + JSON).
- Add telemetry assertions verifying spans/log entries capture the activation flag.

## Scope & Guardrails
- Tests/fixtures/scripts only; do not modify production logic beyond helper hooks needed for dependency injection.
- Keep simulations hermetic by creating Unix sockets in temp dirs; avoid relying on systemd or sudo.
- Document any commands requiring privileges (world doctor, etc.) and justify skips.

## Required Commands
```
cargo fmt
cargo test -p substrate-shell world_enable
cargo test -p substrate-shell world_root
```
Add manual doctor/shim-status invocations (real or simulated) to the session log.

## End Checklist
1. Ensure fmt/tests/manual commands completed; note skips.
2. Commit changes (e.g., `test: ensure shell tolerates socket-activated world agent`).
3. Merge `ps-s1b-shell-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry detailing command outcomes.
5. Confirm S1b-integ prompt reflects any new suites.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1b-test"`), remove worktree, hand off.
