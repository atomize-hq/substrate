# Task S1c-test (Socket activation – provisioning & docs) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1c-code scope, and this prompt.
3. Set `S1c-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1c-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1c-provision-test
   git worktree add wt/ps-s1c-provision-test ps-s1c-provision-test
   cd wt/ps-s1c-provision-test
   ```

## Spec
- Extend installer/uninstaller harnesses (shell + PowerShell) to exercise the new `.socket` deployment/removal paths. Dry-run output is acceptable when sudo access is unavailable, but capture logs.
- Update world doctor/health integration tests (where feasible) to verify they report both `.service` and `.socket` states after provisioning/uninstall.
- Refresh any fixtures or docs describing the harness outputs.
- Mirror the S1b telemetry contract by asserting the `world_socket` JSON fields (`mode`, `path`, `socket_activation`) and shim status summaries emitted by the refreshed shell binaries.

## Scope & Guardrails
- Focus on automated tests/fixtures/scripts; do not modify production provisioning scripts except for helper hooks reviewed with S1c-code.
- Clearly annotate platform skips in session log (e.g., “macOS Lima unavailable on this host, skip recorded”).
- Ensure tests stay hermetic by pointing to temporary directories or mock systemctl invocations when possible.
- Pull expectations directly from the merged S1b shell suites when validating doctor/shim outputs so that behaviors stay consistent across code/test/provisioning phases.

## Required Commands
```
cargo fmt
./tests/installers/install_smoke.sh    # or document skip if platform constraints
scripts/linux/world-provision.sh --profile dev --dry-run
scripts/mac/lima-warm.sh --check-only
pwsh -File scripts/windows/wsl-warm.ps1 -WhatIf
```
Include additional doctor/health invocations if run.

## End Checklist
1. Confirm fmt/tests/scripts (or documented skips) completed.
2. Commit changes (e.g., `test: cover socket-activated provisioners`).
3. Merge `ps-s1c-provision-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing commands/skips.
5. Ensure S1c-integ prompt aligns with the suites you touched.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1c-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
