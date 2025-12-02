# Task R1c-code (Replay world coverage) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R1b outputs, and this prompt.
3. Set `R1c-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1c-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1c-coverage-code
   git worktree add wt/ps-r1c-coverage-code ps-r1c-coverage-code
   cd wt/ps-r1c-coverage-code
   ```

## Spec
- Expose replay world toggles (default on, `--no-world`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`) to the CLI harness/tests without editing global configs.
- Ensure CLI help/docs explain how these toggles interact and log new fields when verbose mode enabled.
- Capture manual smoke commands demonstrating world-on vs no-world runs.

## Scope & Guardrails
- Production CLI/config code + docs only; tests belong to R1c-test.
- Preserve backward compatibility for existing flags/environment variables.
- Document any limitations for macOS/WSL vs Linux.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell replay_world
```
Record manual `substrate --replay ...` invocations.

## End Checklist
1. Ensure fmt/clippy/tests/manual commands complete; note skips.
2. Commit changes (e.g., `feat: expose replay world toggles to CLI`).
3. Merge `ps-r1c-coverage-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm R1c-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1c-code"`), remove worktree, hand off.
