# Task S1d-test (Installer socket-activation parity) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1d-code scope, and this prompt.
3. Set `S1d-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1d-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1d-devinstall-test
   git worktree add wt/ps-s1d-devinstall-test ps-s1d-devinstall-test
   cd wt/ps-s1d-devinstall-test
   ```

## Spec
- Extend `tests/installers/install_smoke.sh` (or a sibling harness) with dev and production scenarios that:
  - Verify `/run/substrate.sock` ownership (`root:substrate 0660`) after each installer runs (or capture sudo output for curl-based install).
  - Confirm the invoking user belongs to the `substrate` group (parse `id` output or env metadata) or log a warning when installing as root.
  - Capture the lingering guidance emitted by each installer and note whether `loginctl enable-linger` was already set.
- Update fixtures/log capture so the smoke script can record “skipped” output on non-systemd platforms.
- Document any platform constraints (e.g., containerized CI environments) inside the session log.

## Scope & Guardrails
- Tests/fixtures/scripts only; do not modify production installers beyond helper hooks.
- Keep the harness hermetic: if running on a host without systemd, record a skip with reasoning rather than forcing privileged operations.
- Update docs/test READMEs if the harness behavior changes.

## Required Commands
```
cargo fmt
./tests/installers/install_smoke.sh --scenario dev
./tests/installers/install_smoke.sh --scenario prod   # or equivalent production-focused run
```
If the smoke script can’t run on the current machine, record the skip and rationale in the session log.

## End Checklist
1. Ensure fmt/tests/scripts completed; document skips with detail.
2. Commit changes (e.g., `test: verify dev installer socket activation`).
3. Merge `ps-s1d-devinstall-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry with command results.
5. Confirm `S1d-integ` prompt reflects any new suites.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1d-test"`), remove worktree, hand off.
