# Task S3-integ (Integrate caged root guard) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S3-code (`feat: implement caged root guard`) and S3-test (`test: caged root guard`) are merged.
3. Update `tasks.json` (set `S3-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S3-integ"`).
4. Create the task branch/worktree:
   ```
   git checkout -b ss-s3-caged-integ
   git worktree add wt/ss-s3-caged-integ ss-s3-caged-integ
   cd wt/ss-s3-caged-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Merge the caged guard code + tests and resolve conflicts.
- Verify anchor enforcement and metadata (caged boolean) across CLI/env/config/installer paths.
- Keep changes integration-only unless fixes are needed to unblock tests.

## Commands
Run from the worktree:
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```

## End Checklist
1. Ensure fmt/clippy/tests and both installer scenarios pass; capture temp roots/results in the log.
2. Commit integration fixes (if any), merge the branch back to `feat/settings-stack`, and remove the worktree.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands + outcomes.
4. Push or hand off once `feat/settings-stack` reflects the merged changes.
