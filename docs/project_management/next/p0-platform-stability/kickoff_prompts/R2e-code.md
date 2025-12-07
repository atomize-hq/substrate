# Task R2e-code (Policy-driven world fs mode) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2e-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2e-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2e-world-fs-code
   git worktree add wt/ps-r2e-world-fs-code ps-r2e-world-fs-code
   cd wt/ps-r2e-world-fs-code
   ```

## Spec
- Add a broker policy flag for world filesystem mode (`read_only` vs `writable`) with default = writable; validate inputs and allow global + per-project settings.
- Shell must include the resolved mode in world requests (PTY + non-PTY) and trace/doctor output so users can see the active mode and remediation guidance.
- World-agent/backends honor the flag: read-only path mounts without an upper/copy-diff (writes fail cleanly); writable path keeps today’s overlay/copy-diff flow.
- Update docs (CONFIGURATION/WORLD, WORLD/REPLAY references) to describe the knob and mention systemd baseline expectations (ProtectHome must permit policy to take effect).

## Scope & Guardrails
- Code touches broker, shell, world-agent/world backends, and docs; test-only work belongs in R2e-test.
- Keep existing env/flag precedence intact (policy should slot between CLI overrides and defaults).
- Ensure PTY/REPL commands follow the same policy as non-PTY; warn once when a write hits read-only mode.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_enable
cargo test -p world-agent   # or document skips if privileged requirements block
```

## End Checklist
1. Ensure fmt/clippy/tests completed (note skips).
2. Commit code/doc updates.
3. Merge `ps-r2e-world-fs-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm R2e-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2e-code"`), remove worktree, hand off.
