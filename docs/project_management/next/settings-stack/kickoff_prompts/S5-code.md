# Task S5-code (Rename anchor + caged guard) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Update `tasks.json` (set `S5-code` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S5-code"`).
3. Create the task branch/worktree:
   ```
   git checkout -b ss-s5-anchor-code
   git worktree add wt/ss-s5-anchor-code ss-s5-anchor-code
   cd wt/ss-s5-anchor-code
   ```
4. Ensure `git status` is clean before coding.

## Scope
- Rename world root selectors to anchor naming (`anchor_mode`/`anchor_path`) across CLI/env/config/installer metadata while keeping backward-compatible parsing of `root_*` keys/envs/flags.
- Update docs/CLI help to explain anchor naming and compatibility.
- Fix caged enforcement so complex commands (e.g., `cd ../ && pwd`) still honor the anchor when world is disabled/unavailable.
- Keep changes to production code/docs only; tests land in S5-test.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure fmt/clippy/tests above pass; capture outputs in the session log.
2. Commit worktree changes and merge back to `feat/settings-stack`; remove the worktree when done.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands/results.
4. Create/confirm the S5-test kickoff prompt path in the log.
