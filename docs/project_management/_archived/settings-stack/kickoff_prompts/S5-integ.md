# Task S5-integ (Anchor naming + caged guard) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S5-code (`Rename root to anchor + caged guard fix`) and S5-test (`Test anchor naming + caged guard`) are merged.
3. Update `tasks.json` (set `S5-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S5-integ"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s5-anchor-integ
   git worktree add wt/ss-s5-anchor-integ ss-s5-anchor-integ
   cd wt/ss-s5-anchor-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Merge the anchor renaming + caged guard code and tests; resolve conflicts.
- Verify compatibility and behavior across CLI/env/config/installer paths for anchor naming and caged enforcement on complex commands.
- Keep changes integration-only unless minimal fixes are needed to unblock tests.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell caged
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy/tests and both installer scenarios pass; capture temp roots/results in the log.
2. Commit integration fixes (if any), merge the branch back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Push or hand off once `feat/settings-stack` reflects the merged changes.
