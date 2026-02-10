# C1-spec: Config/CLI Surface for Sync

## Scope
- Introduce sync settings surfaces (no behavior change):
  - `sync.auto_sync` (bool, default `false`)
  - `sync.direction` enum: `from_world` (default), `from_host`, `both`
  - `sync.conflict_policy` enum: `prefer_host` (default), `prefer_world`, `abort`
  - `sync.exclude`: exclude patterns (YAML list of strings; include patterns are not supported in this track)
  - `dry_run` flag for the `substrate sync` command (logs intended actions, no effect)
- Surfaces:
  - CLI flags for `substrate sync` and persistent config (global + per-dir settings) plus env vars (SUBSTRATE_SYNC_*).
  - `substrate sync` command stub that parses/prints effective settings but does not alter host/world.
- No world-agent changes. No auto-sync hooks. No filesystem mutations.

## Acceptance
- Settings persist via existing settings stack (CLI > dir config > global config > env > default).
- `substrate sync --dry-run` (and without) prints effective settings and exits 0; no side effects.
- Protected paths (`.git`, `.substrate-git`, `.substrate`) are listed in help/output as always-excluded.
- Defaults match spec (`sync.auto_sync=false`, `sync.direction=from_world`, `sync.conflict_policy=prefer_host`).
- Docs/help strings updated where applicable (code task only updates inline help/usage; broader docs can wait for integration).

## Out of Scope
- Any actual sync/apply behavior.
- Auto-sync hooks.
- World-agent changes.

## Command surface (fixed)

### Persistent config keys (YAML-only; Y0)

These keys live in:
- Global: `~/.substrate/config.yaml`
- Workspace: `.substrate/settings.yaml`

Schema:
```yaml
sync:
  auto_sync: false
  direction: from_world        # from_world | from_host | both
  conflict_policy: prefer_host # prefer_host | prefer_world | abort
  exclude:
    - ".git/**"
    - ".substrate/**"
    - ".substrate-git/**"
```

### Env vars

Env vars override YAML config:
 - `SUBSTRATE_SYNC_AUTO_SYNC`: allowed values: `true`, `false`
- `SUBSTRATE_SYNC_DIRECTION`: allowed values: `from_world`, `from_host`, `both`
- `SUBSTRATE_SYNC_CONFLICT_POLICY`: allowed values: `prefer_host`, `prefer_world`, `abort`
- `SUBSTRATE_SYNC_EXCLUDE`: comma-separated list of glob patterns

### CLI flags (`substrate sync`)

CLI flags override env vars and YAML config:
- `--direction <from_world|from_host|both>`
- `--conflict-policy <prefer_host|prefer_world|abort>`
- `--exclude <glob>` (repeatable)
- `--dry-run`
- `--verbose`
