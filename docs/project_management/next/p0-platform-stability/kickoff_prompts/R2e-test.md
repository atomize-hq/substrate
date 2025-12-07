# Task R2e-test (Policy-driven world fs mode) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, R2e-code scope, and this prompt.
3. Set `R2e-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2e-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2e-world-fs-test
   git worktree add wt/ps-r2e-world-fs-test ps-r2e-world-fs-test
   cd wt/ps-r2e-world-fs-test
   ```

## Spec
- Add fixtures that exercise both policy modes:
  - Read-only: confirm writes fail (PTY + non-PTY), doctor/trace reflect read-only, warnings are single-shot.
  - Writable: confirm writes succeed via overlay/copy-diff, telemetry shows writable, and agent/local fallbacks respect the policy.
- Ensure PTY/replay helpers reuse existing harnesses; gate assertions on host capabilities (overlay/cgroup). Document skips where necessary.
- Verify `--world`/`--no-world` overrides still take precedence when policy differs.

## Required Commands
```
cargo fmt
cargo test -p substrate-shell world_enable
cargo test -p world-agent   # or document skips if privileged requirements block
```

## End Checklist
1. Ensure fmt/tests completed; document skips with justification.
2. Commit test/fixture updates.
3. Merge `ps-r2e-world-fs-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include command results).
5. Confirm R2e-integ prompt contents.
6. Commit doc/task/log updates (`git commit -am "docs: finish R2e-test"`), remove worktree, hand off.
