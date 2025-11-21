# Task S3-code (Implement caged root guard) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm `S2-integ` is merged (caged work waits for the current baseline).
3. Update `tasks.json` (set `S3-code` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S3-code"`).
4. Create the task branch + worktree:
   ```
   git checkout -b ss-s3-caged-code
   git worktree add wt/ss-s3-caged-code ss-s3-caged-code
   cd wt/ss-s3-caged-code
   ```
5. Ensure `git status` is clean before editing.

## Scope
- Replace the world-root flags with a standalone `caged` boolean (`--caged` / `--uncaged`) that anchors the shell to a root even when isolation is disabled.
- Keep the existing precedence stack (flag → dir config `.substrate/settings.toml` → global `~/.substrate/config.toml` → env → default).
- Enforce the anchor locally: attempts to leave the root emit an informational warning and bounce back, regardless of world enablement.
- Update docs/config references to the renamed setting and keep installer metadata aligned.

## Commands
Run what you change requires (examples):
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root         # adjust/target as needed
# Optional: ./tests/installers/install_smoke.sh --scenario default
# Optional: ./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy and any targeted tests pass; capture outputs for the session log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree when done.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log summarizing commands/results.
4. Verify the S3-test kickoff prompt path is recorded for the test agent.
