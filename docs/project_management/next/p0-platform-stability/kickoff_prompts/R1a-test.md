# Task R1a-test (Replay isolation fallback) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R1a-code scope, and this prompt.
3. Set `R1a-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1a-isolation-test
   git worktree add wt/ps-r1a-isolation-test ps-r1a-isolation-test
   cd wt/ps-r1a-isolation-test
   ```

## Spec
- Add tests that simulate nft unavailable scenarios, asserting the fallback logic triggers and diagnostics/log text match expectations.
- Cover cleanup helper behavior by stubbing leftover namespaces/rules and verifying detection output.
- Ensure standard isolation remains untouched when nft succeeds (regression tests).

## Scope & Guardrails
- Tests/fixtures only; no production logic changes beyond helper hooks.
- Keep tests deterministic—mock nft command outputs rather than requiring root privileges.
- Document any manual commands requiring elevated rights and whether they were skipped.

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world   # if applicable / note skip
```

## End Checklist
1. Ensure fmt/tests completed; note skips with justification.
2. Commit changes (e.g., `test: add replay nft fallback coverage`).
3. Merge `ps-r1a-isolation-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing test results.
5. Confirm R1a-integ prompt matches the suites you touched.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1a-test"`), remove worktree, hand off.
