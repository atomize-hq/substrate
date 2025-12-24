# C0-spec: Init & Gating

## Scope
- Introduce `substrate init` for workspaces:
  - Create `.substrate/` and `.substrate-git/` (with internal git scaffold) at the directory where init is run.
  - Seed default config (world enablement, sync defaults, ignores).
  - Idempotent: reuses existing state, reports what was created/updated.
- Gating world-sync features:
  - `substrate sync`, `substrate checkpoint`, and `substrate rollback` must require that `substrate init` has been run in the workspace (presence of `.substrate/` and valid metadata).
  - If not initialized, the command must exit `2` with a clear error pointing to `substrate init`.
- Internal git readiness:
  - On init, create `.substrate-git/repo.git` (git directory) and ensure it is ignored by the user repo via a workspace `.gitignore` entry.
  - Internal git is used by later slices for commits/checkpoints/rollback; init creates the directory structure only.
- CLI/UX:
  - `substrate init` reports paths created, existing state, and next steps.
  - Provide a `--force` flag to repair missing pieces without deleting user data.

## Acceptance
- `substrate init` creates `.substrate/` and `.substrate-git/` (including `.substrate-git/repo.git`) without touching user `.git`; reruns are safe and non-destructive.
- `substrate sync`, `substrate checkpoint`, and `substrate rollback` fail with exit `2` until init succeeds; the error message points to `substrate init`.
- Default config written matches the defaults defined in C1 (sync settings) and Y0 (settings stack).
- `.gitignore` contains entries for `.substrate/` and `.substrate-git/` after init.

## Out of Scope
- Actual sync behavior, auto-sync, host↔world application.
- Detailed internal git commit flows (handled in C6).
