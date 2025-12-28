# PCM0 — Workspace + Config Inventory and CLI (ADR-0003)

## Scope (authoritative)
Implement the ADR-0003 workspace and config contract:

### Workspace discovery (walk-up)
- Workspace root is the nearest ancestor (including `cwd`) containing `.substrate/workspace.yaml`.
- Discovery walks parents to filesystem root and selects the first match.
- Discovery never merges multiple workspaces.
- If no match exists, the process is “not in a workspace”.

### Workspace initialization
Command:
- `substrate workspace init [PATH] [--force]`

Behavior:
- `PATH` defaults to `.`.
- Nested workspace refusal:
  - If any parent directory of `PATH` contains `.substrate/workspace.yaml`, the command exits `2` and performs no filesystem writes.
- On success, ensure these paths exist:
  - `PATH/.substrate/`
  - `PATH/.substrate/workspace.yaml`
  - `PATH/.substrate/policy.yaml`
  - `PATH/.substrate-git/`
  - `PATH/.substrate-git/repo.git/`
- File creation rules:
  - If `PATH/.substrate/workspace.yaml` is missing, write default config YAML matching ADR-0003 schema.
  - If `PATH/.substrate/policy.yaml` is missing, write the built-in default policy YAML matching ADR-0003 schema.
  - Existing files are never overwritten; `--force` repairs only missing directories/files.
- `.gitignore` rules:
  - Create or update `PATH/.gitignore` to include these exact ignore rules (order does not matter; duplicates are allowed):
    - `.substrate-git/`
    - `.substrate/*`
    - `!.substrate/workspace.yaml`
    - `!.substrate/policy.yaml`

Exit codes:
- `0`: success (including already initialized).
- `2`: nested workspace refusal or invalid `PATH`.
- `1`: unexpected failure.

### Config schema (YAML; strict)
Config YAML schema is identical for:
- global config at `$SUBSTRATE_HOME/config.yaml`
- workspace config at `<workspace_root>/.substrate/workspace.yaml`

Schema (exact keys):
```yaml
world:
  enabled: true
  anchor_mode: workspace
  anchor_path: ""
  caged: true

policy:
  mode: observe

sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
```

Strict parsing:
- Unknown keys are a hard error.
- Type mismatches are a hard error.

Legacy artifact rejection (hard error):
- Presence of `<workspace_root>/.substrate/settings.yaml` is a hard error with an actionable message directing the user to `.substrate/workspace.yaml`.

Protected excludes (always-on):
- Effective `sync.exclude` always includes these patterns and they cannot be removed by config, env, or CLI:
  - `.git/**`
  - `.substrate/**`
  - `.substrate-git/**`

### Config discovery and precedence (effective config)
For a given `cwd`:
1. Resolve `$SUBSTRATE_HOME`:
   - If `SUBSTRATE_HOME` is set and non-empty, use it.
   - Else, use `~/.substrate`.
2. Read global config from `$SUBSTRATE_HOME/config.yaml` if present; otherwise use built-in defaults.
3. Resolve `<workspace_root>` for `cwd`.
4. If `<workspace_root>` exists, read workspace config from `<workspace_root>/.substrate/workspace.yaml`.
5. Merge with precedence (highest to lowest):
   - CLI flags (only the subset of keys with CLI flags)
   - Environment variables
   - Workspace config
   - Global config
   - Built-in defaults

### Config CLI (workspace scope)
Workspace-scope commands require `<workspace_root>`; if missing they exit `2` with an error that directs the user to `substrate workspace init`.

- `substrate config show [--json]`
  - Default output: YAML mapping matching the schema.
  - `--json` output: JSON object equivalent to the YAML mapping.
- `substrate config set [--json] UPDATE...`
  - Applies updates to `<workspace_root>/.substrate/workspace.yaml` only.

Exit codes:
- `0`: success (including no-op).
- `2`: unsupported key/operator, invalid value/type, or missing workspace.
- `1`: unexpected failure.

### Config CLI (global scope)
- `substrate config global init [--force]`
  - Create `$SUBSTRATE_HOME/config.yaml` if missing.
  - Overwrite `$SUBSTRATE_HOME/config.yaml` if `--force` is present.
- `substrate config global show [--json]`
  - If `$SUBSTRATE_HOME/config.yaml` exists, print its contents.
  - If missing, print built-in defaults.
- `substrate config global set [--json] UPDATE...`
  - Apply updates to `$SUBSTRATE_HOME/config.yaml` (create the file if missing).

Exit codes:
- `0`: success (including no-op).
- `2`: unsupported key/operator or invalid value/type.
- `1`: unexpected failure.

### Update syntax (config set)
Each `UPDATE` is one of:
- `key=value`
- `key+=value` (list append)
- `key-=value` (list remove; exact string match)

Allowed update targets:
- `world.enabled`
- `world.anchor_mode`
- `world.anchor_path`
- `world.caged`
- `policy.mode`
- `sync.auto_sync`
- `sync.direction`
- `sync.conflict_policy`
- `sync.exclude` (list only)

Value parsing rules:
- Booleans: `true|false|1|0|yes|no|on|off` (case-insensitive).
- Enums: case-insensitive and must match the schema values.
- Strings: raw string after operator.
- `sync.exclude` with `=`: value is a YAML list literal.

## Non-scope (explicit)
- Policy schema and policy CLI (`PCM1`).
- Policy evaluation and routing semantics (`PCM2`).
- Env scripts and world enable semantics (`PCM3`).

## Acceptance (implementation outcomes)
- Workspace init and discovery semantics match this spec and ADR-0003 exactly.
- Config strict parsing rejects unknown keys, type mismatches, and legacy `.substrate/settings.yaml`.
- Effective config precedence matches ADR-0003 for every schema key, including list `+=` and `-=` behavior for `sync.exclude`.
- Protected excludes are always present in effective config and cannot be removed.

