# Task R2d-code (Replay origin-aware defaults & agent routing) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2a/R2b/R2c outputs, and this prompt.
3. Set `R2d-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2d-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2d-replay-origin-code
   git worktree add wt/ps-r2d-replay-origin-code ps-r2d-replay-origin-code
   cd wt/ps-r2d-replay-origin-code
   ```

## Spec
- Record execution origin (`host`/`world`) and transport on command_complete spans (trace schema + replay_context); greenfield schema so no compat shims required.
- Default replay to the recorded origin; add `--flip-world` (alias `--flip`) to invert it. `--world` / `--no-world` keep highest precedence.
- When world is selected, prefer the agent socket (/run/substrate.sock or recorded endpoint). On failure emit a single warning and fall back to the local backend (overlay/fuse/copy-diff) while preserving cwd/anchor/caging/env from the span.
- Verbose output/telemetry must show origin, flip reason, selected strategy + endpoint, and copy-diff root (including override env).
- Improve copy-diff fallback: include /run + /tmp + /var/tmp, honor `SUBSTRATE_COPYDIFF_ROOT`, and de-duplicate warnings.

## Scope & Guardrails
- Code touches replay/shell/world-agent/trace; tests/fixtures live in R2d-test.
- Keep existing flags/env compatible; new flag aliases allowed but avoid breaking current CLI.
- Keep warnings single-shot per replay; agent failure must not abort if local backend succeeds.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-replay -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
```
Capture any manual `substrate --replay --replay-verbose` runs (agent healthy vs missing, flip on/off).

## End Checklist
1. Ensure fmt/clippy/tests/manual replay (agent/host/flip) completed; note skips.
2. Commit schema/replay/shell/world-agent changes + docs.
3. Merge `ps-r2d-replay-origin-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm R2d-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2d-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
