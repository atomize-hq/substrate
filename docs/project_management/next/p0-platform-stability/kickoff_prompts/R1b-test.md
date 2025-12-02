# Task R1b-test (Replay verbose scopes & warnings) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, the R1b-code scope, and this prompt.
3. Set `R1b-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1b-verbosity-test
   git worktree add wt/ps-r1b-verbosity-test ps-r1b-verbosity-test
   cd wt/ps-r1b-verbosity-test
   ```

## Spec
- Extend replay CLI tests to assert the `scopes: [...]` line (and JSON payload) appears only when `--replay-verbose` is active.
- Verify warning prefixes differ between shell and replay contexts, including JSON outputs/log fields.
- Update fixtures for PowerShell/Windows transcripts if they display the new line.

## Scope & Guardrails
- Tests/fixtures only; keep them deterministic.
- Avoid depending on actual system policies—mock scope metadata via fixture spans.
- Document any manual commands used to gather baseline outputs.

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Note any manual verbose replay runs in the session log.

## End Checklist
1. Ensure fmt/tests/manual commands completed; note skips.
2. Commit changes (e.g., `test: cover replay verbose scope output`).
3. Merge `ps-r1b-verbosity-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm R1b-integ prompt matches the suites upgraded.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1b-test"`), remove worktree, hand off.
