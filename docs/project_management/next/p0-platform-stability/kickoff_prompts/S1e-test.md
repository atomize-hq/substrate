# Task S1e-test (Installer state tracking & cleanup) – TEST

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, updated `tasks.json`, `session_log.md`, S1e-code outputs, and this prompt.
3. Set `S1e-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1e-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1e-installer-test
   git worktree add wt/ps-s1e-installer-test ps-s1e-installer-test
   cd wt/ps-s1e-installer-test
   ```

## Spec
- Add tests/harness coverage that exercises the new installer metadata file (creation, upgrade/migration, missing/corrupt cases).
- Simulate multi-user entries so we verify cleanup flags only remove memberships/lingering for recorded users.
- Exercise the uninstall cleanup flag path (dry-run/mocked `loginctl` & `group` commands if needed) and ensure default mode still prints guidance without changes.
- Update test docs/log capture so future agents know how to run the harness on hosts without systemd.

## Scope & Guardrails
- Tests only; production script edits belong to S1e-code.
- Keep harnesses idempotent and safe on hosts without sudo/systemd—emit structured skips with reasoning.
- Capture all commands/skips in the session log.

## Required Commands
```
cargo fmt
<tests/installers commands you add>  # e.g., ./tests/installers/install_state_smoke.sh --scenario metadata
shellcheck <any new harness scripts>
```
Document exact commands or justified skips.

## End Checklist
1. Ensure fmt/tests/scripts completed (note skips with justification).
2. Commit worktree changes (e.g., `test: cover installer state metadata`).
3. Merge `ps-s1e-installer-test` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm `S1e-integ` prompt remains accurate; edit if new harness steps were added.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1e-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
