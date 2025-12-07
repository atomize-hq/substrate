# Task R2f-code (Host replay span parity) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2f-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2f-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2f-host-replay-code
   git worktree add wt/ps-r2f-host-replay-code ps-r2f-host-replay-code
   cd wt/ps-r2f-host-replay-code
   ```

## Spec
- Ensure host-only executions (`--no-world`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, policy host paths) still emit shim spans with `span_id` + replay_context (`execution_origin=host`) so they are replayable.
- Keep host-only runs from probing the agent/socket/world backend; preserve policy decisions and telemetry fields with backward-compatible trace schema.
- Replay must consume host spans and default to host execution while honoring existing overrides (`--world/--no-world`, `--flip-world`) with clear single-shot warnings when forced.

## Scope & Guardrails
- Touch production code in shell/shim/replay/trace; test-only helpers belong in R2f-test.
- Avoid duplicate span emission; async REPL behavior must remain intact.
- Do not regress world-mode replay or socket activation flows; host-only path should remain lightweight when the agent/socket is absent.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell replay_world
cargo test -p substrate-replay -- --nocapture
```

## End Checklist
1. Ensure fmt/clippy/tests completed (note skips).
2. Commit code/doc changes.
3. Merge `ps-r2f-host-replay-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm R2f-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2f-code"`), remove worktree, hand off.
