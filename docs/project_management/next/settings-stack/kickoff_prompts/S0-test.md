# Task S0-test (Test bundled manager manifest) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Read `settings_stack_plan.md`, `tasks.json`, latest `session_log.md`, and this prompt.
3. Set `S0-test` to `in_progress` in `tasks.json` and add a START entry to the session log. Commit the doc update (`git commit -am "docs: start S0-test"`).
4. Create the worktree:
   ```
   git worktree add wt/ss-s0-manifest-test feat/settings-stack
   cd wt/ss-s0-manifest-test
   ```

## Scope
- Extend tests/harness to verify manifests are present after install:
  - Enhance `tests/installers/install_smoke.sh` (and related fixtures) to assert
    `config/manager_hooks.yaml` exists under each scenario's prefix
    (default + `--no-world`).
  - Add a health/doctor smoke check (e.g., invoke the installed `substrate`
    binary with `--no-world` pointing at the temporary prefix/shims) to ensure
    `substrate health --json` no longer fails.
- Only modify test files/scripts. Production logic for bundling was done in S0-code.

## Suggested Commands
```
cargo fmt
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
# If you add a dedicated smoke harness for health, run it and note outputs
```
Document all commands in the END log entry.

## End Checklist & Follow-ups
1. Ensure harness/test runs pass; capture outputs (paths to temp dirs, etc.).
2. Commit test changes (`git commit -am "test: verify bundled manifest"`).
3. Merge results back to `feat/settings-stack`.
4. Update `tasks.json` (status → `completed`) and append an END entry to the session log summarizing commands/results.
5. Author the following kickoff prompts in `docs/project_management/next/settings-stack/kickoff_prompts/` and reference their paths in the log:
   - `S0-integ`
   - Updated `S1-code` and `S1-test` prompts (ensure they reflect any new prerequisites from S0)
6. Commit the doc updates (`git commit -am "docs: finish S0-test + prompts"`).
7. Remove the worktree (`git worktree remove wt/ss-s0-manifest-test`) and hand off to the integration agent.

## Deliverables
- Updated installer tests verifying config manifests exist and health succeeds post-install.
- Kickoff prompts ready for `S0-integ`, `S1-code`, and `S1-test`.
- Session log entry linking to all relevant prompts and test outputs.
