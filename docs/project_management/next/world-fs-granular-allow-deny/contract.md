# world-fs-granular-allow-deny — contract surface

This file is the single place to consolidate the user-facing contract for this feature (CLI/config/exit codes/paths).

Authoritative references:
- ADR: `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Schema: `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- Protocol: `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- Env: `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`

## CLI
- Commands:
  - Existing `substrate policy ...` commands continue to be used to edit policy patch files.
  - New/changed keys are described in Config below.

## Config
- Files and precedence:
  - Effective policy continues to be resolved by the host via broker patch precedence (ADR-0008/ADR-0012):
    1. Global policy patch: `$SUBSTRATE_HOME/policy.yaml`
    2. Workspace policy patch: `<workspace_root>/.substrate/policy.yaml` (when a workspace exists)
    3. Defaults

- Schema (breaking; no backwards compatibility):
  - Legacy keys are invalid and MUST hard error (no silent ignore), including:
    - `world_fs.read_allowlist`
    - `world_fs.write_allowlist`
  - New V2 keys (full isolation only):
    - `world_fs.enforcement: strict|best_effort`
    - `world_fs.read.allow_list`, `world_fs.read.deny_list`
    - `world_fs.discover.allow_list`, `world_fs.discover.deny_list` (optional; defaults to mirror `read`)
    - `world_fs.write.allow_list`, `world_fs.write.deny_list` (required only when `world_fs.mode=writable`)

- Pattern constraints (hard errors):
  - Patterns MUST be project-root-relative.
  - Absolute patterns (`/...`) are invalid.
  - Any `..` segment is invalid.
  - Trailing `/` is allowed but ignored during normalization.
  - `allow_list` patterns MUST NOT contain any glob metacharacters (`*`, `?`, `[`, `]`).
  - `deny_list` patterns MUST be either:
    - a literal project-root-relative path/prefix containing no glob metacharacters, OR
    - a wildcard pattern that uses ONLY `*` and/or `**`.
    - `?` and `[...]` are NOT supported and MUST be rejected if present in any deny pattern.
  - `allow_list` MUST be non-empty for any configured dimension.
  - `world_fs.enforcement` MUST be present iff at least one `deny_list` is non-empty.
  - If any `deny_list` is non-empty, `world_fs.require_world` MUST be `true`.

- Isolation constraints (hard errors):
  - Deny lists (`*.deny_list`) are supported only in `world_fs.isolation=full`.
  - `world_fs.enforcement` is supported only in `world_fs.isolation=full`.
  - If `world_fs.isolation=workspace`, `world_fs.enforcement`, `world_fs.read`, `world_fs.discover`, and `world_fs.write` MUST be omitted. If any are present, they MUST be rejected as invalid config (hard error).

Rationale (user-facing):
- Workspace isolation does not support granular allow/deny/discover controls. Rejecting these keys (instead of ignoring them) prevents a false sense of protection when users switch from `full` to `workspace`.
Remediation:
- Either remove `enforcement/read/discover/write` from the policy when using `isolation=workspace`, or set `isolation=full`.

## Exit codes
- Taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- “Hard error” means: Substrate MUST NOT execute the command; it MUST fail during policy validation or snapshot validation.
  - Host CLI: exit code `2`.
  - World-agent HTTP `/v1/execute`: HTTP `400` with JSON body `{ "error": "<...>" }` (see `PROTOCOL.md`).
  - World-agent WebSocket `/v1/stream` `start_session`: send a fatal `{"type":"error", ...}` frame and close the connection (see `PROTOCOL.md`).
- Policy/config errors:
  - `2`: invalid schema / invalid pattern / disallowed key combination (e.g., deny under workspace)
- World enforcement failures:
  - `4`: full isolation enforcement failed while required (e.g., strict lockdown prerequisites not met and `world_fs.require_world=true`)

## Platform guarantees
- Linux:
  - Deny enforcement is supported only in `world_fs.isolation=full`.
  - `world_fs.enforcement=strict` MUST ensure deny rules are a hard security boundary (the workload cannot undo mount-based deny masks).
- macOS:
  - Out of scope for this ADR; if strict deny is requested and cannot be enforced, execution must fail closed when `require_world=true`.
- Windows:
  - Out of scope for this ADR; if strict deny is requested and cannot be enforced, execution must fail closed when `require_world=true`.

## Protected paths / invariants
- Deny masks MUST be applied inside the per-command mount namespace and MUST occur before executing user code.
- In strict mode, the workload MUST NOT retain the ability to call mount/umount APIs or otherwise undo deny masks within the mount namespace.
- Deny masks MUST apply to all nameable in-world project views in full isolation, including both `/project/...` and `$SUBSTRATE_MOUNT_PROJECT_DIR/...`.

## Denied operation semantics (deterministic)
When a filesystem operation is denied by policy (via deny masks and/or Landlock):
- `discover` denies MUST manifest as `EACCES` (`Permission denied`) for directory traversal/listing/metadata operations requiring directory visibility.
- `read` denies MUST manifest as `EACCES` (`Permission denied`) for file reads.
- `write` denies MUST manifest as `EROFS` (`Read-only file system`) for write/modify/delete operations.

When a mount-family syscall is blocked by `world_fs.enforcement=strict`:
- The blocked syscall MUST fail with `EPERM` (`Operation not permitted`) and MUST NOT terminate the process (no `SIGSYS`-kill behavior).
