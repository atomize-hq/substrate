# Task S2-test (Test settings stack) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm `S1-integ` is merged (S2 tasks wait for integration). Coordinate with the S2-code agent as needed.
3. Update `tasks.json` (set `S2-test` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S2-test"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s2-settings-test
   git worktree add wt/ss-s2-settings-test ss-s2-settings-test
   cd wt/ss-s2-settings-test
   ```
5. Ensure `git status` is clean before editing tests.

## Scope
- Add tests covering the world root settings stack precedence (flag → dir config → global config → env → default) and the project/follow-cwd/custom modes.
- Extend installer/tests to assert `config.toml` carries the new world root keys where appropriate.
- Avoid production code changes except tiny test-only helpers.

## Commands
Keep runtime minimal (examples):
```
cargo fmt
cargo test -p substrate-shell world_enable            # target updated unit/integration coverage
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt and targeted tests/scripts pass; capture command outputs for the session log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree when finished.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log (include commands, results, blockers).
4. Create the S2-integ kickoff prompt if missing and reference all prompts in the log.
