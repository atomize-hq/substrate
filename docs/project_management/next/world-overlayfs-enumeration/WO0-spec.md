# WO0-spec — OverlayFS Enumeration Health + Stable Mount Topology (Linux)

## Scope (authoritative)
Implement a Linux world filesystem strategy that guarantees directory enumeration correctness inside the world project view, with a fallback chain when the primary overlay strategy is unhealthy.

## Contract

### Functional requirements
- When Substrate runs a command in world mode against a project directory:
  - A file created in the project directory MUST be discoverable via directory enumeration inside the world view.
  - Directory enumeration MUST include `.` and `..` entries (directly or via equivalent `readdir` semantics), and MUST not be “always empty” on a non-empty directory.

### World requirement
- World is **required** if either is true:
  - The CLI explicitly requests world execution (`--world`), or
  - Policy mode is `enforce` and the command is under a “requires world” constraint, including any of:
    - `world_fs.require_world=true`
    - `world_fs.mode=read_only`
    - `world_fs.isolation=full`
    - command matched `allow_with_restrictions` (isolated match)
- World is **optional** otherwise.

### Strategy selection
- Primary strategy: kernel overlayfs (`overlay`).
- Fallback strategy: fuse-overlayfs (`fuse`).
- If neither strategy is viable:
  - If world is required: fail closed (exit code `3`).
  - If world is optional: fall back to host execution with the exact warning line and trace annotation defined in ADR-0004.

### Health check (enumeration)
- Probe id: `enumeration_v1`
- The health check MUST validate directory enumeration on the mounted project view and MUST NOT rely solely on stat-ing known filenames.
- The health check MUST use a dedicated probe overlay (not the user session overlay) and MUST:
  1. Create a probe file at `./.substrate_enum_probe` in the probe merged directory.
  2. Run `ls -a1` in that directory and assert the output contains a line exactly equal to `.substrate_enum_probe`.
  3. Remove `./.substrate_enum_probe`.
- The health check MUST run:
  - after mount creation, before running user commands (for new sessions), or
  - at the start of a command run when reusing cached session state.

### Warning line (world optional host fallback)
- If and only if Substrate executes on host due to “world optional + no viable strategy”, it MUST emit exactly one warning line to stderr:
  - `substrate: warn: world unavailable; falling back to host`

### Observability
- Trace spans (on `command_complete` events) MUST include:
  - `world_fs_strategy_primary`: `overlay|fuse`
  - `world_fs_strategy_final`: `overlay|fuse|host`
  - `world_fs_strategy_fallback_reason`: one of:
    - `none`
    - `primary_unavailable`
    - `primary_mount_failed`
    - `primary_probe_failed`
    - `fallback_unavailable`
    - `fallback_mount_failed`
    - `fallback_probe_failed`
    - `world_optional_fallback_to_host`
- `substrate world doctor --json` MUST include (DS0 doctor envelope):
  - `world.world_fs_strategy.primary`: `overlay`
  - `world.world_fs_strategy.fallback`: `fuse`
  - `world.world_fs_strategy.probe.id`: `enumeration_v1`
  - `world.world_fs_strategy.probe.probe_file`: `.substrate_enum_probe`
  - `world.world_fs_strategy.probe.result`: `pass|fail`
  - `world.world_fs_strategy.probe.failure_reason`: string or null

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `3`: required world filesystem strategy unavailable

## Out of scope
- macOS and Windows world backends.
- Policy/schema changes.
