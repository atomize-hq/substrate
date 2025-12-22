# Task R2d-test (Replay origin-aware defaults & agent routing) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2d-code scope, and this prompt.
3. Set `R2d-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2d-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2d-replay-origin-test
   git worktree add wt/ps-r2d-replay-origin-test ps-r2d-replay-origin-test
   cd wt/ps-r2d-replay-origin-test
   ```

## Spec
- Add fixtures that record world vs host spans and assert default replay follows the recorded origin; `--flip-world` (alias `--flip`) inverts it; `--world` / `--no-world` still override.
- Mock agent socket success/failure (Unix stub) to prove agent-first selection, single warning on failure, and fallback to local backend with preserved cwd/anchor/caging/env.
- Assert copy-diff fallback retries (/run, /tmp, /var/tmp) and `SUBSTRATE_COPYDIFF_ROOT` override with de-duped warning text and verbose strategy output.
- Capture verbose output expectations for origin/flip reason, selected strategy, endpoint, and copy-diff root.

## Required Commands
```
cargo fmt
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Record any manual `substrate --replay ...` runs used for debugging; note platform skips.

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit worktree changes (tests/fixtures).
3. Merge `ps-r2d-replay-origin-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include command results).
5. Confirm R2d-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2d-test"`), remove worktree, hand off.
