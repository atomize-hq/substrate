# PCM1 — Policy Inventory and CLI (ADR-0003)

## Scope (authoritative)
Implement the ADR-0003 policy contract: schema, strict parsing, invariants, discovery, and CLI commands.

### Policy schema (YAML; strict)
Policy YAML schema is identical for:
- global policy at `$SUBSTRATE_HOME/policy.yaml`
- workspace policy at `<workspace_root>/.substrate/policy.yaml`

Schema (exact keys):
```yaml
id: "string"
name: "string"

world_fs:
  mode: writable
  isolation: project
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []

net_allowed: []

cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
```

Strict parsing:
- Unknown keys are a hard error.
- Type mismatches are a hard error.

Policy invariants (validated at load time; hard error on violation):
- `world_fs.isolation=full` and `world_fs.require_world=false`
- `world_fs.mode=read_only` and `world_fs.require_world=false`

Command pattern semantics (`cmd_allowed`, `cmd_denied`, `cmd_isolated`):
- If a pattern contains `*`, treat it as a glob pattern.
- Otherwise, treat it as a substring match.

### Policy discovery (effective policy selection)
For a given `cwd`:
1. Resolve `$SUBSTRATE_HOME`.
2. Resolve `<workspace_root>` for `cwd`.
3. If `<workspace_root>` exists and `<workspace_root>/.substrate/policy.yaml` exists, load it.
4. Else if `$SUBSTRATE_HOME/policy.yaml` exists, load it.
5. Else, use the built-in default policy.

### Policy CLI (workspace scope)
Workspace-scope commands require `<workspace_root>`; if missing they exit `2` with an error that directs the user to `substrate workspace init`.

- `substrate policy init [--force]`
  - Create `<workspace_root>/.substrate/policy.yaml` if missing.
  - With `--force`, rewrite it to the built-in default policy.
- `substrate policy show [--json]`
  - Print the effective policy selected by policy discovery.
- `substrate policy set [--json] UPDATE...`
  - Apply updates to `<workspace_root>/.substrate/policy.yaml` only.

Exit codes:
- `0`: success (including no-op).
- `2`: unsupported key/operator, invalid value/type, invalid invariants, or missing workspace.
- `1`: unexpected failure.

### Policy CLI (global scope)
- `substrate policy global init [--force]`
- `substrate policy global show [--json]`
- `substrate policy global set [--json] UPDATE...`

Global commands operate on `$SUBSTRATE_HOME/policy.yaml` (create if missing).

Exit codes:
- `0`: success (including no-op).
- `2`: unsupported key/operator, invalid value/type, or invalid invariants.
- `1`: unexpected failure.

### Update syntax (policy set)
Each `UPDATE` is one of:
- `key=value`
- `key+=value` (list append)
- `key-=value` (list remove; exact string match)

Allowed update targets:
- `id`, `name`
- `world_fs.mode`, `world_fs.isolation`, `world_fs.require_world`
- `world_fs.read_allowlist`, `world_fs.write_allowlist`
- `net_allowed`
- `cmd_allowed`, `cmd_denied`, `cmd_isolated`
- `require_approval`, `allow_shell_operators`
- `limits.max_memory_mb`, `limits.max_cpu_percent`, `limits.max_runtime_ms`, `limits.max_egress_bytes`
- `metadata` (full replace only; `metadata+=` and `metadata-=` are forbidden)

Value parsing rules:
- Booleans: `true|false|1|0|yes|no|on|off` (case-insensitive).
- Enums: case-insensitive and must match the schema values.
- Numbers: base-10 integers.
- Strings: raw string after operator.
- For list keys with `=`: value is a YAML list literal.

## Non-scope (explicit)
- Policy mode semantics and runtime routing (`PCM2`).
- Env scripts and world enable semantics (`PCM3`).

## Acceptance (implementation outcomes)
- Policy YAML strict parsing and invariants match this spec and ADR-0003 exactly.
- Policy discovery selection order matches this spec and ADR-0003 exactly.
- Policy CLI init/show/set/global init/show/set semantics match this spec and ADR-0003 exactly.

