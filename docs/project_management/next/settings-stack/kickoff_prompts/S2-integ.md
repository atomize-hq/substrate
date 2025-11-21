# Task S2-integ (Integrate settings stack) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S2-code (`feat: add world root settings stack`) and S2-test (`test: settings stack`) are merged.
3. Update `tasks.json` (set `S2-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S2-integ"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s2-settings-integ
   git worktree add wt/ss-s2-settings-integ ss-s2-settings-integ
   cd wt/ss-s2-settings-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Merge the S2 settings stack code + tests (world root precedence, installer config updates) and resolve any conflicts.
- Verify world root modes/precedence across CLI/env/config files and that installer metadata keeps the `[world]` keys.
- Keep changes integration-only unless fixes are needed to unblock tests.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_enable
cargo test -p substrate-shell world_root
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy/tests and both installer scenarios pass; capture temp roots/results in the log.
2. Commit integration fixes (if any), merge the branch back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Push or hand off once `feat/settings-stack` reflects the merged changes.
