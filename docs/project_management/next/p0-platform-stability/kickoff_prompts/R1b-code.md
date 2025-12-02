# Task R1b-code (Replay verbose scopes & warnings) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R1a outputs, and this prompt.
3. Set `R1b-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1b-verbosity-code
   git worktree add wt/ps-r1b-verbosity-code ps-r1b-verbosity-code
   cd wt/ps-r1b-verbosity-code
   ```

## Spec
- Add a concise `scopes: [..]` line that prints next to the world strategy block whenever `--replay-verbose` (or equivalent JSON flag) is active; ensure JSON responses include the scopes array too.
- Differentiate warning prefixes so shell path warnings explicitly say “shell world-agent path …” while replay warnings keep the `[replay]` prefix.
- Update docs (`REPLAY.md`, `TRACE.md`, CLI help) describing the new verbose scopes line and warning semantics.

## Scope & Guardrails
- Production CLI/replay code + docs only; tests belong to R1b-test.
- Guard verbose output behind the existing flag so default behavior stays unchanged.
- Maintain Windows/macOS examples in docs (PowerShell/cmd) when updating usage text.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-replay -- -D warnings
cargo test -p substrate-replay -- --nocapture
```
Capture at least one manual `substrate --replay --replay-verbose ...` run or note why it was skipped.

## End Checklist
1. Ensure fmt/clippy/tests/manual commands complete; document skips.
2. Commit changes (e.g., `feat: add replay verbose scopes output`).
3. Merge `ps-r1b-verbosity-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing command results and manual smoke.
5. Confirm R1b-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1b-code"`), remove worktree, hand off.
