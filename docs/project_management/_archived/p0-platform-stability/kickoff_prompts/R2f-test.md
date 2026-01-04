# Task R2f-test (Host replay span parity) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2f-code scope, and this prompt.
3. Set `R2f-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2f-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2f-host-replay-test
   git worktree add wt/ps-r2f-host-replay-test ps-r2f-host-replay-test
   cd wt/ps-r2f-host-replay-test
   ```

## Spec
- Add fixtures that run host-only commands (`--no-world`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, policy host path) and assert spans include `span_id` + replay_context with `execution_origin=host`.
- Replay those host spans and verify they execute on the host without agent/world socket probes; warnings must be single-shot when overrides force world usage.
- Cover async REPL and non-PTY replay entrypoints; gate expectations when CI lacks world sockets or env differs.

## Scope & Guardrails
- Tests/fixtures only; keep production logic in R2f-code.
- Avoid brittle path/socket assertions; skip or soften when host prerequisites are missing.
- Keep warning expectations consistent with R2b/R2c dedupe behavior.

## Required Commands
```
cargo fmt
cargo test -p substrate-shell replay_world
cargo test -p substrate-replay -- --nocapture
```

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit test/fixture updates.
3. Merge `ps-r2f-host-replay-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm R2f-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2f-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
