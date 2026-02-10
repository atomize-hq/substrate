# C3-spec: Auto-Sync (Non-PTY) + Safety Rails

## Scope
- Add auto-sync hook for non-PTY sessions: on session close/exit, apply world→host diff using the same engine as C2.
- Respect settings from C1 (`sync.auto_sync`, `sync.direction`, `sync.conflict_policy`, `sync.exclude`).
- Auto-sync runs only when:
  - `sync.auto_sync=true`, and
  - `sync.direction=from_world`
- Additional safety:
  - Skip if diff exceeds size threshold or contains protected paths only (log reason).
  - Clear logging of what was applied/skipped and why; surface errors to caller.
- Non-PTY only; PTY still out of scope.
- Platform guard: if overlay diff is unavailable, auto-sync does not run and logs once.

## Acceptance
- Auto-sync triggers on session teardown when enabled and `sync.direction=from_world`; otherwise no-op.
- Uses same conflict policy/filtering/size guard as C2.
- Protected paths remain untouched; auto-sync aborts with clear message if violations would occur.
- Dry-run setting logs planned actions without mutating host.
- No regressions to manual `substrate sync`.

## Out of Scope
- PTY diffs.
- Host→world pre-sync.
- Internal git integration.
