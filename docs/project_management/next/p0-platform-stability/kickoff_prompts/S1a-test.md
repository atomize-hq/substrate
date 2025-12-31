# Task S1a-test (Socket activation – agent plumbing) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `S1a-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1a-agent-test
   git worktree add wt/ps-s1a-agent-test ps-s1a-agent-test
   cd wt/ps-s1a-agent-test
   ```

## Spec
- Author unit tests that mock LISTEN_FDS env/descriptor hand-offs, including multiple sockets and FD offset handling.
- Provide integration coverage (or harness-based simulations) ensuring the direct-bind fallback still works when LISTEN_FDS unset.
- Validate telemetry/log helpers emit the socket-activation flag so structured spans include the mode indicator.

## Scope & Guardrails
- Limit edits to tests, fixtures, and sanctioned helper hooks needed for injection.
- Keep tests hermetic: simulate descriptors via Unix domain sockets/pipe pairs rather than touching real systemd.
- Document any platform-specific skips (e.g., WSL/lima) in the session log with rationale.

## Required Commands
```
cargo fmt
cargo test -p world-agent
```
Add any helper scripts or harness invocations you run to the session log.

## End Checklist
1. Ensure fmt/tests (and any supplemental commands) completed; note skips with justification.
2. Commit changes (e.g., `test: cover world-agent LISTEN_FDS activation`).
3. Merge `ps-s1a-agent-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` with END entry describing command results/skips.
5. Verify S1a-integ prompt remains accurate; update only if new requirements surfaced.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1a-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
