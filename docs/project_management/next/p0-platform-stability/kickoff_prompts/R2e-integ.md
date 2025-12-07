# Task R2e-integ (Policy-driven world fs mode) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm R2e-code/test completed; read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `R2e-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R2e-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r2e-world-fs-integ
   git worktree add wt/ps-r2e-world-fs-integ ps-r2e-world-fs-integ
   cd wt/ps-r2e-world-fs-integ
   ```

## Scope
- Merge ps-r2e-world-fs-code/test, resolve conflicts across broker/shell/world-agent/docs.
- Verify policy plumbing selects read-only vs writable modes consistently (PTY + non-PTY) and that doctor/trace/diff output reflects the mode.
- Ensure systemd socket/service hardening stays compatible with policy-controlled writes.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_enable
cargo test -p world-agent   # or document skips if privileged requirements block
```

## End Checklist
1. Merge code/test branches; resolve conflicts.
2. Run required fmt/lint/tests; record outputs (note skips).
3. Fast-forward merge into feat/p0-platform-stability-follow-up.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Commit doc/task/log updates (`git commit -am "docs: finish R2e-integ"`), remove worktree, hand off.
