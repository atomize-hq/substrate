# C1-spec: Config/CLI Surface for Sync

## Scope
- Introduce sync settings surfaces (no behavior change):
  - `auto_sync` (bool, default `false`)
  - `sync_direction` enum: `from_world` (default), `from_host`, `both`
  - `conflict_policy` enum: `prefer_host` (default), `prefer_world`, `abort`
  - `sync_filters`: include/exclude patterns (minimal support: exclude list; may stub includes if not implemented)
  - `dry_run` flag for the `substrate sync` command (logs intended actions, no effect)
- Surfaces:
  - CLI flags for `substrate sync` and persistent config (global + per-dir settings) plus env vars (SUBSTRATE_SYNC_*).
  - `substrate sync` command stub that parses/prints effective settings but does not alter host/world.
- No world-agent changes. No auto-sync hooks. No filesystem mutations.

## Acceptance
- Settings persist via existing settings stack (CLI > dir config > global config > env > default).
- `substrate sync --dry-run` (and without) prints effective settings and exits 0; no side effects.
- Protected paths (`.git`, `.substrate-git`, `.substrate`) are listed in help/output as always-excluded.
- Defaults match spec (`auto_sync=false`, `sync_direction=from_world`, `conflict_policy=prefer_host`).
- Docs/help strings updated where applicable (code task only updates inline help/usage; broader docs can wait for integration).

## Out of Scope
- Any actual sync/apply behavior.
- Auto-sync hooks.
- World-agent changes.
