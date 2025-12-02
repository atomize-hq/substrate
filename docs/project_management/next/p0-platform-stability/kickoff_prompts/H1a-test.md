# Task H1a-test (Health manager parity – aggregation logic) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, H1a-code scope, and this prompt.
3. Set `H1a-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1a-health-test
   git worktree add wt/ps-h1a-health-test ps-h1a-health-test
   cd wt/ps-h1a-health-test
   ```

## Spec
- Add unit/integration tests covering host/world combinations for each supported manager: host+world missing (OK), host present/world missing (attention), world present/host missing (degraded but not attention).
- Assert telemetry/JSON output includes the new structured fields.
- Update fixtures for CLI text output if severity verbiage changes.

## Scope & Guardrails
- Tests/fixtures only; keep detections mocked so real host tooling isn’t required.
- Use hermetic temp directories/environment overrides to mimic manager presence.
- Document any manual `substrate health` runs used for sample output.
- Preserve the R1c replay world coverage assumptions: the CLI already emits `[replay] world toggle` + warning lines when worlds are disabled, so your tests should avoid mutating those toggles outside of targeted health scenarios and keep fixtures aligned with the new verbose output.

## Required Commands
```
cargo fmt
cargo test -p substrate-shell health
```
Add any extra suites in the session log (e.g., world doctor harness if touched).

## End Checklist
1. Ensure fmt/tests/manual commands completed; note skips.
2. Commit changes (e.g., `test: cover refined health manager aggregation`).
3. Merge `ps-h1a-health-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm H1a-integ prompt stays accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish H1a-test"`), remove worktree, hand off.
