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
- Add a policy flag (global + per-project) for world filesystem mode (`read_only` vs `writable`) with default writable.
- Broker validates the flag and surfaces clear errors for invalid values.
- Shell threads the resolved policy into PTY + non-PTY world requests and records the active mode in traces/doctor output.
- World-agent/backends honor the flag (read-only path mounts without upper/copy-diff; writable path matches today's behavior) and emit single-shot warnings when writes fail under read-only mode.
- Docs (CONFIGURATION/WORLD, WORLD/REPLAY) describe the knob and mention that installers/systemd must permit `/home` writes so policy can take effect.

## Scope & Guardrails
- Touch broker, shell, world-agent/backends, and docs; test-only work belongs in R2e-test.
- Existing CLI/env precedence stays intact (`--world`/`--no-world` override policy when provided).
- Both PTY and non-PTY paths must honor the policy.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_enable
cargo test -p world-agent   # document skips if privileges lacking
```

## End Checklist
1. Ensure fmt/clippy/tests completed (note skips).
2. Commit code/doc updates.
3. Merge `ps-r2e-world-fs-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm R2e-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2e-code"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
