# Task S1-code (Migrate install config to TOML) – CODE

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Read `settings_stack_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Update `tasks.json` (set S1-code to `in_progress`) and add a START entry to
   the session log. Commit the doc-only change (`git commit -am "docs: start S1-code"`).
4. Create the worktree:
   ```
   git worktree add wt/ss-s1-config-code feat/settings-stack
   cd wt/ss-s1-config-code
   ```
5. Confirm `git status` is clean before editing.

## Scope
- Replace `~/.substrate/config.json` with TOML serialization:
  - Update `crates/shell/src/commands/world_enable.rs` to read/write
    `config.toml` (new parser/serializer with `[install] world_enabled`).
  - Update installer/uninstaller (`scripts/substrate/install-substrate.sh`,
    `scripts/substrate/uninstall.sh`) to create/update the TOML file.
  - Ensure manager env exports (`SUBSTRATE_WORLD`, `SUBSTRATE_WORLD_ENABLED`)
    still derive from the new metadata.
- Refresh docs referencing the old JSON file (`docs/INSTALLATION.md`,
  `docs/CONFIGURATION.md`, `docs/UNINSTALL.md`, any other mention).
- Do **not** modify test files; limit yourself to production code + docs.

## Commands
Run as needed from the worktree (examples):
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_enable   # optional smoke if required
```
Keep test runs minimal; no new tests should be authored in this task.

## End Checklist
1. Ensure `cargo fmt` and `cargo clippy -p substrate-shell -- -D warnings` succeed.
2. Commit worktree changes (e.g., `feat: migrate install config to toml`).
3. Return to the repo root and merge onto `feat/settings-stack`
   (`git checkout feat/settings-stack && git merge --ff-only wt/ss-s1-config-code`).
4. Update `tasks.json` (status → `completed`) and append an END entry to the
   session log summarizing commands, results, and any blockers.
5. Ensure the S1-test kickoff prompt lives at
   `docs/project_management/next/settings-stack/kickoff_prompts/S1-test.md`,
   reference it in the session log, and note any special guidance for the test
   agent.
6. Commit doc updates on `feat/settings-stack`
   (`git commit -am "docs: finish S1-code"`).
7. Remove the worktree if finished (`git worktree remove wt/ss-s1-config-code`).
