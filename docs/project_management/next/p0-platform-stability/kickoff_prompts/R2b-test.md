# Task R2b-test (Replay fallbacks & warnings) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2b-code scope, and this prompt.
3. Set `R2b-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2b-replay-fallback-test
   git worktree add wt/ps-r2b-replay-fallback-test ps-r2b-replay-fallback-test
   cd wt/ps-r2b-replay-fallback-test
   ```

## Spec
- Add tests ensuring agent→local fallback warnings emit once per replay (with actionable text).
- Cover copy-diff scratch root retries and overrides (env/flag) to ensure logs show the chosen path.
- Exercise ENOSPC retry behavior and verify logging/telemetry matches spec; fixtures must simulate constrained scratch dirs (origin/flip handled in R2d).

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Capture any manual `substrate --replay ...` verification when debugging warnings (note skips).

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit worktree changes (tests/fixtures).
3. Merge `ps-r2b-replay-fallback-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include command results).
5. Confirm R2b-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2b-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
