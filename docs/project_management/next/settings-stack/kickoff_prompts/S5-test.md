# Task S5-test (Anchor naming + caged guard) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Update `tasks.json` (set `S5-test` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S5-test"`).
3. Create the task branch/worktree:
   ```
   git checkout -b ss-s5-anchor-test
   git worktree add wt/ss-s5-anchor-test ss-s5-anchor-test
   cd wt/ss-s5-anchor-test
   ```
4. Ensure `git status` is clean before editing tests.

## Scope
- Add tests that cover anchor naming (`anchor_mode`/`anchor_path`) with backward compatibility for `root_*` keys/env/flags and precedence (flag > dir config > global config > env > default).
- Add tests for caged guard behavior on complex commands (e.g., chained `cd`) when world is disabled/unavailable and when enabled.
- Update installer smoke expectations if metadata/env exports change due to anchor naming.
- Keep production code changes minimal (test scaffolding only if needed).

## Commands
Run from the worktree:
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell caged
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure commands above pass; capture outputs/temp roots in the session log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands/results/blockers.
4. Create/confirm the S5-integ kickoff prompt path in the log.
