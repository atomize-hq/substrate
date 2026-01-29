# Schema — World FS Granular Allow/Deny (V2) (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It defines:
- the V2 policy schema for `world_fs` (policy patch files), and
- the V2 policy snapshot schema consumed by world-agent (`PolicySnapshotV2`).

## 1) Policy patch schema (YAML) — `world_fs` (V2)

### 1.1 Top-level keys
```yaml
world_fs:
  mode: read_only | writable
  isolation: workspace | full
  require_world: true | false
  enforcement: strict | best_effort         # full isolation only

  # Optional: if omitted, discover defaults to read (same allow/deny).
  discover:
    allow_list: [ <glob>, ... ]
    deny_list:  [ <glob>, ... ]             # default: []

  read:
    allow_list: [ <glob>, ... ]
    deny_list:  [ <glob>, ... ]             # default: []

  # Required iff mode=writable.
  write:
    allow_list: [ <glob>, ... ]
    deny_list:  [ <glob>, ... ]             # default: []
```

### 1.2 Backwards compatibility
- None.
- Any legacy keys are invalid and MUST hard error, including:
  - `world_fs.read_allowlist`
  - `world_fs.write_allowlist`

### 1.3 Pattern grammar (globs)
All patterns are evaluated as **project-root-relative globs**:
- Leading `./` is permitted but has no semantic meaning.
- Absolute paths (`/...`) are invalid.
- Any path segment equal to `..` is invalid.

Operators can use typical glob syntax:
- `*` matches within a single path segment.
- `**` matches across path segments (recursive).
- `?` and `[...]` MAY be supported depending on the chosen glob engine; if unsupported, they MUST be rejected (hard error) rather than ignored.

### 1.4 Allow/deny semantics
For each dimension (`discover`, `read`, `write`):
- A path is permitted iff:
  - it matches at least one entry in `allow_list`, AND
  - it matches no entry in `deny_list`.
- `deny_list` overrides `allow_list` (nested exceptions are allowed).

### 1.5 Defaults
- `deny_list` defaults to empty (`[]`) when omitted.
- If `discover` is omitted, it defaults to `read` (same allow/deny).

### 1.6 Isolation constraints (hard errors)
- If `world_fs.isolation=workspace`:
  - `world_fs.enforcement` MUST NOT be set to `strict` or `best_effort`.
  - Any `deny_list` usage MUST hard error.
  - `discover` MUST NOT be configured.
  - `read` MUST NOT be configured.
  - (This ADR scopes deny/visibility controls to `full` only; workspace behavior remains allowlist-only via existing mechanisms.)

### 1.7 Strict vs best-effort (full isolation only)
- `world_fs.enforcement=strict`:
  - Deny rules are a hard security boundary.
  - The workload MUST NOT be able to undo deny masks (see `ENV.md` and ADR security posture).
- `world_fs.enforcement=best_effort`:
  - Deny masks are applied before exec, but the workload may be able to undo them.
  - This mode exists only for compatibility with tooling that needs mount capabilities.

### 1.8 Filename-glob denies (`**/*.pem`) guarantee statement
- Filename-glob deny patterns (e.g., `**/*.pem`) are enforced as:
  - “snapshot at exec start”: the helper scans the project view immediately before executing the inner command, and masks all matching existing paths.
- Non-guarantees:
  - If a process creates/renames a matching file and reads it within the same long-running command invocation, that is not guaranteed to be blocked by v1 of this feature.

## 2) Policy snapshot schema (JSON) — `PolicySnapshotV2`

The world-agent MUST consume `PolicySnapshotV2` only.
- `schema_version` MUST be `2`.
- Unknown fields MUST be rejected.

### 2.1 Shape
```json
{
  "schema_version": 2,
  "world_fs": {
    "mode": "read_only|writable",
    "isolation": "workspace|full",
    "require_world": true,
    "enforcement": "strict|best_effort",
    "discover": { "allow_list": ["..."], "deny_list": ["..."] },
    "read":     { "allow_list": ["..."], "deny_list": ["..."] },
    "write":    { "allow_list": ["..."], "deny_list": ["..."] }
  },
  "net_allowed": ["..."],
  "limits": {
    "max_memory_mb": 0,
    "max_cpu_percent": 0,
    "max_runtime_ms": 0,
    "max_egress_bytes": 0
  }
}
```

Notes:
- `discover` MAY be omitted; when omitted it is treated as equal to `read` (same allow/deny).
- `write` MUST be omitted when `mode=read_only` (hard error if present).
- `read`/`discover` MUST be omitted in `isolation=workspace` (hard error if present).

