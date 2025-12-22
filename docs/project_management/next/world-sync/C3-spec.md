# C3-spec: Auto-Sync (Non-PTY) + Safety Rails

## Scope
- Add auto-sync hook for non-PTY sessions: on session close/exit, apply world→host diff using the same engine as C2.
- Respect settings from C1 (auto_sync, sync_direction, conflict_policy, filters). Auto-sync should only run when `auto_sync=true` and `sync_direction` includes `from_world` or `both`.
- Additional safety:
  - Skip if diff exceeds size threshold or contains protected paths only (log reason).
  - Clear logging of what was applied/skipped and why; surface errors to caller.
  - Allow “dry-run” mode to log intended auto-sync without applying (if enabled via setting).
- Non-PTY only; PTY still out of scope.
- Platform guard: skip gracefully when overlay unavailable; log once.

## Acceptance
- Auto-sync triggers on session teardown when enabled and direction includes `from_world`; otherwise no-op.
- Uses same conflict policy/filtering/size guard as C2.
- Protected paths remain untouched; auto-sync aborts with clear message if violations would occur.
- Dry-run setting logs planned actions without mutating host.
- No regressions to manual `substrate sync`.

## Out of Scope
- PTY diffs.
- Host→world pre-sync.
- Internal git integration.
