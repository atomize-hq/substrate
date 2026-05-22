# World Root Selection and Caging (Internals)

This document explains how Substrate:

- selects the “world root” (the directory that is overlaid/diffed and that full-isolation allow/deny rules are relative to),
- optionally “cages” the session so `cd ..` cannot escape that root,
- and why `world.anchor_mode` is *not* equivalent to `world.caged`.

This is an implementation note; the stable operator contract for the knobs lives in
`docs/reference/config/world.md`.

## Key terms

- **Workspace root**: the nearest ancestor containing `<root>/.substrate/workspace.yaml` (unless disabled by
  `<root>/.substrate/workspace.disabled`). Implementation: `crates/shell/src/execution/workspace.rs`.
- **World root / project_dir**: the directory the world backend overlays and uses as the “project” mount.
  In full isolation, allow/deny lists are interpreted relative to this root.
- **Caging**: preventing the interactive shell from leaving the chosen root via `cd`.

## Configuration surface

The effective config is resolved from defaults + global config + workspace config + CLI/env overrides. For details, see:

- `docs/CONFIGURATION.md`
- `docs/reference/config/contract.md`
- ADRs: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`,
  `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`

The settings relevant to this document:

- `world.anchor_mode`: `workspace` | `follow-cwd` | `custom`
- `world.anchor_path`: required when `anchor_mode=custom`
- `world.caged`: `true` or `false`

## How the shell resolves the world root

The shell computes a `WorldRootSettings` from the effective config and the current working directory:

- Implementation: `crates/shell/src/execution/settings/builder.rs` (`resolve_world_root`)
- Data type: `WorldRootSettings { mode, path, caged }`

Resolution rules (Linux/macOS/Windows share this high-level logic; backends differ):

1. **`anchor_mode=workspace`**
   - In code this is `WorldRootMode::Project`.
   - The anchor path resolves to the workspace root if a workspace marker exists; otherwise it falls back to the directory
     where `substrate` launched.
2. **`anchor_mode=follow-cwd`**
   - Anchor path starts as the launch directory, but `WorldRootSettings` treats the *effective root* as the *current* cwd
     (see below).
3. **`anchor_mode=custom`**
   - Anchor path resolves to `world.anchor_path` (relative paths are interpreted relative to the launch directory).
   - The path must exist and be a directory.

Two important derived values:

- `WorldRootSettings::anchor_root(current_dir)`: the root that should apply for a given current directory.
  - `follow-cwd` returns `current_dir`.
  - other modes return the resolved `path`.
- `WorldRootSettings::effective_root()`: used by some call sites when they want “the root right now”.
  - `follow-cwd` returns `std::env::current_dir()`.
  - other modes return the resolved `path`.

## How root selection reaches the world backend

Substrate communicates the selected root to downstream layers via exported env vars:

- `SUBSTRATE_ANCHOR_MODE`: `workspace` | `follow-cwd` | `custom`
- `SUBSTRATE_ANCHOR_PATH`: the resolved anchor path (empty unless needed)

In the async REPL, the shell also updates these env vars when starting/restarting persistent sessions:

- REPL code: `crates/shell/src/repl/async_repl.rs`
  - `apply_anchor_env_for_cwd(...)` resolves the root for a given cwd and writes `SUBSTRATE_ANCHOR_MODE/PATH` into the
    session env.
  - The REPL restarts a persistent world session when the policy snapshot or workspace root changes (the “drift restart”).

The world-service uses `SUBSTRATE_ANCHOR_MODE/PATH` + the request cwd to compute `project_dir`:

- `crates/world-service/src/service.rs` (`resolve_project_dir`)
  - `workspace`: `project_dir = SUBSTRATE_ANCHOR_PATH || cwd`
  - `follow-cwd`: `project_dir = cwd`
  - `custom`: `project_dir = SUBSTRATE_ANCHOR_PATH` (required)

That `project_dir` is passed into the world backend as `WorldSpec.project_dir`, which determines:

- what directory is overlaid/bind-mounted as the “project view”,
- what paths are used for fs diffs,
- and (in full isolation) what “relative allow/deny patterns” mean.

## How caging is enforced (and when it is disabled)

“Caging” is implemented as a guard that wraps `cd` in the interactive shell. On Linux the backend injects a shell snippet
that:

- runs a real `cd`,
- checks the resulting `pwd -P`,
- and if it is outside the anchor root, prints a message and jumps back to the anchor root.

Implementation: `crates/world/src/guard.rs` (`wrap_with_anchor_guard`).

Whether this guard is enabled is controlled by:

- `SUBSTRATE_CAGED` (derived from `world.caged`)
- and `SUBSTRATE_ANCHOR_MODE`

Important behavior:

- If `SUBSTRATE_ANCHOR_MODE=follow-cwd`, the guard is disabled even when `world.caged=true`.
  - Code: `crates/world/src/guard.rs` (`should_guard_anchor`) returns false for `FollowCwd`.
  - Rationale: if the “root” is allowed to move with cwd, “prevent leaving the root” becomes ill-defined.

So **`anchor_mode=follow-cwd` implies “uncaged” behavior**, but the inverse is not true:

- `world.caged=false` lets you roam, but the chosen world root may remain anchored (workspace/custom).
- `anchor_mode=follow-cwd` lets you roam *and* changes the world root semantics to track cwd.

## Why `anchor_mode` matters for full isolation allow/deny

In full isolation, policy allow/deny lists are interpreted relative to the world root. That means the same allowlist entry
can mean different absolute paths depending on the anchor mode.

Example policy (conceptual):

```yaml
world_fs:
  isolation: full
  write:
    allow_list: ["testdir"]
```

If the workspace root is `/work` and you are currently in `/work/testdir`:

- `anchor_mode=workspace` (root pinned to `/work`):
  - `testdir` means `/work/testdir`
  - `touch ./file` in `/work/testdir` is allowed
- `anchor_mode=follow-cwd` (root becomes `/work/testdir`):
  - `testdir` means `/work/testdir/testdir`
  - `touch ./file` in `/work/testdir` is denied
  - `touch ./testdir/file` is allowed

This is observable in the computed Landlock env vars that world-service passes into the world backend (Linux):

- `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST`: write prefixes (root-relative)
- `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST`: absolute paths that Landlock should allow writing to

Repro harness:

- Script: `scripts/dev/test_anchor_modes.sh`

## Related implementation docs

- Workspace detection and config precedence:
  - `docs/CONFIGURATION.md`
  - `docs/reference/config/contract.md`
  - `docs/internals/env/inventory.md` (exported state env vars)
- Policy snapshot and full isolation filesystem enforcement:
  - `docs/WORLD.md`
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- REPL persistent sessions and drift restarts:
  - `docs/internals/repl/persistent_session.md`

