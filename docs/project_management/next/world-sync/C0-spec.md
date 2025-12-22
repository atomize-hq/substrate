# C0-spec: Init & Gating

## Scope
- Introduce `substrate init` for workspaces:
  - Create `.substrate/` and `.substrate-git/` (with internal git scaffold) at the directory where init is run.
  - Seed default config (world enablement, sync defaults, ignores).
  - Idempotent: reuses existing state, reports what was created/updated.
- Gating world features:
  - When world is enabled (REPL or non-PTY/PTY world commands), require that `substrate init` has been run in the workspace (presence of `.substrate/` and valid metadata).
  - If not initialized, world mode is denied with a clear error; host-only mode remains available.
- Internal git readiness:
  - On init, create `.substrate-git/.git` and ensure it is ignored by the user repo.
  - Prepare for both host and world usage (world-side clone/init stub allowed to be lazy until C6).
- CLI/UX:
  - `substrate init` reports paths created, existing state, and next steps.
  - Provide a `--force` or `--reinit` flag to repair missing pieces without deleting user data.

## Acceptance
- `substrate init` creates `.substrate/` and `.substrate-git/` with internal git initialized, without touching user `.git`; reruns are safe and non-destructive.
- World entry (REPL and non-PTY/PTY world commands) is blocked until init succeeds; error message points to `substrate init`.
- Host-only mode still works without init.
- Default config written matches current defaults (world disabled/enabled as appropriate, sync defaults from C1).
- Internal ignores ensure `.substrate-git` is not added to user git.

## Out of Scope
- Actual sync behavior, auto-sync, host↔world application.
- Detailed internal git commit flows (handled in C6).
