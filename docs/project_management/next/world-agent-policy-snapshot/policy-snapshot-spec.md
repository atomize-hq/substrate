# Policy Snapshot Spec — World-Agent Enforcement Inputs (PolicySnapshotV1)

Related decisions (authoritative):
- `docs/project_management/next/world-agent-policy-snapshot/decision_register.md` (DR-0001..DR-0011)

Related architecture docs:
- `docs/project_management/next/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
- `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`
- `docs/WORLD.md`
- `docs/TRACE.md`

## Scope
- Define the `PolicySnapshotV1` payload carried from host → world-agent for enforcement inputs.
- Define validation rules for `PolicySnapshotV1`.
- Define the enforcement boundary between host and world-agent.
- Define deterministic refresh contracts for policy and config on the host so changes are visible on the next executed command.
- Define the multi-user posture and the operator boundary (socket ACL).
- Define the cross-platform parity contract (Linux/macOS/WSL) and explicit fallback rules.

## Non-goals
- Designing a new policy language or changing user-facing policy keys.
- Introducing in-protocol authentication/authorization for world-agent requests.
- Defining a full trace schema migration (this spec only defines required snapshot-related fields).

## Definitions
- **Host**: the Substrate shell/shim layer on the developer workstation that evaluates command policy via `crates/broker`.
- **World-agent**: the privileged execution backend reached over a local transport (`/run/substrate.sock` on Linux; forwarded equivalents on macOS/WSL).
- **Effective merged policy**: defaults overlaid by `$SUBSTRATE_HOME/policy.yaml` and (when a workspace exists) `<workspace_root>/.substrate/policy.yaml`.
- **Policy snapshot**: a versioned, minimal payload containing only the enforcement inputs world-agent needs.
- **Legacy local policy resolution**: world-agent resolving policy internally (broker reads inside the agent process).

## Enforcement Boundary (authoritative)
- The host MUST enforce command policy decisions (`cmd_allowed`, `cmd_denied`, `cmd_isolated`, `require_approval`, `allow_shell_operators`) via the broker before world-agent execution is attempted. (DR-0009)
- The world-agent MUST NOT implement command allow/deny/isolation/approval decisions and MUST NOT accept `cmd_*` fields in a snapshot payload. (DR-0003, DR-0009)
- The world-agent MUST enforce only:
  - `world_fs` (filesystem mode/isolation/require_world + allowlists),
  - `net_allowed` (network allowlist),
  - `limits` (resource limits). (DR-0003)
- A direct call to world-agent that bypasses the host is outside the command policy enforcement boundary; operators MUST treat world-agent socket access as privileged. (DR-0001)

### Manual regression check: `cmd_denied` enforcement is host-only (exit 126)
Linux-only sanity check that proves `cmd_denied` is enforced before any world-agent routing is attempted.

```bash
export SUBSTRATE_HOME="$(mktemp -d)"

cat >"$SUBSTRATE_HOME/config.yaml" <<'YAML'
world:
  enabled: true
  anchor_mode: follow-cwd
  anchor_path: ""
  caged: false

policy:
  mode: enforce
YAML

cat >"$SUBSTRATE_HOME/policy.yaml" <<'YAML'
world_fs:
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist: ["*"]
  write_allowlist: []

cmd_denied:
  - "echo*"
YAML

export SUBSTRATE_OVERRIDE_WORLD=enabled
export SUBSTRATE_OVERRIDE_POLICY_MODE=enforce
export SUBSTRATE_WORLD_SOCKET="/tmp/substrate-waps-0007-nonexistent.sock"

