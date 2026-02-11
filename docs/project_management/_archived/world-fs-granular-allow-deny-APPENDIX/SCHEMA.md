# Schema — World FS Allow/Deny Appendix (V3) (Authoritative)

This document is authoritative for Appendix A + B of:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It defines:
- the V3 policy patch schema for `world_fs` (policy patch files), and
- the snapshot schema consumed by world-agent (`PolicySnapshotV3`).

## 1) Policy patch schema (YAML) — `world_fs` (V3)

### 1.1 Top-level keys
```yaml
world_fs:
  # Host path visibility / rootfs isolation:
  # - true  => host paths remain nameable
  # - false => host paths are not nameable
  host_visible: true|false

  # Routing behavior when the world backend is unavailable / handshake fails:
  fail_closed:
    routing: true|false

  # Deny enforcement posture (only meaningful when any deny_list is non-empty):
  # - strict        => deny rules are a hard security boundary; fail if strict cannot be enforced
  # - prefer_strict => use strict when available; otherwise fall back without failing
  # - weak          => deny rules are applied but are not a hard boundary
  deny_enforcement: strict|prefer_strict|weak

  # Policy-level caging requirement (REPL + command execution):
  caged_required: true|false

  # Directory visibility (full isolation only).
  discover:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []

  # File read access (full isolation only).
  read:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []

  # Project write behavior (always valid).
  write:
    enabled: true|false
    # Full isolation only:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []
```

### 1.2 Backwards compatibility
- None.
- The following legacy keys are invalid and MUST hard error (exit `2` on host):
  - `world_fs.read_allowlist`
  - `world_fs.write_allowlist`
  - `world_fs.mode`
  - `world_fs.isolation`
  - `world_fs.require_world`
  - `world_fs.enforcement`

### 1.3 Pattern grammar (patterns)
All patterns are evaluated as **project-root-relative patterns**:
- Leading `./` is permitted but has no semantic meaning.
- Trailing `/` is permitted but has no semantic meaning (it is ignored during normalization).
- Absolute paths (`/...`) are invalid.
- Any path segment equal to `..` is invalid.

Supported syntax (deterministic; hard errors on violation):
- `allow_list` entries MUST NOT contain any glob metacharacters (`*`, `?`, `[`, `]`).
  - Allowed forms are: `.` OR a literal project-root-relative path/prefix.
- `deny_list` entries MUST be one of:
  - a literal project-root-relative path/prefix containing no glob metacharacters, OR
  - a wildcard pattern that uses ONLY `*` and/or `**`.
  - `?` and `[...]` are NOT supported and MUST be rejected if present in any deny pattern.

Wildcard matching rules (deny_list only):
- `*` matches within a single path segment (it MUST NOT match `/`).
- `**` matches across path segments (it matches `/`) and is used for recursive matching.

Literal (non-wildcard) pattern semantics:
- Let `p` be a normalized literal pattern (no wildcards; no leading `/`; no `..`; trailing `/` trimmed unless `p="."`).
- Let `rel` be a normalized project-root-relative path.

Then `p` matches `rel` iff:
- `p == "."`, OR
- `rel == p`, OR
- `rel` starts with `p + "/"`.

### 1.4 Defaults (explicit)
- `world_fs.host_visible` defaults to `true`.
- `world_fs.fail_closed.routing` defaults to `false`.
- `world_fs.write.enabled` defaults to `true`.
- `deny_list` defaults to `[]` when omitted.
- If `discover` is omitted and `world_fs.host_visible=false`, it defaults to `read` (same allow/deny).
- In full isolation (`world_fs.host_visible=false`), if `read.allow_list` is omitted, it defaults to `["."]`.
- In full isolation (`world_fs.host_visible=false`), if `write.enabled=true` and `write.allow_list` is omitted, it defaults to `["."]`.

### 1.5 Validation rules (hard errors)

#### 1.5.1 Routing invariants
- If `world_fs.write.enabled=false`, then `world_fs.fail_closed.routing` MUST be `true`.

#### 1.5.2 Full-isolation-only keys
- If `world_fs.host_visible=true`:
  - `world_fs.read` MUST be omitted.
  - `world_fs.discover` MUST be omitted.
  - `world_fs.write.allow_list` and `world_fs.write.deny_list` MUST be omitted.
  - Any deny list usage MUST be rejected as invalid config.

#### 1.5.3 Allow/deny shape
- For `read`, `discover`, and `write` (when applicable):
  - `allow_list` MUST be non-empty after defaulting.
  - A path is permitted iff it matches at least one `allow_list` entry AND matches no `deny_list` entry.
  - `deny_list` overrides `allow_list`.

#### 1.5.4 Deny enforcement posture
- If any `deny_list` is non-empty (in any dimension):
  - `world_fs.deny_enforcement` MUST be present.
- If all `deny_list` values are empty (or omitted):
  - `world_fs.deny_enforcement` MAY be present.

## 2) Snapshot schema — `PolicySnapshotV3` (JSON)

### 2.1 Versioning
- `schema_version` MUST be the integer `3`.
- Unknown fields MUST be rejected by world-agent (HTTP 400 / fatal error frame).

### 2.2 Canonical JSON shape
```json
{
  "schema_version": 3,
  "world_fs": {
    "host_visible": true,
    "fail_closed": { "routing": false },
    "deny_enforcement": "strict|prefer_strict|weak",
    "caged_required": false,
    "discover": { "allow_list": ["."], "deny_list": [] },
    "read": { "allow_list": ["."], "deny_list": [] },
    "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
  }
}
```

### 2.3 Snapshot canonicalization and hashing
The canonical `PolicySnapshotV3` JSON payload used for host-side drift detection hashing MUST be:
- the UTF-8 bytes produced by serializing the in-memory `PolicySnapshotV3` struct via `serde_json` (compact; no whitespace),
- with all defaults applied (per section 1.4) before serialization,
- with patterns normalized and validated per section 1.3 before serialization,
- with object field ordering as emitted by struct serialization and array ordering preserved as given (no deduplication or sorting).

Hashing:
- The host snapshot hash is the lowercase hex SHA-256 of the canonical JSON payload bytes above.
