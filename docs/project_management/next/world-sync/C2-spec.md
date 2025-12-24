# C2-spec: Manual World→Host Sync (Non-PTY)

## Scope
- Implement `substrate sync` to apply world→host changes for non-PTY sessions only.
- Source of truth: fs_diff/overlay upper from non-PTY executions for the current session/world.
- Features:
  - Direction handling:
    - `sync.direction=from_world` is supported.
    - `sync.direction=from_host` and `sync.direction=both` must fail with exit `4` and a clear “not implemented until C5” message.
  - Conflict policy: `prefer_host` (skip/leave host), `prefer_world` (overwrite host), `abort` (fail on conflict).
  - Filters: honor excludes (including hardcoded protected paths `.git`, `.substrate-git`, `.substrate`, sockets/dev nodes). If the diff contains only protected paths, `substrate sync` must exit `5`.
  - Size/limit guard: bail with a clear message if the diff exceeds fixed thresholds:
    - max paths: `10000`
    - max bytes: `104857600` (100 MiB)
    - exit code: `5`
  - Logging: summary of applied/skipped paths and conflict decisions.
- Manual command only; no auto-sync hooks yet.
- No PTY support yet.

## Acceptance
- With `sync.direction=from_world`, `substrate sync` applies non-PTY overlay changes to host per conflict policy and filters.
- With `sync.direction=from_host` or `sync.direction=both`, `substrate sync` exits `4` with a clear “not implemented until C5” message.
- Protected paths are never touched; if only protected paths would change, `substrate sync` exits `5` with a clear message.
- Size guard prevents applying overly large diffs; `substrate sync` exits `5` and the message includes threshold and observed size/count.
- On unsupported platforms or when the overlay diff is unavailable, `substrate sync` exits `4` with a clear message and performs no mutations.

## Out of Scope
- Auto-sync.
- PTY overlay/diffs.
- Host→world application.
- Internal git integration.