substrate -c 'echo __waps_0007__'
echo "exit=$?"
```

Expected:
- Exit code is `126`.
- `__waps_0007__` is not printed (command does not execute).
- The command does not fail with a world backend error despite `require_world=true` and a nonexistent socket path.

## Authorization Boundary (operator contract)
- The authorization boundary for world-agent requests MUST be the OS-level transport ACL (Linux: Unix socket ownership/mode). (DR-0001)
- On Linux, `/run/substrate.sock` MUST be owned by `root:substrate` with mode `0660`. (DR-0001)
- Operators MUST treat membership in the socket group (`substrate`) as equivalent to granting access to the privileged execution backend.

## Multi-user posture (Linux)
- The supported Linux deployment model is a shared system service (root-run) gated by socket ACLs; Substrate does not require per-user world-agent services for this feature. (DR-0007)
- Operators MUST ensure socket ownership/mode is correct:
  - `sudo ls -l /run/substrate.sock` (expected: `root substrate` and `srw-rw----` / `0660`).
- Operators MUST ensure only intended users are in the socket group:
  - `id -nG "$USER"` to inspect membership.
  - `sudo usermod -aG substrate <user>` to grant access (requires re-login to take effect).
- On systemd hosts, operators SHOULD provision and validate using the standard workflow in `docs/WORLD.md` (socket activation, group creation, and service readiness).

## PolicySnapshotV1 (schema)

### Snapshot placement (Agent API request)
- A world-agent execute/stream request MUST carry `policy_snapshot` when world-agent enforcement is used and the agent supports snapshot ingestion. (DR-0011)
- `policy_snapshot` MUST be omitted (not present) when the host is not using world-agent for that command.
- `policy_snapshot` is an additive request field; the host MUST send it only when world-agent capability detection indicates snapshot ingestion support.

### Snapshot support detection (doctor report)
The host determines snapshot support using agent capability/doctor endpoints. `GET /v1/doctor/world` returns a world enforcement report with `schema_version: 2` and includes snapshot-related fields:

```json
{
  "schema_version": 2,
  "ok": true,
  "collected_at_utc": "2026-01-08T00:00:00Z",
  "policy_snapshot_v1_supported": true,
  "policy_resolution_mode": "snapshot_v1",
  "landlock": { "supported": true, "abi": 3, "reason": null },
  "world_fs_strategy": {
    "primary": "overlay",
    "fallback": "fuse",
    "probe": { "id": "enumeration_v1", "probe_file": ".substrate_enum_probe", "result": "pass", "failure_reason": null }
  }
}
```

Legacy world-agents may still return `schema_version: 1` and omit `policy_snapshot_v1_supported` / `policy_resolution_mode`; clients MUST default these fields safely.

### Schema (JSON shape)
```json
{
  "policy_snapshot": {
    "schema_version": 1,
    "world_fs": {
      "mode": "writable",
      "isolation": "workspace",
      "require_world": false,
      "read_allowlist": ["*"],
      "write_allowlist": []
    },
    "net_allowed": [],
    "limits": {
      "max_memory_mb": null,
      "max_cpu_percent": null,
      "max_runtime_ms": null,
      "max_egress_bytes": null
    }
  }
}
```

### Field definitions
- `schema_version`:
  - Type: integer
  - Required: yes
  - Allowed values: `1`

- `world_fs`:
  - Type: object
  - Required: yes
  - Fields:
    - `mode`: `"writable"` | `"read_only"`
    - `isolation`: `"workspace"` | `"full"`
    - `require_world`: boolean
    - `read_allowlist`: list of strings
    - `write_allowlist`: list of strings

- `net_allowed`:
  - Type: list of strings
  - Required: yes
  - Semantics: allowed host/domain patterns used by world network filtering.

- `limits`:
  - Type: object
  - Required: yes (keys optional)
  - Keys:
    - `max_memory_mb`: integer or null (null means “unset”)
    - `max_cpu_percent`: integer or null (null means “unset”)
    - `max_runtime_ms`: integer or null (null means “unset”)
    - `max_egress_bytes`: integer or null (null means “unset”)

## PolicySnapshotV1 validation (authoritative)

### Structural validation (world-agent)
- If `policy_snapshot` is present, world-agent MUST reject the request when:
  - `schema_version` is missing or not `1`.
  - Any required field is missing (`world_fs`, `net_allowed`, `limits`).
  - Any field has the wrong type (e.g., non-string allowlist entries).
  - Any unknown fields are present in `policy_snapshot` or its nested objects.

### Invariant validation (host and world-agent)
- `world_fs.mode="read_only"` MUST imply `world_fs.require_world=true`.
- `world_fs.isolation="full"` MUST imply `world_fs.require_world=true`.
- Each entry in `world_fs.read_allowlist`, `world_fs.write_allowlist`, and `net_allowed` MUST be a non-empty string after trimming whitespace.
- When present and non-null, each `limits.*` value MUST be an integer strictly greater than `0`.

### Canonicalization boundary (world-agent)
PolicySnapshotV1 contains merged patterns, not enforcement-ready paths. (DR-0004)

When applying full isolation enforcement, the world-agent MUST canonicalize allowlist patterns into enforcement inputs using only:
- the snapshot allowlist strings, and
- the agent-known project root (`project_dir`) for the request.

Canonicalization rules (deterministic):
- Absolute allowlist patterns are honored only when they refer to the project root or a descendant of it; other absolute patterns MUST be ignored for enforcement.
- Any allowlist entry containing a `..` path segment MUST be ignored for enforcement.
- Glob/meta characters (`*`, `?`, `[...]`) MUST be reduced to the directory prefix up to the first meta character when computing:
  - writable mount prefixes (project write allowlist),
  - Landlock read/write path allowlists.
- `"*"`/`"**"` (and equivalent root globs) MUST map to the project root.

## Host policy snapshot generation contract (authoritative)
- For each command routed to world-agent, the host MUST compute the effective merged policy for the command’s `cwd` and generate a `PolicySnapshotV1` from it. (DR-0002)
- The host MUST generate a minimal snapshot containing only: `world_fs`, `net_allowed`, `limits`. (DR-0003)
- The host MUST NOT include command policy keys (`cmd_*`, approval-related keys) in `PolicySnapshotV1`. (DR-0003, DR-0009)
- The host MUST attach the snapshot to the world-agent request when the agent indicates snapshot support. (DR-0011)

## Host refresh contracts (policy + config)

### Policy refresh contract (host resolver)
- The host MAY cache the effective merged policy, but MUST preserve next-command visibility for:
  - external file edits to policy patch files, and
  - in-process policy writes (`policy global|workspace set|reset|init`). (DR-0006)

Cache key (policy):
- workspace root identity (or “no workspace”),
- global policy patch path: `$SUBSTRATE_HOME/policy.yaml`,
- workspace policy patch path (when a workspace exists): `<workspace_root>/.substrate/policy.yaml`.

Invalidation triggers (policy):
- Before each executed command, the host MUST `stat` the relevant policy patch file(s) (existence + mtime + size) and invalidate the cache when any metadata changes. (DR-0006)
- After an in-process write completes for `policy global|workspace set|reset|init`, the host MUST invalidate the policy cache immediately (even if `stat`-based invalidation would also catch it). (DR-0006)

### Config refresh contract (parity with policy)
- The host MAY cache the effective merged config, but MUST preserve next-command visibility for:
  - external file edits to config patch files, and
  - in-process config writes (`config global|workspace set|reset|init`). (DR-0010)

Cache key (config):
- workspace root identity (or “no workspace”),
- global config patch path: `$SUBSTRATE_HOME/config.yaml`,
- workspace config patch path (when a workspace exists): `<workspace_root>/.substrate/workspace.yaml`.

Invalidation triggers (config):
- Before each executed command, the host MUST `stat` the relevant config patch file(s) (existence + mtime + size) and invalidate the cache when any metadata changes. (DR-0010)
- After an in-process write completes for `config global|workspace set|reset|init`, the host MUST invalidate the config cache immediately. (DR-0010)

## Observability contract (trace + doctor)

### Required trace fields
For every executed command, trace output MUST record:
- `policy_resolution_mode`: `"snapshot_v1"` | `"legacy_local"` (DR-0008)
- `policy_snapshot_schema`: integer or null
  - MUST be `1` when `policy_resolution_mode="snapshot_v1"`.
  - MUST be null/omitted when `policy_resolution_mode="legacy_local"`.
- `policy_snapshot_hash`: string or null
  - MUST be present and non-empty when `policy_resolution_mode="snapshot_v1"`.
  - MUST be null/omitted when `policy_resolution_mode="legacy_local"`.

### Snapshot hash rules
- `policy_snapshot_hash` MUST be computed without logging raw snapshot contents.
- `policy_snapshot_hash` MUST be deterministic for the same `PolicySnapshotV1` value.

### Doctor visibility (world scope)
- `GET /v1/doctor/world` (and `substrate world doctor --json`) MUST report:
  - whether the connected world-agent supports PolicySnapshotV1 ingestion, and
  - the active policy resolution mode used by the agent (`snapshot_v1` vs `legacy_local`). (DR-0008, DR-0011)

## Cross-platform parity contract (Linux/macOS/WSL)

### Parity guarantees (authoritative)
- On every platform where world-agent is used, the host MUST attach `PolicySnapshotV1` to world-agent requests when the agent indicates support. (DR-0011)
- When a valid snapshot is present, world-agent MUST ignore legacy local policy resolution for enforcement inputs. (DR-0011)

### Fallback rules (explicit)
- If the agent does not support PolicySnapshotV1 ingestion:
  - If `world_fs.require_world=true` in the effective policy, the host MUST fail closed (no host fallback execution).
  - If `world_fs.require_world=false` in the effective policy, the host MAY fall back to host execution and MUST emit exactly one warning for that command. (aligns with `docs/WORLD.md`)
- If `policy_snapshot` is present but invalid:
  - World-agent MUST reject the request deterministically (HTTP 400 / bad request classification).
  - The host MUST treat this as a hard error for the command (no silent fallback execution).

## Acceptance criteria (spec-level)
- The spec defines PolicySnapshotV1 fields and validation rules.
- The spec defines the host/world-agent enforcement boundary with an explicit “cmd policy is host-only” statement.
- The spec defines the policy + config refresh contracts with deterministic invalidation and next-command visibility.
- The spec defines the operator boundary as the socket ACL and includes multi-user guidance.
- The spec defines cross-platform parity and explicit fallback rules.
