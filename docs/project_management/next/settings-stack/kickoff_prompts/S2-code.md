# Task S2-code (Implement settings stack & world root flag) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm `S1-integ` is merged (S2 tasks must wait until integration lands).
3. Update `tasks.json` (set `S2-code` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S2-code"`).
4. Create the task branch + worktree:
   ```
   git checkout -b ss-s2-settings-code
   git worktree add wt/ss-s2-settings-code ss-s2-settings-code
   cd wt/ss-s2-settings-code
   ```
5. Ensure `git status` is clean before editing.

## Scope
- Implement the settings stack + world root CLI flags (`--world-root-mode/--world-root-path`), honoring precedence: flag → directory config (`.substrate/settings.toml`) → global config (`~/.substrate/config.toml`) → env vars → default.
- Keep config parsing in the shell aligned with the new TOML install metadata and preserve manager env exports.
- Update docs (`CONFIGURATION.md`, `USAGE.md`, etc.) to describe the new settings stack and modes.
- Do **not** modify test files; test coverage belongs to S2-test.

## Commands
Run what you change requires (examples):
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_enable          # adjust/target as needed
# Optional sanity if installer surfaces: ./tests/installers/install_smoke.sh --scenario default
# Optional: ./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy and any targeted tests pass; capture outputs for the session log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree when done.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log summarizing commands/results.
4. Verify the S2-test kickoff prompt path is recorded in the log for the test agent.
