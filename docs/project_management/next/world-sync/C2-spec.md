# C2-spec: Manual World→Host Sync (Non-PTY)

## Scope
- Implement `substrate sync` to apply world→host changes for non-PTY sessions only.
- Source of truth: fs_diff/overlay upper from non-PTY executions for the current session/world.
- Features:
  - Direction handling: support `from_world` and `both` (but host→world still stubbed; from_host should return clear error).
  - Conflict policy: `prefer_host` (skip/leave host), `prefer_world` (overwrite host), `abort` (fail on conflict).
  - Filters: honor excludes (including hardcoded protected paths `.git`, `.substrate-git`, `.substrate`, sockets/dev nodes). Optional include list OK to stub.
  - Size/limit guard: bail with clear message if diff exceeds configurable threshold (can be fixed constant for now).
  - Logging: summary of applied/skipped paths and conflict decisions.
- Manual command only; no auto-sync hooks yet.
- No PTY support yet.

## Acceptance
- `substrate sync` with `sync_direction=from_world` applies non-PTY overlay changes to host per conflict policy and filters.
- `sync_direction=from_host` fails with explicit “not implemented” message; `both` runs world→host then reports host→world as unimplemented.
- Protected paths are never touched; command exits non-zero if only protected paths would change (with message).
- Size guard prevents applying overly large diffs; message includes threshold and observed size/count.
- Behavior gated to Linux overlay path; on unsupported platforms, command degrades with clear error/skip.

## Out of Scope
- Auto-sync.
- PTY overlay/diffs.
- Host→world application.
- Internal git integration.
