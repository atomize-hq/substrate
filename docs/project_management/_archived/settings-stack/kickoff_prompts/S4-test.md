# Task S4-test (Test force world override) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Set `S4-test` to `in_progress` in `tasks.json`, add a START entry to the session log, and commit the doc update (`git commit -am "docs: start S4-test"`).
3. Create the task branch/worktree:
   ```
   git checkout -b ss-s4-world-override-test
   git worktree add wt/ss-s4-world-override-test ss-s4-world-override-test
   cd wt/ss-s4-world-override-test
   ```
4. Ensure `git status` is clean before editing tests.

## Scope
- Add tests confirming `--world` overrides disabled install/config/env to force world isolation, while `--no-world` still disables it.
- Cover flag/config/env precedence and interactions with caged/world-root handling.
- Update installer smoke assertions if metadata/env exports change for the new flag.

## Commands
Run from the worktree:
```
cargo fmt
cargo test -p substrate-shell world_root
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/tests and both installer scenarios pass; record temp roots in the log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Create/confirm the S4-integ kickoff prompt path and reference it in the log.
