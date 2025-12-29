# Task R1a-code (Replay isolation fallback) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1c outputs, and this prompt.
3. Set `R1a-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1a-isolation-code
   git worktree add wt/ps-r1a-isolation-code ps-r1a-isolation-code
   cd wt/ps-r1a-isolation-code
   ```

## Spec
- Implement the nft cgroup matching fallback + diagnostics inside replay/world backends: attempt primary rule path, then fallback with actionable messages when unavailable.
- Add cleanup helpers (CLI or library) that detect and optionally purge leftover netns/nft rules; log instructions for manual cleanup.
- Update REPLAY/WORLD docs to describe the fallback logic, cleanup commands, and capability requirements (Linux vs Lima vs WSL).

## Scope & Guardrails
- Production code/docs only; tests belong to R1a-test.
- Keep platform guards (`#[cfg]`) correct so unsupported OS builds don’t regress.
- Avoid enabling privileged cleanup commands automatically—expose them as opt-in diagnostics.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-replay -- -D warnings
cargo test -p substrate-replay -- --nocapture
```
Record any manual replay invocations or cleanup scripts run.

## End Checklist
1. Ensure fmt/clippy/tests/manual commands complete (note skips).
2. Commit changes (e.g., `feat: add replay nft fallback + diagnostics`).
3. Merge `ps-r1a-isolation-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry, summarizing command results and manual checks.
5. Confirm R1a-integ prompt still matches scope; edit if new steps required.
6. Commit doc/task/log updates (`git commit -am "docs: finish R1a-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
