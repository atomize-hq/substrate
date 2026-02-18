# Integration Map — Full isolation Landlock ↔ OverlayFS backing dirs allowlist

## Scope
- Restore allowlisted project writes in full isolation on Linux when Landlock is supported and overlayfs is the active filesystem strategy.

## Inputs → Derived state → Actions → Outputs

### Inputs
- Policy (effective, already resolved and injected by the world-agent):
  - `world_fs.isolation`
  - `world_fs.mode`
  - `world_fs.write_allowlist`
  - Landlock allowlists exported to the exec wrapper:
    - `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST`
    - `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST`
- Runtime env:
  - `SUBSTRATE_MOUNT_PROJECT_DIR` (project mountpoint inside the mount namespace)
- Procfs:
  - `/proc/self/mountinfo`

### Derived state
- `overlay_upperdir` and `overlay_workdir` for the project mountpoint, parsed from mountinfo `super_options`.

### Actions
- World backend mounts (existing behavior):
  - Construct full isolation rootfs and overlay view of the project.
  - Remount only `world_fs.write_allowlist` prefixes as writable (mount semantics).
- World-agent Landlock wrapper (this feature):
  - Parse mountinfo for the project mount entry.
  - Derive overlayfs internal backing dirs (`upperdir`, `workdir`).
  - Extend the Landlock write allowlist with these derived internal write roots.
  - Apply Landlock ruleset and exec the requested command.

### Outputs
- Allowlisted writes succeed under full isolation with Landlock enabled.
- Non-allowlisted writes remain denied.
- Policy snapshot schema and hash remain unchanged (derived internal paths are runtime-only).

## Component map (where changes land)
- `crates/world` (Linux-only):
  - Add a helper that parses `/proc/self/mountinfo` and derives overlayfs `upperdir` + `workdir` for a mountpoint.
- `crates/world-agent` (Linux-only behavior):
  - Extend the full-isolation Landlock exec wrapper to include derived overlayfs internal write roots in the Landlock write allowlist.

## Sequencing alignment
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- This feature directory is the authoritative Planning Pack for ADR-0015.
