# WO0-spec — OverlayFS Enumeration Health + Stable Mount Topology (Linux)

## Scope (authoritative)
Implement a Linux world filesystem strategy that guarantees directory enumeration correctness inside the world project view, with a fallback chain when the primary overlay strategy is unhealthy.

## Contract

### Functional requirements
- When Substrate runs a command in world mode against a project directory:
  - A file created in the project directory MUST be discoverable via directory enumeration inside the world view.
  - Directory enumeration MUST include `.` and `..` entries (directly or via equivalent `readdir` semantics), and MUST not be “always empty” on a non-empty directory.

### Strategy selection
- Primary strategy: kernel overlayfs (when available and healthy).
- Fallback strategy: fuse-overlayfs (when kernel overlayfs is unavailable or fails the enumeration health check).
- If neither strategy is viable:
  - If world is required: fail closed (exit code `3`).
  - If world is optional: fall back to host execution with a clear warning and trace annotation.

### Health check (enumeration)
- The health check MUST validate directory enumeration on the mounted project view and MUST NOT rely solely on stat-ing known filenames.
- The health check MUST run:
  - after mount creation, before running user commands (for new sessions), or
  - at the start of a command run when reusing cached session state.

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `3`: required world filesystem strategy unavailable

## Out of scope
- macOS and Windows world backends.
- Policy/schema changes.

