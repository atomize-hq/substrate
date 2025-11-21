# Task S1-test (Test TOML install config) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Read `settings_stack_plan.md`, `tasks.json`, the latest `session_log.md`, and this prompt.
3. Update `tasks.json` (set `S1-test` to `in_progress`) and add a START entry to
   the session log. Commit the doc-only change (`git commit -am "docs: start S1-test"`).
4. Create the worktree:
   ```
   git worktree add wt/ss-s1-config-test feat/settings-stack
   cd wt/ss-s1-config-test
   ```
5. Confirm `git status` is clean before editing.

## Scope
- Add/adjust tests that exercise the new TOML install config:
  - Unit tests in `crates/shell/src/commands/world_enable.rs` covering
    load/save (valid/invalid/missing keys, extras preserved).
  - Extend `tests/installers/install_smoke.sh` (and/ or related fixtures) to
    assert the installer writes `config.toml` with correct `[install]` payload
    for default and `--no-world` paths.
- Do **not** modify production code apart from tiny test-only helpers.
- Keep runtime commands minimal; you may run targeted tests directly related to
  this task.

## Commands (examples)
```
cargo fmt
cargo test -p substrate-shell world_enable
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```
Run only what is required to validate the new tests.

## End Checklist & Follow-ups
1. Ensure `cargo fmt` and your targeted tests pass; capture command output for the END log entry.
2. Commit test changes in the worktree (e.g., `test: cover config toml`).
3. Return to `feat/settings-stack` and merge the worktree.
4. Update `tasks.json` (status → `completed`) and append an END entry to the session log.
5. Author the following kickoff prompts (place them under
   `docs/project_management/next/settings-stack/kickoff_prompts/` and mention paths in the log):
   - `S1-integ`
   - `S2-code`
   - `S2-test` (note in the log that S2 tasks cannot begin until S1-integ merges)
6. Commit the doc updates (`git commit -am "docs: finish S1-test + prompts"`).
7. Remove the worktree (`git worktree remove wt/ss-s1-config-test`) and hand off to the integration agent.
