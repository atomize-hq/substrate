# Task S4-code (Force world override flag) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Update `tasks.json` (set `S4-code` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S4-code"`).
3. Create the task branch/worktree:
   ```
   git checkout -b ss-s4-world-override-code
   git worktree add wt/ss-s4-world-override-code ss-s4-world-override-code
   cd wt/ss-s4-world-override-code
   ```
4. Ensure `git status` is clean before coding.

## Scope
- Add a `--world` flag that forces world isolation for a single run even when install/config/env disables it; keep `--no-world` as the opt-out.
- Ensure flag precedence (flag > config/env) and alignment with caged/world-root handling.
- Update docs and installer metadata/env exports if needed to mention the override.

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
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands/results/temp roots.
4. Create/confirm the S4-test kickoff prompt path in the log.
