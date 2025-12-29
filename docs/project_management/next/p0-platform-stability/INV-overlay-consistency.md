# Investigation: World FS Mode Consistency (PTY vs non-PTY)

## Summary
- Policy `world_fs_mode` toggles are not yielding a single consistent world view. Non-PTY writes in writable mode stay inside the world overlay and disappear when remounting read-only; PTY writes in writable mode land on the host. Switching back to read-only shows only host state, so world-private files (e.g., `poo` from `touch`) vanish while host writes (e.g., `pee` from `nano`) persist.
- Mode switches reuse the same world id, but the PTY path bypasses the writable overlay, so reads after a mode change reflect different backing trees (host vs overlay upper).
- In read-only mode, PTY mounts read-only overlay and correctly EROFS on writes; in writable mode PTY runs on the host project dir.

## Repro (observed)
1. Set profile to `world_fs_mode: read_only`; start `substrate`.
   - `touch foo` → EROFS; `ls` shows host files.
2. Exit, set profile to `world_fs_mode: writable`; start `substrate`.
   - `touch poo` succeeds and appears in-session but not on host after exit.
   - `nano pee` creates file; after exit `pee` exists on host.
3. Re-enter writable world: `poo` and `pee` visible in-session.
4. Switch to read_only and re-enter: `pee` visible (host), `poo` missing (overlay-only).

## Current hypothesis
- PTY path (writable) executes on the host project dir (no overlay). Non-PTY path uses overlay/copy-diff. When fs_mode flips, the overlay lower is the host, so overlay-only files disappear while host-written files remain. We need a unified isolation strategy for PTY writable sessions.

## Next steps for the agent
1. Confirm PTY execution path in writable mode: trace where `ensure_session_world` sets `fs_mode` and how `world-agent/src/pty.rs` mounts overlays (read-only only). Verify there is no writable overlay for PTY.
2. Design: either run PTY sessions through a writable overlay (persistent per world) or explicitly document/guard host writes. Aim for parity: PTY and non-PTY must see the same world root and isolation rules.
3. Update fs_mode reuse logic to remount the same overlay for PTY and non-PTY so mode switches do not drop overlay state or leak host writes.
4. Add targeted tests:
   - Writable: PTY and non-PTY create files and are visible across commands; host remains unchanged when exiting.
   - Read-only: PTY and non-PTY both EROFS; host unchanged.
   - Mode switch: create in writable, switch to read-only, verify visibility (read-only sees overlay contents) and no host leaks.
5. Trace/telemetry: ensure spans report `fs_mode` and overlay use for PTY; add warnings if PTY falls back to host.

## Artifacts to collect
- Repro traces showing `execution_origin`, `world_fs_mode`, and world_id for PTY vs non-PTY commands.
- Mount logs indicating whether PTY ran with overlay/fuse or host path.
