# Task S1c-code (Socket activation – provisioning & docs) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, prior S1 logs, and this prompt.
3. Set `S1c-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1c-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1c-provision-code
   git worktree add wt/ps-s1c-provision-code ps-s1c-provision-code
   cd wt/ps-s1c-provision-code
   ```

## Spec
- Update platform installers/uninstallers (Linux `world-provision.sh`, macOS `lima-warm.sh`, Windows `wsl-warm.ps1`, plus helper scripts like `scripts/substrate/world-enable.sh`) to deploy/manage both `.service` and `.socket` units.
- Ensure idempotent reruns and uninstall flows clean up both unit types; document sudo/permissions guidance.
- Refresh docs (`docs/WORLD.md`, `docs/INSTALLATION.md`, platform runbooks) with socket-activation instructions and manual verification commands.
- Thread the S1b shell readiness outputs through docs: highlight the new `world_socket` block from `substrate world doctor --json` and the socket-activation summary in `substrate --shim-status` so operators know what to expect after provisioning.

## Scope & Guardrails
- Production scripts + documentation only; test harness updates belong to S1c-test.
- Keep platform-specific instructions accurate—note when commands require sudo/PowerShell elevation.
- Provide sample systemd unit content or link to the existing templates; avoid hard-coding host paths where not portable.
- Cross-check references against the S1b shell changes (`ensure_world_agent_ready`, doctor/shim outputs) instead of re-implementing readiness logic; docs should call out `SUBSTRATE_WORLD_SOCKET`/`socket_activation` terminology verbatim so later tests can assert them.

## Required Commands
```
cargo fmt   # run from repo root before final check-in
scripts/linux/world-provision.sh --profile dev --dry-run    # or document actual run/skips
scripts/mac/lima-warm.sh --check-only                       # record output/skip
pwsh -File scripts/windows/wsl-warm.ps1 -WhatIf             # or describe skip
```
Capture outputs/skips in the session log.

## End Checklist
1. Verify script changes + recorded command output.
2. Commit changes (e.g., `feat: add socket units to provisioners`).
3. Merge `ps-s1c-provision-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing command results.
5. Confirm S1c-integ prompt is accurate; edit if new requirements introduced.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1c-code"`), remove worktree, hand off.
