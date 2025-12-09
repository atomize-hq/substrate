# World Sync (Draft)

This is a user-focused guide to Substrate’s world sync capabilities. It assumes all planned features are implemented (manual and auto sync, PTY support, host→world pre-sync, internal git history on both host and world, and rollback).

## What world sync does
- Keeps your host workspace and the isolated world overlay consistent.
- Lets you choose when and how to sync:
  - Direction: `from_world`, `from_host`, or `both`.
  - Conflict policy: favor host, favor world, or abort on conflicts.
  - Filters: skip protected paths and your own excludes.
- Supports manual commands and optional auto-sync on session exit.
- Records world↔host changes into an internal git repo (`.substrate-git`) for checkpoints and rollbacks.

## Protected paths (always skipped)
- `.git`, `.substrate-git`, `.substrate`
- Sockets, device nodes, and other special files

## Configuration surfaces
Precedence: CLI flag > directory config (.substrate/settings.toml) > global config (~/.substrate/config.toml) > env vars > defaults.

Key settings:
- `auto_sync` (default: false)
- `sync_direction` = `from_world` (default) | `from_host` | `both`
- `conflict_policy` = `prefer_host` (default) | `prefer_world` | `abort`
- `sync_filters` (exclude patterns)
- `dry_run` (for sync commands)
- Internal git:
  - `use_internal_git` (default: true)
  - `enforce_clean_tree_before_sync` (default: false)

Env var examples:
- `SUBSTRATE_SYNC_AUTO=true`
- `SUBSTRATE_SYNC_DIRECTION=both`
- `SUBSTRATE_SYNC_CONFLICT=prefer_world`
- `SUBSTRATE_SYNC_EXCLUDES=dist,node_modules`
- `SUBSTRATE_USE_INTERNAL_GIT=false`

## Commands

### Manual sync
```
substrate sync [--direction from_world|from_host|both] [--conflict prefer_host|prefer_world|abort] [--dry-run]
```
- `from_world`: apply world changes to host.
- `from_host`: pre-sync host into the world overlay (before commands).
- `both`: host→world then world→host.
- Honors excludes, size guard, and protected paths; will refuse if only protected paths would change.
- `--dry-run` logs intended actions without modifying files.

### Auto-sync
- Controlled by `auto_sync` and `sync_direction`.
- Runs on session exit when enabled and direction includes the needed path (e.g., from_world).
- Respects conflict policy, filters, size guard, and dry-run mode.
- Skips cleanly on platforms without overlay support (logs reason).

### Checkpoint
```
substrate checkpoint
```
- Records the current workspace state into `.substrate-git` (no sync).
- No-op if internal git disabled or no changes.

### Rollback
```
substrate rollback last
substrate rollback checkpoint <id>
substrate rollback session <id>
```
- Restores files from `.substrate-git` history.
- Honors clean-tree guard (unless forced) and protected-path rules.
- Refreshes world overlay to match restored host; warns if refresh fails.
- Creates a metadata commit noting the rollback (when internal git is enabled).

## Conflict policy
- `prefer_host`: host wins; world changes skipped on conflict.
- `prefer_world`: world wins; host overwritten on conflict.
- `abort`: stop on conflict with an error.
Host→world pre-sync uses the same policies but applied in that direction.

## Filters and size guard
- Exclude list applied to both directions; protected paths are hardcoded.
- Size guard prevents huge diffs from applying; command reports the threshold and observed size/count.

## Internal git (.substrate-git)
- Auto-initialized; isolated from your repo.
- Commits created after world→host syncs (when enabled) and via `substrate checkpoint`.
- Clean-tree guard (optional) blocks sync if external edits are present.
- Rollback commands use these commits; missing commits/tags surface clear errors.

## Common workflows

### Manual review/apply world changes
```
substrate sync --direction from_world --conflict prefer_host --dry-run
substrate sync --direction from_world --conflict prefer_host
```

### Auto-apply world changes on exit
Set `auto_sync=true` and `sync_direction=from_world` (or `both`) in config; run your session normally. On exit, the diff applies per policy and records to `.substrate-git`.

### Keep world overlay fresh from host before commands
```
substrate sync --direction from_host --conflict prefer_host
```
(Runs host→world pre-sync; no host mutations.)

### Create a checkpoint
```
substrate checkpoint
```

### Roll back to last change
```
substrate rollback last
```

### Roll back to a checkpoint
```
substrate rollback checkpoint <id>
```

## Platform notes
- Linux: overlay-based; full feature set available when overlay privileges exist.
- macOS/Windows: behavior depends on world backend support; commands degrade with clear errors when a path isn’t supported.

## Safety tips
- Keep `auto_sync` off until you’re comfortable; start with `--dry-run`.
- Use `prefer_host` if unsure; switch to `prefer_world` only when you trust world changes.
- Leave `use_internal_git` on to get checkpoints/rollback.
- Avoid syncing large generated trees; add them to excludes.
## Initialization (`substrate init`)
- Before using world features, run `substrate init` in your workspace root. This creates `.substrate/`, `.substrate-git/`, and seeds config with your chosen defaults (world enablement, sync settings, ignores).
- World mode (REPL or non-PTY commands) is gated on a successful `substrate init`. Without init, Substrate runs in host-only mode.
- `substrate init` is repeatable/safe: it will reuse existing `.substrate` state and report what was created/updated.
