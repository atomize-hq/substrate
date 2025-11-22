# Task S1-integ (Integrate TOML install config) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S1-code (`feat: migrate install config to toml`) and S1-test (`test: cover toml install config`) are merged.
3. Update `tasks.json` (set `S1-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S1-integ"`).
4. Create the worktree:
   ```
   git worktree add wt/ss-s1-config-integ feat/settings-stack
   cd wt/ss-s1-config-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Merge the TOML install config code + test branches and resolve any conflicts.
- Verify the CLI + installer write/read `config.toml` with `[install].world_enabled` and keep manager env exports intact.
- Keep changes integration-only: avoid new feature work unless needed to unstick the merge.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_enable
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy/tests and both installer scenarios pass; note temp install roots/results in the log.
2. Commit integration fixes (if any), merge the worktree back to `feat/settings-stack`, and remove the worktree when done.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Confirm the S2-code and S2-test kickoff prompts exist (paths below) and reference them in the log.
