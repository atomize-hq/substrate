# Task S1e-code (Installer state tracking & cleanup) – CODE

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, S1d outputs, and this prompt.
3. Set `S1e-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1e-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1e-installer-code
   git worktree add wt/ps-s1e-installer-code ps-s1e-installer-code
   cd wt/ps-s1e-installer-code
   ```

## Spec
- Extend both installers (`dev-install-substrate.sh`, `install-substrate.sh`) to record host-state provenance (whether the `substrate` group existed before installing, which users were added, loginctl lingering state) in a metadata file under `~/.substrate` with schema/version info for future toggles.
- Update uninstall scripts to read that metadata and, when the operator opts in (new flag + future interactive hook), automatically remove recorded group memberships, delete the group only when no other members remain, and disable lingering if Substrate previously enabled it. When metadata is missing or indicates the feature was already on, fall back to the existing guidance-only behavior.
- Lay the groundwork for upcoming interactive install/uninstall UX: document the metadata file, describe the cleanup flag, and note how interactive prompts will surface these decisions.

## Scope & Guardrails
- No interactive prompts yet—just the underlying plumbing plus a CLI flag (e.g., `--auto-cleanup` / `--cleanup-state`) that the upcoming UX work can call.
- Metadata must be resilient to partial installs/uninstalls; corrupted or absent files should not prevent install/uninstall, merely downgrade to warnings.
- Keep installers idempotent: re-running should merge/refresh metadata without duplicating entries.
- Continue logging reminders when cleanup is skipped so existing scripts/tests don’t lose their current messaging.

## Required Commands
```
cargo fmt
shellcheck scripts/substrate/dev-install-substrate.sh scripts/substrate/install-substrate.sh scripts/substrate/dev-uninstall-substrate.sh scripts/substrate/uninstall-substrate.sh
```
Capture output (or note justified skips) in the session log.

## End Checklist
1. Ensure fmt/shellcheck completed; document any skips.
2. Commit worktree changes (e.g., `feat: track installer host state`).
3. Merge `ps-s1e-installer-code` into `feat/p0-platform-stability-follow-up`.
4. Update `tasks.json` + `session_log.md` END entry (include commands run).
5. Confirm/create the `S1e-integ` prompt if scope changed.
6. Commit doc/task/log updates (`git commit -am "docs: finish S1e-code"`), remove worktree, hand off.
