# C5-spec: Host→World Pre-Sync & Directionality

## Scope
- Implement host→world sync path used before entering/running commands (pre-sync) when `sync.direction` is `from_host` or `both`.
- Semantics:
  - `from_host`: apply host changes into world overlay before execution; no world→host apply.
  - `both`: pre-sync host→world, later world→host (manual or auto per previous specs).
- Conflict policy for host→world:
  - `prefer_host`: overwrite world overlay with host state.
  - `prefer_world`: keep overlay upper; host changes logged as skipped.
  - `abort`: fail when host/world differ on the same path.
- Filters: respect excludes and size guard as earlier; protected paths remain untouched.
- Clear logging for applied/skipped/conflict cases; exit `4` when the overlay diff path is unavailable.

## Acceptance
- With `sync.direction=from_host` or `sync.direction=both`, `substrate sync` performs host→world pre-sync per policy/filters and reports skipped/applied paths.
- Auto paths (if any) only run when direction includes `from_host`; no unintended host mutation.
- World overlay state after pre-sync reflects chosen conflict policy; non-PTY and PTY overlays remain aligned.
- Unsupported platforms exit `4` with a clear message and perform no mutations.

## Out of Scope
- Internal git integration.
- Rollback CLI.
