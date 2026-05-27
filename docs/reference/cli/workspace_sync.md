# `substrate workspace sync` (How it behaves today)

This page explains `workspace sync` in plain English using only **host** vs **world** terms.

If you want the deeper implementation details (overlayfs, agents, code pointers), see:
`docs/internals/world/workspace_sync_filesystem_model.md`.

For Substrate's workspace-history commands, see:
`docs/reference/cli/workspace_history.md`.

## Host vs world: what are the two filesystems?

- **Host** = your real workspace directory on your machine.
- **World** = what commands run under `substrate` see.

Today (Linux world backend), the world’s view of your project is:

> **World = Host (live) + World-pending changes**

That means:
- Changes you make on **host** are generally visible in **world immediately**.
- Changes you make in **world** are stored as **pending world changes** until you sync them to host.

## What `workspace sync` does

`workspace sync` is primarily about handling **pending world changes**:

- It can apply pending **world → host** changes.
- It can also reconcile cases where **host changed** but the world is still “hiding” those host
  changes behind pending world changes.

## Directions

### `direction=from_world`

“Apply pending world changes onto host.”

- Host is mutated.
- World pending changes are cleared after a successful apply.

### `direction=from_host`

“Make world stop hiding the host for paths where world has pending changes.”

Important:
- This does **not** “upload host changes into world” as a separate copy.
- Host is already the baseline of what the world sees.
- `from_host` only matters for paths where the world currently has pending changes that shadow host.

### `direction=both`

“Reconcile first (`from_host`), then apply (`from_world`).”

## Conflict policy

When the same path has changed on both sides, Substrate treats it as a conflict and applies your
conflict policy:

- `prefer_host`: don’t apply the world’s version for that path; keep the host version.
- `prefer_world`: overwrite the host version with the world version.
- `abort`: refuse to sync anything if any conflict exists.

## Common scenarios (what to expect)

### New file created in world

- World: file exists immediately.
- Host: file does not exist yet.
- Run `workspace sync` (from_world): host gets the file.

### Modify file in world

- World: modified contents are visible in world.
- Host: unchanged until you run `workspace sync` (from_world).

### Delete file in world

- World: the file is gone in world.
- Host: the file stays until you run `workspace sync` (from_world), which can delete it on host
  (unless it’s protected or skipped due to conflict policy).

### New file created on host

- Host: file exists immediately.
- World: file is visible immediately (because world tracks the host baseline).
- Running `workspace sync` does not “send” it anywhere; it is already visible in world.

### Delete a file on host after it was synced from world

If a file was created in world, synced to host, and then the pending world changes were cleared:
- deleting it on host will also make it disappear in world (because it is now part of the shared baseline).

## How to preview safely

- `substrate workspace sync --dry-run --verbose`
  - shows how many pending changes exist and what would be applied/skipped.

## Auto-sync (`sync.auto_sync=true`)

If `sync.auto_sync=true`, Substrate will attempt to run `workspace sync` automatically:

- After a successful one-shot world command (for example `substrate -c "..."`).
- When exiting the interactive REPL (`substrate` → `exit` / Ctrl+D).

Auto-sync respects your effective `sync.direction` (`from_world` or `both`) and
`sync.conflict_policy`. If auto-sync fails, Substrate prints an `auto-sync failed: ...` message and
returns a non-zero exit code.

## Safety rails

Some paths are protected and will not be synced/mutated:
- `.git/**`
- `.substrate/**`

If the world’s pending changes include protected paths, sync refuses.

## Related workspace history commands

`workspace sync` is separate from Substrate's checkpoint and rollback history commands:

- `substrate workspace checkpoint`
- `substrate workspace rollback <target>`

Those commands use Substrate's internal history store rather than world pending diffs. Their
operator-facing contract lives in:

- `docs/reference/cli/workspace_history.md`
