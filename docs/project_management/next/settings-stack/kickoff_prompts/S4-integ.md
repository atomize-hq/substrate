# Task S4-integ (Integrate force world override) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S4-code (`feat: force world override flag`) and S4-test (`test: force world override`) are merged.
3. Update `tasks.json` (set `S4-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S4-integ"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s4-world-override-integ
   git worktree add wt/ss-s4-world-override-integ ss-s4-world-override-integ
   cd wt/ss-s4-world-override-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Merge the force-world flag code + tests and resolve conflicts.
- Verify precedence/flag behavior across CLI/env/config/installer paths and caged/world-root handling.
- Keep changes integration-only unless fixes are needed to unblock tests.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy/tests and both installer scenarios pass; capture temp roots/results in the log.
2. Commit integration fixes (if any), merge the branch back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Push or hand off once `feat/settings-stack` reflects the merged changes.
