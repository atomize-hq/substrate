# Schema — World FS Granular Allow/Deny (V2) (Authoritative)

This document is authoritative for:
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

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

  # Full isolation only. Optional: if omitted, discover defaults to read (same allow/deny).
  discover:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []

  # Full isolation only.
  read:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []

  # Full isolation only. Required iff mode=writable.
  write:
    allow_list: [ <pattern>, ... ]
    deny_list:  [ <pattern>, ... ]          # default: []
```

Workspace example (valid):
```yaml
world_fs:
  mode: read_only | writable
  isolation: workspace
  require_world: true | false
  # NOTE: In workspace isolation, `enforcement`, `read`, `discover`, and `write` are not allowed.
```

### 1.2 Backwards compatibility
- None.
- Any legacy keys are invalid and MUST hard error, including:
  - `world_fs.read_allowlist`
  - `world_fs.write_allowlist`

### 1.3 Pattern grammar (patterns)
All patterns are evaluated as **project-root-relative patterns**:
- Leading `./` is permitted but has no semantic meaning.
- Trailing `/` is permitted but has no semantic meaning (it is ignored during normalization).
- Absolute paths (`/...`) are invalid.
- Any path segment equal to `..` is invalid.

Supported syntax (deterministic; hard errors on violation):
- `allow_list` entries MUST NOT contain any glob metacharacters (`*`, `?`, `[`, `]`).
  - Allowed forms are: `.` OR a literal project-root-relative path/prefix (examples below).
- `deny_list` entries MUST be one of:
  - a literal project-root-relative path/prefix containing no glob metacharacters, OR
  - a wildcard pattern that uses ONLY `*` and/or `**`.
  - `?` and `[...]` are NOT supported and MUST be rejected (hard error) if present in any deny pattern.

Wildcard matching rules (deny_list only):
- `*` matches within a single path segment (it MUST NOT match `/`).
- `**` matches across path segments (it matches `/`) and is used for recursive matching.

Examples (valid):
- Allow literals:
  - `.` (entire project)
  - `src`
  - `docs/public.txt`
  - `dir/subdirectory/` (trailing slash is normalized away; treated as `dir/subdirectory`)
- Deny wildcards:
  - `**/*.pem` (matches any `.pem` anywhere)
  - `ignore-*.md` (matches only at the project root: `ignore-a.md`; does not match `docs/ignore-a.md`)
  - `**/ignore-*.md` (matches `ignore-a.md` at root and `docs/ignore-a.md` anywhere)
  - `secrets/**` (matches everything under `secrets/`)

Examples (invalid; MUST hard error):
- `allow_list: ["src/**"]` (wildcards are not allowed in allow_list)
- `deny_list: ["file?.txt"]` (contains `?`)
- `deny_list: ["file[0-9].txt"]` (contains `[...]`)

### 1.4 Allow/deny semantics
For each dimension (`discover`, `read`, `write`):
- A path is permitted iff:
  - it matches at least one entry in `allow_list`, AND
  - it matches no entry in `deny_list`.
- `deny_list` overrides `allow_list` (nested exceptions are allowed).

Literal (non-wildcard) pattern semantics:
- Let `p` be a normalized literal pattern (no wildcards; no leading `/`; no `..`; trailing `/` trimmed unless `p="."`).
- Let `rel` be a normalized project-root-relative path.

Then `p` matches `rel` iff:
- `p == "."`, OR
- `rel == p`, OR
- `rel` starts with `p + "/"`.

Examples:
- `p = "secrets"` matches `secrets`, `secrets/a.txt`, `secrets/nested/b.txt`.
- `p = "secrets/"` is normalized to `secrets` (same matches as above).
- `p = "docs/public.txt"` matches only `docs/public.txt` (and not `docs/public.txt.bak`).
- `p = "src"` does not match `srcfile` (because it is neither `src` nor under `src/`).

### 1.5 Validation rules (hard errors)
- For any configured dimension (`read`, `discover`, `write`), `allow_list` MUST be non-empty.
- `world_fs.enforcement` MUST be present iff at least one `deny_list` is non-empty.
  - If `world_fs.enforcement` is present and all `deny_list` values are empty (or omitted), it MUST be rejected as invalid config (hard error).
- If any `deny_list` is non-empty:
  - `world_fs.isolation` MUST be `full`, and
  - `world_fs.require_world` MUST be `true`.

### 1.6 Defaults
- `deny_list` defaults to empty (`[]`) when omitted.
- If `discover` is omitted, it defaults to `read` (same allow/deny).

### 1.7 Isolation constraints (hard errors)
- If `world_fs.isolation=workspace`:
  - `world_fs.enforcement` MUST be omitted. If present, it MUST be rejected as invalid config (hard error).
  - `world_fs.read`, `world_fs.discover`, and `world_fs.write` MUST be omitted. If any are present, they MUST be rejected as invalid config (hard error).
  - Any `deny_list` usage MUST be rejected as invalid config (hard error).

### 1.8 Strict vs best-effort (full isolation only)
- `world_fs.enforcement=strict`:
  - Deny rules are a hard security boundary.
  - The workload MUST NOT be able to undo deny masks (see `ENV.md` and ADR security posture).
- `world_fs.enforcement=best_effort`:
  - Deny masks are applied before exec, but the workload can undo them after it starts.
  - This mode exists only for compatibility with tooling that needs mount capabilities.

### 1.9 Wildcard denies (e.g., `**/*.pem`) guarantee statement
Wildcard deny patterns are supported in `deny_list` only (never in `allow_list`).

Guarantee:
- Wildcard deny patterns (e.g., `**/*.pem`, `**/ignore-*.md`) are enforced as:
  - “snapshot at exec start”: immediately before executing the inner command, the helper enumerates the set of matching paths that exist at that time, and masks each matching path in all nameable in-world project views (both `/project/...` and `$SUBSTRATE_MOUNT_PROJECT_DIR/...`) before any user code executes.

Symlink handling (to avoid ambiguity and overpromises):
- The snapshot scan MUST NOT follow symlinks while traversing the project tree.
- Deny masks apply to paths, not to the conceptual “targets” of symlinks.

Examples:
- `deny_list: ["**/*.pem"]`:
  - denies `certs/a.pem`
  - denies `a.pem`
  - does not deny `a.pemd`
- `deny_list: ["ignore-*.md"]`:
  - denies `ignore-a.md`
  - does not deny `docs/ignore-a.md` (use `**/ignore-*.md` to deny anywhere)
- `deny_list: ["secrets/**"]`:
  - denies `secrets/secret.txt`
  - denies `secrets/nested/key.txt`

Non-guarantees:
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
- `discover` is optional; when omitted it is treated as equal to `read` (same allow/deny).
- `write` MUST be omitted when `mode=read_only` (hard error if present).
- If `isolation=workspace`, `enforcement`, `read`, `discover`, and `write` MUST be omitted (hard error if present).
- `enforcement` MUST be present iff at least one `deny_list` is non-empty.
- If any `deny_list` is non-empty, `require_world` MUST be true.
