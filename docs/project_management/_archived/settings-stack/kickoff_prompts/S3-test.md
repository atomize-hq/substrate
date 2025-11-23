# Task S3-test (Test caged root guard) – TEST

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm `S2-integ` is merged and coordinate with the S3-code agent.
3. Update `tasks.json` (set `S3-test` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S3-test"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s3-caged-test
   git worktree add wt/ss-s3-caged-test ss-s3-caged-test
   cd wt/ss-s3-caged-test
   ```
5. Ensure `git status` is clean before editing tests.

## Scope
- Add tests for the `caged` boolean: flag/config/env precedence and enforcement while world isolation is disabled and enabled.
- Verify the shell bounces back to the anchor with an informational warning when leaving the root.
- Extend installer/tests to assert `config.toml` (and dir settings) carry the `caged` setting.
- Avoid production code changes beyond tiny test helpers.

## Commands
Keep runtime minimal (examples):
```
cargo fmt
cargo test -p substrate-shell world_root           # adjust target as needed
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt and targeted tests/scripts pass; capture outputs for the session log.
2. Commit worktree changes, merge back to `feat/settings-stack`, and remove the worktree when finished.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log (include commands/results/blockers).
4. Create the S3-integ kickoff prompt if missing and reference all prompts in the log.
