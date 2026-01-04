# Task R2c-code (Replay coverage polish) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2b-integ outputs, and this prompt.
3. Set `R2c-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2c-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2c-replay-coverage-code
   git worktree add wt/ps-r2c-replay-coverage-code ps-r2c-replay-coverage-code
   cd wt/ps-r2c-replay-coverage-code
   ```

## Spec
- Polish replay CLI help/docs to describe agent vs host-only vs local fallback (include warning samples). Origin/flip and agent-first schema updates land in R2d.
- Update TRACE/WORLD docs + telemetry references so existing replay warnings from R2b are documented.
- Capture manual `substrate --replay --replay-verbose ...` runs showing agent-on vs `--no-world`; log outputs in session log.
- Ensure warning text added in R2b matches docs and CLI messaging.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell replay_world
```
Record manual `substrate --replay ...` samples (agent vs no-world).

## End Checklist
1. Ensure fmt/clippy/tests/manual replay demos completed; note skips.
2. Commit CLI/doc/telemetry updates.
3. Merge `ps-r2c-replay-coverage-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include commands run).
5. Confirm R2c-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2c-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
