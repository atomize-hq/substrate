# C4-spec: PTY Overlay Diff + World→Host Sync

## Scope
- Extend PTY execution path to expose usable fs_diff/overlay upper for the current session/world.
- Enable manual `substrate sync` to apply PTY-originated changes (world→host) with the same conflict/filter/size guard semantics as non-PTY.
- Enable auto-sync for PTY sessions on exit when enabled and direction includes `from_world`/`both`.
- Ensure PTY overlay state aligns with non-PTY overlay (shared session/world) and protected paths remain untouched.
- Platform guards/skip with clear messaging when overlay/privileges unavailable.

## Acceptance
- PTY commands produce retrievable diffs for the session; manual sync can apply them to host respecting conflict/filter/size guard.
- Auto-sync applies PTY changes on session exit when enabled; otherwise no-op.
- Errors surface clearly (e.g., no overlay support, read-only failures).
- Non-PTY behavior unchanged/regression-free.

## Out of Scope
- Host→world pre-sync.
- Internal git integration.
