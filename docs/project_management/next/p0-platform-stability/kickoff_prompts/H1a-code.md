# Task H1a-code (Health manager parity – aggregation logic) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R1 logs, and this prompt.
3. Set `H1a-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1a-health-code
   git worktree add wt/ps-h1a-health-code ps-h1a-health-code
   cd wt/ps-h1a-health-code
   ```

## Spec
- Refactor manager detection data structures to capture host/world presence + version info per manager (direnv, asdf, conda, pyenv, etc.).
- Update the health aggregator to compute severity (“attention required” only when host has a manager and world is missing) and provide structured telemetry.
- Keep legacy JSON outputs backward compatible while adding new fields described in docs.

## Scope & Guardrails
- Production code/docs only; tests handled in H1a-test.
- Avoid removing existing detection hooks; extend them to report structured state.
- Ensure null-handling remains safe when managers unsupported on a platform.
- R1c locked down replay world toggles + verbose `[replay] warn:` output—keep those CLI behaviors intact while updating health telemetry/docs and reference them when explaining world-vs-host assumptions.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell health
```
Record any manual `substrate health --json` runs used for validation.

## End Checklist
1. Ensure fmt/clippy/tests/manual commands completed; note skips.
2. Commit changes (e.g., `feat: refine health manager aggregation`).
3. Merge `ps-h1a-health-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm H1a-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish H1a-code"`), remove worktree, hand off.
