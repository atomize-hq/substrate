# Task S1d-code (Installer socket-activation parity) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `S1d-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1d-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1d-devinstall-code
   git worktree add wt/ps-s1d-devinstall-code ps-s1d-devinstall-code
   cd wt/ps-s1d-devinstall-code
   ```

## Spec
- Update both `scripts/substrate/dev-install-substrate.sh` and `scripts/substrate/install-substrate.sh` so they:
  - Ensure the `substrate` group exists (create if missing) and add the invoking user when running on Linux (or print guidance when running via curl installer as non-root).
  - Reload the socket/service units so `/run/substrate.sock` becomes `root:substrate 0660`, matching the provisioning scripts.
  - Print clear lingering guidance (`loginctl enable-linger <user>`) including detection of the current status.
- Update both uninstall scripts so they remove the units/socket cleanly and optionally remind operators about lingering/group cleanup.
- Refresh docs (`AGENTS.md`, `INSTALLATION.md`, `WORLD.md`) to call out the installer behavior and new requirements for both dev and production setups.

## Scope & Guardrails
- Production shell scripts + docs only; test harness updates handled by S1d-test.
- Keep the installer idempotent (re-running should not break existing installs).
- When touching documentation, mention both the group setup and lingering step so manual testers know what to expect.

## Required Commands
```
cargo fmt
shellcheck scripts/substrate/dev-install-substrate.sh scripts/substrate/dev-uninstall-substrate.sh scripts/substrate/install-substrate.sh scripts/substrate/uninstall-substrate.sh
```
Log shellcheck output (or justify skips) in the session log.

## End Checklist
1. Ensure fmt/shellcheck completed; note skips with justification.
2. Commit worktree changes (e.g., `feat: align dev installer with socket activation`).
3. Merge `ps-s1d-devinstall-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include commands run).
5. Confirm `S1d-integ` prompt remains accurate; edit if scope changed.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1d-code"`), remove worktree, hand off on `feat/p0-platform-stability-follow-up`.
