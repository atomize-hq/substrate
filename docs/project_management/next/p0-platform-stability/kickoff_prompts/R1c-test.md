# Task R1c-test (Replay world coverage) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R1c-code scope, and this prompt.
3. Set `R1c-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1c-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1c-coverage-test
   git worktree add wt/ps-r1c-coverage-test ps-r1c-coverage-test
   cd wt/ps-r1c-coverage-test
   ```

## Spec
- Expand replay integration tests to run spans under three configurations: default world-on, `--no-world`, and `SUBSTRATE_REPLAY_USE_WORLD=disabled`.
- Assert warnings/verbose scope output (including the `scopes: [...]` line and shell vs replay prefixes) align with each mode (e.g., `[replay] world disabled via flag`).
- Update fixtures for JSON/text outputs touched by these toggles.

## Scope & Guardrails
- Tests/fixtures only; keep them deterministic.
- Use environment overrides to mimic capability limits rather than depending on actual root features.
- Document manual commands if used to capture expected output.

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Include any manual `substrate --replay` runs (world-on/off) in the log.

## End Checklist
1. Ensure fmt/tests/manual commands completed; note skips.
2. Commit changes (e.g., `test: cover replay world toggles`).
3. Merge `ps-r1c-coverage-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing commands/results.
5. Confirm R1c-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1c-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
