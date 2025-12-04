# Task R2b-code (Replay fallbacks & warnings) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2a-code outputs, and this prompt.
3. Set `R2b-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2b-replay-fallback-code
   git worktree add wt/ps-r2b-replay-fallback-code ps-r2b-replay-fallback-code
   cd wt/ps-r2b-replay-fallback-code
   ```

## Spec
- Emit a single warning when replay falls back from agent → local backend (include cause and next steps). No repeated lines per command.
- Enhance copy-diff fallback: retry alternate scratch roots (/run, /tmp, /var/tmp) with clear logging; support overrides via `SUBSTRATE_COPYDIFF_ROOT`.
- Log/telemetry the chosen fallback path so traces show whether agent/local/copy-diff was used and why.
- Update docs/help to describe warning wording, copy-diff root overrides, and manual cleanup guidance (origin/flip comes in R2d).

## Scope & Guardrails
- Code changes in replay/shell/world-agent for warning/fallback plumbing + doc updates. Tests handled in R2c-test.
- Keep existing flags/env compatible; do not change the default agent preference from R2a.
- Ensure warnings respect verbose mode (still `[replay] warn:` style).

## Required Commands
```
cargo fmt
cargo clippy -p substrate-replay -- -D warnings
cargo test -p substrate-replay -- --nocapture
```
Record any manual `substrate --replay ...` showcasing fallback/warnings.

## End Checklist
1. Ensure fmt/clippy/tests/manual warnings replay completed; note skips.
2. Commit changes (e.g., `feat: improve replay fallback warnings`).
3. Merge `ps-r2b-replay-fallback-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm R2c-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2b-code"`), remove worktree, hand off.
