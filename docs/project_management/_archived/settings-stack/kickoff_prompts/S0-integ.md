# Task S0-integ (Integrate manifest bundling) – INTEGRATION

## Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Confirm S0-code (`feat: bundle manager manifest`) and S0-test (`test: verify bundled manifest`) are merged.
3. Update `tasks.json` (set `S0-integ` to `in_progress`) and add a START entry to the session log. Commit the doc-only change (`git commit -am "docs: start S0-integ"`).
4. Create the worktree:
   ```
   git worktree add wt/ss-s0-manifest-integ feat/settings-stack
   cd wt/ss-s0-manifest-integ
   ```
5. Ensure `git status` is clean before merging.

## Scope
- Bring S0 code + test commits together (release packaging now bundles `config/manager_hooks.yaml`; installer copies manifests under each version).
- Run the installer smoke harness for both scenarios to prove the manifests and health checks survive integration.
- Avoid new feature work; only resolve conflicts and wiring issues if they surface.

## Commands
Run the smoke harness from the worktree:
```
./tests/installers/install_smoke.sh --scenario default
./tests/installers/install_smoke.sh --scenario no-world
```
Optional (if you touch Rust code): `cargo fmt --all -- --check`

## End Checklist
1. Ensure both smoke scenarios pass (manager manifest present, `substrate health --json` succeeds) and note temp paths in the log.
2. Commit integration changes (if any), merge the worktree back to `feat/settings-stack`, and remove the worktree when done.
3. Update `tasks.json` (status → `completed`) and append an END entry to the session log with commands/results.
4. Verify the S1-code and S1-test kickoff prompts remain accurate; reference their paths in the log.
