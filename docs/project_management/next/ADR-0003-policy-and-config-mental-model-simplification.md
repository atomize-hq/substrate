# ADR-0003 — Policy + Config Mental Model Simplification

## Status

- Status: Accepted
- Date (UTC): 2025-12-27
- Owner(s): spenser

## Scope

- Feature directory: `docs/project_management/next/policy_and_config_mental_model_simplification/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Related Docs

- Sequencing: `docs/project_management/next/sequencing.json`
- Plan: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`
- Tasks: `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`
- Session log: `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`
- Specs:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`
- Decision Register: `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`
- Integration map: `docs/project_management/next/policy_and_config_mental_model_simplification/integration_map.md`
- Manual playbook: `docs/project_management/next/policy_and_config_mental_model_simplification/manual_testing_playbook.md`
- Prior (untemplated) ADR content preserved verbatim:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification_OG.md`
- World-sync specs that MUST be updated to align with this ADR before `world_sync` implementation:
  - `docs/project_management/next/world-sync/C0-spec.md`
  - `docs/project_management/next/world-sync/C1-spec.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: e8d328d623951d68134863af5a22efa7ef683f360ce0518038aac10a223d5f7a

ADR_BODY_SHA256: <run `python3 scripts/planning/check_adr_exec_summary.py --adr <this-file> --fix` after editing>

- Existing: Config/policy discovery and naming is ambiguous (multiple file names, “settings vs config vs policy”, and colliding terms like `cage/caged` and `root/anchor`).
- New: One strict, testable inventory of config/policy files + precedence, with explicit removals and no legacy fallbacks.
- Why: Shrinks the mental model, reduces bug surface, and increases reproducibility (operators can predict exactly what is loaded).
- Links:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`

- Existing: Policy mode semantics and runtime state can be drift-prone/hard to reason about.
- New: Policy mode is explicit (`disabled|observe|enforce`), strict parsing is required, and cached state is stabilized via `env.sh`.
- Why: Makes enforcement reliable and removes footguns for agent workflows.
- Links:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`

## Problem / Context

Substrate currently has overlapping configuration/policy artifacts and naming collisions that make the runtime mental model hard to reason about. Examples include:

- Multiple competing “policy” and “settings” concepts (config vs settings vs profile vs policy).
- Multiple on-disk file names for “policy” with different schemas and behaviors.
- Naming collisions:
  - `world.caged` / `--caged` is a shell roaming guard.
  - `world_fs.cage` historically referred to filesystem isolation.
  - “root” vs “anchor” existed simultaneously.
- Drift-prone runtime state:
  - Installers/world-enable can write world state exports into `manager_env.sh`, but runtime overwrites that file with a different script, causing “cached state” to be unreliable.
- Ambiguous world enable semantics:
  - `--prefix` influences scripts/logs, while metadata historically wrote under `$SUBSTRATE_HOME`, enabling split-brain behavior.

This ADR defines a cleaner, stricter model with a single mental model and strict contracts.

## Goals

- Define a single, explicit mental model for global defaults vs workspace overrides.
- Make all on-disk configuration and policy locations unambiguous and testable.
- Remove naming collisions by:
  - removing all “root” naming (anchor-only),
  - removing filesystem-isolation “cage” naming (isolation-only),
  - keeping `--caged/--uncaged` as roaming-guard-only terminology.
- Make policy mode explicit and unambiguous: `disabled|observe|enforce`.
- Make cached state stable via `env.sh` and prevent runtime drift.
- Make `world enable` semantics unambiguous via `--home`.
- Lock the model in with guardrail tests (precedence + discovery + removal constraints).

## Non-Goals

- Implementing world-sync functionality itself (this ADR only defines prerequisites and updated contracts that world-sync must adopt).
- Preserving any backward compatibility, migration tooling, aliases, or legacy discovery fallbacks.
- Adding additional configuration/policy surfaces beyond what is explicitly specified here.

## User Contract (Authoritative)

### Hard rules (non-negotiable)

1. **No backward compatibility**:
   - No CLI aliases, env var aliases, schema aliases, or file name fallbacks.
   - Any code that exists solely for legacy support MUST be removed.
2. **Strict parsing**:
   - Unknown keys MUST be a hard error (config and policy).
   - Type mismatches MUST be a hard error (config and policy).
3. **Single source of truth**:
   - Only the files listed in this ADR are valid configuration/policy inputs.

### Terminology

#### Substrate Home (`$SUBSTRATE_HOME`)

`$SUBSTRATE_HOME` MUST be resolved as:

- If `SUBSTRATE_HOME` is set and non-empty: use it.
- Else: use `~/.substrate`.

#### Workspace Root (`<workspace_root>`)

The workspace root is the nearest ancestor directory (including the current directory) that contains:

- `.substrate/workspace.yaml`

Workspace discovery MUST:

- Walk parent directories up to the filesystem root.
- Select the first match (nearest ancestor).
- Never merge multiple workspaces.

If no match exists, the current directory is “not in a workspace”.

### Canonical on-disk inventory

#### Global (system-wide defaults)

- `$SUBSTRATE_HOME/config.yaml`
- `$SUBSTRATE_HOME/policy.yaml`
- `$SUBSTRATE_HOME/env.sh`
- `$SUBSTRATE_HOME/manager_env.sh`

#### Workspace (repo/directory-scoped overrides)

- `<workspace_root>/.substrate/workspace.yaml`
- `<workspace_root>/.substrate/policy.yaml`

#### Explicit removals (must not exist in code)

The following legacy artifacts MUST be removed from loader/CLI logic:

- `.substrate/settings.yaml`
- `.substrate-profile`
- `.substrate-profile.d/*`
- `.substrate-policy.yaml`
- Any TOML-era config/settings files (`config.toml`, `.substrate/settings.toml`, etc.)
- Any legacy “root” fields/flags/env vars (see “Removed names”)
- Any legacy filesystem-isolation “cage” naming (see “Filesystem isolation rename”)
- Any `--prefix` / `SUBSTRATE_PREFIX` semantics for home selection (see “World enable home semantics”)

### Exit codes

Exit code taxonomy reference (unless overridden here): `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.

This ADR explicitly defines exit codes for the commands it specifies:

- `0`: success (including no-op “already up to date” behavior where applicable)
- `2`: user/actionable error (missing workspace, invalid key/value/type, invalid path, nested-workspace refusal, unsupported flag/operator)
- `1`: unexpected failure

### Workspace initialization

#### Command

`substrate workspace init [PATH] [--force]`

Arguments:

- `PATH`: workspace root directory to initialize; if omitted, it defaults to `.`.

Flags:

- `--force`: repair missing pieces idempotently, without deleting or overwriting user data.

No other flags are permitted by this ADR.

Nested workspace refusal:

- If any parent directory of `PATH` contains `.substrate/workspace.yaml`, `substrate workspace init` MUST:
  - exit with status `2`, and
  - perform no filesystem writes.

Filesystem effects (success path):

On success, `substrate workspace init` MUST ensure these paths exist:

- `PATH/.substrate/`
- `PATH/.substrate/workspace.yaml`
- `PATH/.substrate/policy.yaml`
- `PATH/.substrate-git/`
- `PATH/.substrate-git/repo.git/` (internal git scaffold directory; no interaction with the user repo’s `.git`)

File creation rules:

- If `PATH/.substrate/workspace.yaml` is missing: write defaults (see “Config schema”).
- If `PATH/.substrate/policy.yaml` is missing: write default policy (see “Policy schema”).
- If either file exists:
  - Without `--force`: do not modify it.
  - With `--force`: still do not overwrite it; only repair missing files/directories elsewhere.

`.gitignore` rules (required):

`substrate workspace init` MUST create or update `PATH/.gitignore` such that it includes these exact ignore rules (order does not matter; duplicates are allowed but discouraged):

- `.substrate-git/`
- `.substrate/*`
- `!.substrate/workspace.yaml`
- `!.substrate/policy.yaml`

No other `.gitignore` behavior is permitted by this ADR.

Exit codes:

- `0`: initialized successfully (including “already initialized”).
- `2`: nested workspace refusal or invalid `PATH`.
- `1`: unexpected failure.

### Config

#### Config schema (YAML; strict)

Both global config and workspace config MUST be YAML mappings with exactly these keys:

```yaml
world:
  enabled: true                 # bool
  anchor_mode: workspace        # workspace | follow-cwd | custom
  anchor_path: ""               # string path; required when anchor_mode=custom
  caged: true                   # bool (roaming guard)

policy:
  mode: observe                 # disabled | observe | enforce

sync:
  auto_sync: false              # bool
  direction: from_world         # from_world | from_host | both
  conflict_policy: prefer_host  # prefer_host | prefer_world | abort
  exclude: []                   # list[string] of glob patterns
```

Unknown keys MUST be a hard error.

Protected sync excludes (always-on):

`sync.exclude` MUST always behave as if the following protected patterns are prepended (and cannot be removed by config/env/CLI):

- `.git/**`
- `.substrate/**`
- `.substrate-git/**`

#### Config discovery and precedence (effective config)

For a given `cwd`, the effective config MUST be computed as:

1. Resolve `$SUBSTRATE_HOME`.
2. Read global config from `$SUBSTRATE_HOME/config.yaml` if it exists; otherwise use built-in defaults.
3. Resolve `<workspace_root>` for `cwd`.
4. If `<workspace_root>` exists, read workspace config from `<workspace_root>/.substrate/workspace.yaml`.
5. Merge with precedence (highest to lowest):
   - CLI flags (only the subset of keys with CLI flags)
   - Environment variables
   - Workspace config
   - Global config
   - Built-in defaults

Config parsing MUST be strict:

- Unknown keys MUST be a hard error.
- Type mismatches MUST be a hard error.

#### Config CLI

Workspace scope:

Workspace-scope commands MUST require `<workspace_root>` to exist; otherwise they MUST exit `2` with an error directing the user to `substrate workspace init`.

- `substrate config show [--json]`
  - Prints the effective config for `cwd` (YAML default, JSON with `--json`).
- `substrate config set [--json] UPDATE...`
  - Applies updates to `<workspace_root>/.substrate/workspace.yaml` only.

Global scope:

- `substrate config global init [--force]`
  - Creates `$SUBSTRATE_HOME/config.yaml` if missing; overwrites if `--force`.
- `substrate config global show [--json]`
  - Prints the contents of `$SUBSTRATE_HOME/config.yaml` if present; otherwise prints built-in defaults.
- `substrate config global set [--json] UPDATE...`
  - Applies updates to `$SUBSTRATE_HOME/config.yaml` (creating the file if missing).

`UPDATE` syntax (config set):

Each `UPDATE` MUST be one of:

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
- `sync.exclude` (list-only; `+=` and `-=` are allowed; `=` is allowed only with a YAML list literal)

Value parsing rules:

- Booleans: `true|false|1|0|yes|no|on|off` (case-insensitive).
- Enums: case-insensitive, must match the allowed values in the schema.
- Strings: the raw string after the operator.
- For `sync.exclude` with `=`: the value MUST be a YAML list literal (e.g., `[]`, `["a","b"]`).

Exit codes (`config show/set`):

- `0`: success (including no-op).
- `2`: unsupported key/operator, invalid value/type, or missing workspace (workspace scope).
- `1`: unexpected failure.

### Policy

#### Policy schema (YAML; strict)

Both global policy and workspace policy MUST be YAML mappings with exactly this schema:

```yaml
id: "string"
name: "string"

world_fs:
  mode: writable            # writable | read_only
  isolation: project        # project | full
  require_world: false      # bool
  read_allowlist: ["*"]     # list[string] (glob patterns)
  write_allowlist: []       # list[string] (glob patterns)

net_allowed: []             # list[string]

cmd_allowed: []             # list[string] (command pattern semantics below)
cmd_denied: []              # list[string] (command pattern semantics below)
cmd_isolated: []            # list[string] (command pattern semantics below)

require_approval: false     # bool
allow_shell_operators: true # bool

limits:                     # object (required; individual fields may be null)
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}                # map[string]string (required; may be empty)
```

Unknown keys MUST be a hard error.

Policy invariants (must be validated):

Policy load MUST fail (hard error) if any of the following are true:

- `world_fs.isolation=full` AND `world_fs.require_world=false`
- `world_fs.mode=read_only` AND `world_fs.require_world=false`

Command pattern semantics (`cmd_*`):

For `cmd_allowed`, `cmd_denied`, and `cmd_isolated`:

- If the pattern contains `*`, it MUST be treated as a glob pattern.
- Otherwise, it MUST be treated as a substring match.

No other pattern semantics are permitted by this ADR.

#### Policy discovery (effective policy)

For a given `cwd`, effective policy MUST be selected as:

1. Resolve `$SUBSTRATE_HOME`.
2. Resolve `<workspace_root>` for `cwd`.
3. If `<workspace_root>` exists and `<workspace_root>/.substrate/policy.yaml` exists: load it.
4. Else if `$SUBSTRATE_HOME/policy.yaml` exists: load it.
5. Else: use the built-in default policy.

Policy parsing MUST be strict:

- Unknown keys MUST be a hard error.
- Type mismatches MUST be a hard error.

#### Policy mode (`disabled|observe|enforce`)

`policy.mode` (from effective config) MUST control runtime behavior:

- `disabled`:
  - Substrate MUST NOT evaluate policy (no allow/deny decisions computed).
  - Trace/logging still occurs.
- `observe`:
  - Substrate MUST evaluate policy and record the decision in trace/telemetry.
  - Substrate MUST NOT block execution.
- `enforce`:
  - Substrate MUST evaluate policy.
  - Substrate MUST block and/or prompt according to policy.

Observe differs from disabled because observe records *policy decisions* (not just commands), enabling deterministic audit/replay and future agent planning.

How policy decisions apply (command-level):

When `policy.mode` is `observe` or `enforce`, policy evaluation MUST implement these semantics:

- `cmd_denied`:
  - First-match wins by evaluation order: if the command matches any `cmd_denied` pattern it is considered “denied”.
  - In `enforce`: Substrate MUST block execution.
  - In `observe`: Substrate MUST allow execution but MUST record “would deny”.
- `cmd_allowed`:
  - If `cmd_allowed` is empty: it imposes no allowlist restriction.
  - If `cmd_allowed` is non-empty and the command matches none of its patterns:
    - In `enforce`: Substrate MUST block execution.
    - In `observe`: Substrate MUST allow execution but MUST record “would deny”.
- `cmd_isolated`:
  - If the command matches any `cmd_isolated` pattern:
    - The command is marked “requires world” (see “World execution requirements” below).

When `policy.mode` is `disabled`, Substrate MUST NOT evaluate `cmd_*` lists at all.

World execution requirements (policy + config + CLI):

The decision “run in world vs host” MUST be made for each execution using:

1. The world selection inputs (highest to lowest):
   - CLI: `--world` / `--no-world`
   - Env: `SUBSTRATE_WORLD=enabled|disabled`
   - Config: `world.enabled`
2. Policy-derived “requires world” constraints (only when `policy.mode` is `observe` or `enforce`):
   - `world_fs.require_world=true` requires world
   - Any command match in `cmd_isolated` requires world
   - By invariants, `world_fs.mode=read_only` and `world_fs.isolation=full` also require world

Resolution rules:

- If any policy-derived “requires world” constraint applies:
  - In `enforce`: Substrate MUST run the command in world; if that is not possible it MUST fail closed (hard error).
  - In `observe`: Substrate MUST record “would require world” and MUST NOT change the world/host selection solely due to this requirement.
- If `--no-world` is provided:
  - In `enforce` and a “requires world” constraint applies: Substrate MUST fail closed (hard error).
  - Otherwise: Substrate MUST run on host.
- If `--world` is provided:
  - Substrate MUST attempt world execution for this run, regardless of config/env.

World backend availability rule:

- If Substrate determines that world execution is required (by `--world` or by policy-derived requirement in `enforce`), but the world backend is unavailable: Substrate MUST fail closed (hard error).
- If Substrate selects world execution (via config/env) but world execution is not required by `enforce`, and the world backend is unavailable: Substrate MUST fall back to host execution and MUST record the fallback reason in trace/telemetry.

Approvals “save to policy” writes (enforce mode):

If (and only if) enforce mode requests interactive approval and the user selects “save this approval to policy file”, Substrate MUST write to:

- If `<workspace_root>` exists: `<workspace_root>/.substrate/policy.yaml` (creating it if missing).
- Else: `$SUBSTRATE_HOME/policy.yaml` (creating it if missing).

No other write locations are permitted by this ADR.

#### Policy CLI

Workspace scope:

Workspace-scope commands MUST require `<workspace_root>` to exist; otherwise they MUST exit `2` with an error directing the user to `substrate workspace init`.

- `substrate policy init [--force]`
  - Creates `<workspace_root>/.substrate/policy.yaml` if missing.
  - With `--force`, rewrites it to the built-in default policy.
- `substrate policy show [--json]`
  - Prints the effective policy chosen by “Policy discovery”.
- `substrate policy set [--json] UPDATE...`
  - Applies updates to `<workspace_root>/.substrate/policy.yaml` only.

Global scope:

- `substrate policy global init [--force]`
- `substrate policy global show [--json]`
- `substrate policy global set [--json] UPDATE...`

Global commands operate on `$SUBSTRATE_HOME/policy.yaml` (creating it when needed).

`UPDATE` syntax (policy set):

Each `UPDATE` MUST be one of:

- `key=value`
- `key+=value` (list append)
- `key-=value` (list remove; exact string match)

Allowed update targets:

- `id`, `name`
- `world_fs.mode`, `world_fs.isolation`, `world_fs.require_world`
- `world_fs.read_allowlist`, `world_fs.write_allowlist` (list)
- `net_allowed` (list)
- `cmd_allowed`, `cmd_denied`, `cmd_isolated` (list)
- `require_approval`, `allow_shell_operators`
- `limits.max_memory_mb`, `limits.max_cpu_percent`, `limits.max_runtime_ms`, `limits.max_egress_bytes`
- `metadata` (full replace only; `metadata+=` and `metadata-=` are forbidden)

Value parsing rules:

- Booleans: `true|false|1|0|yes|no|on|off` (case-insensitive).
- Enums: case-insensitive, must match allowed values in schema.
- Numbers: parsed as base-10 integers.
- Strings: the raw string after the operator.
- For list keys with `=`: the value MUST be a YAML list literal.

Exit codes (`policy init/show/set`):

- `0`: success (including no-op).
- `2`: unsupported key/operator, invalid value/type, or missing workspace (workspace scope).
- `1`: unexpected failure.

### World / anchor / roaming guard (naming and semantics)

Removed names (must not exist in code):

All “root” naming MUST be removed:

- Config keys: `world.root_mode`, `world.root_path`
- CLI flags/aliases: `--world-root-mode`, `--world-root-path`
- Env vars: `SUBSTRATE_WORLD_ROOT_MODE`, `SUBSTRATE_WORLD_ROOT_PATH`
- Trace/span legacy fields that exist solely for compatibility

Canonical names are:

- Config: `world.anchor_mode`, `world.anchor_path`
- CLI: `--anchor-mode`, `--anchor-path`
- Env: `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`

Anchor resolution (runtime semantics):

Substrate MUST resolve the **anchor root** for a given execution as follows:

Inputs (highest to lowest precedence):

1. CLI: `--anchor-mode`, `--anchor-path`
2. Env: `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`
3. Effective config: `world.anchor_mode`, `world.anchor_path`

Rules:

- If `anchor_mode=workspace`:
  - If `<workspace_root>` exists: anchor root MUST be `<workspace_root>`.
  - Else: anchor root MUST be the process launch directory (`cwd` at Substrate start).
- If `anchor_mode=follow-cwd`:
  - Anchor root MUST be the current working directory at the time the anchor is consulted (dynamic).
- If `anchor_mode=custom`:
  - `anchor_path` MUST be provided and MUST be non-empty.
  - If `anchor_path` is relative, it MUST be resolved relative to the process launch directory.
  - The resolved path MUST exist and MUST be a directory; otherwise it is a hard error.

The anchor root MUST be used consistently as the “project boundary” for:

- The roaming guard (`--caged/--uncaged`)
- World filesystem isolation in `project` mode

Roaming guard (`--caged/--uncaged`):

`world.caged` and `--caged/--uncaged` are a roaming guard only:

- They MUST control whether the shell allows `cd` outside the resolved anchor root.
- They MUST NOT be described as filesystem isolation or policy enforcement.

Filesystem isolation rename (policy-only):

The filesystem isolation concept MUST use only the “isolation” naming:

- Policy key: `world_fs.isolation` (`project|full`)
- Env export (host → world): `SUBSTRATE_WORLD_FS_ISOLATION`

All filesystem-isolation “cage” naming MUST be removed:

- Policy key: `world_fs.cage`
- Env export: `SUBSTRATE_WORLD_FS_CAGE`

### Environment variables (exhaustive for this ADR)

The following env vars MUST exist and MUST have the semantics described:

- `SUBSTRATE_HOME`
- `SUBSTRATE_WORLD=enabled|disabled`
- `SUBSTRATE_ANCHOR_MODE=workspace|follow-cwd|custom`
- `SUBSTRATE_ANCHOR_PATH=<PATH>`
- `SUBSTRATE_CAGED=<bool>`
- `SUBSTRATE_POLICY_MODE=disabled|observe|enforce`
- Sync (from world-sync C1 spec):
  - `SUBSTRATE_SYNC_AUTO_SYNC=<bool>`
  - `SUBSTRATE_SYNC_DIRECTION=from_world|from_host|both`
  - `SUBSTRATE_SYNC_CONFLICT_POLICY=prefer_host|prefer_world|abort`
  - `SUBSTRATE_SYNC_EXCLUDE=<comma-separated globs>`

No other env vars are permitted by this ADR for these concerns.

### Environment scripts (global; required)

This ADR defines two generated, global shell scripts under `$SUBSTRATE_HOME/`:

- `$SUBSTRATE_HOME/env.sh`
- `$SUBSTRATE_HOME/manager_env.sh`

`$SUBSTRATE_HOME/env.sh` (stable exports; generated):

Purpose:

- Provide a single, stable source of “cached Substrate state” for shells and tooling.
- Prevent drift by ensuring world-related exports are never stored in the runtime-generated `manager_env.sh`.

Ownership and write rules:

- `$SUBSTRATE_HOME/env.sh` MUST be written only by:
  - Installers (`scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`)
  - `substrate config global init|set`
  - `substrate world enable`
- Runtime `substrate` execution MUST NOT rewrite `$SUBSTRATE_HOME/env.sh`.

File format rules:

- It MUST be a bash script.
- It MUST start with a shebang line: `#!/usr/bin/env bash`
- It MUST be safe to source repeatedly (idempotent exports only).

Exports (exhaustive for this ADR):

- `export SUBSTRATE_HOME="<resolved $SUBSTRATE_HOME>"`
- `export SUBSTRATE_WORLD=enabled|disabled` (derived from `world.enabled`)
- `export SUBSTRATE_CAGED=1|0` (derived from `world.caged`)
- `export SUBSTRATE_ANCHOR_MODE=workspace|follow-cwd|custom` (derived from `world.anchor_mode`)
- `export SUBSTRATE_ANCHOR_PATH="<string>"` (derived from `world.anchor_path`; empty string allowed)
- `export SUBSTRATE_POLICY_MODE=disabled|observe|enforce` (derived from `policy.mode`)

No other exports are permitted by this ADR in `$SUBSTRATE_HOME/env.sh`.

`$SUBSTRATE_HOME/manager_env.sh` (runtime manager wiring; generated):

Purpose:

- Runtime-only glue for sourcing manager init snippets and user bash environment hooks.
- It MUST preserve the stable exports by sourcing `$SUBSTRATE_HOME/env.sh`.

Ownership and write rules:

- Runtime `substrate` execution MUST (re)generate `$SUBSTRATE_HOME/manager_env.sh` on startup when shims are enabled.
- `substrate world deps *` MUST ensure `$SUBSTRATE_HOME/manager_env.sh` exists before invoking any guest-side manager tooling.
- Installers and `substrate world enable` MUST NOT write world-related exports into `$SUBSTRATE_HOME/manager_env.sh`.

File format rules:

- It MUST be a bash script.
- It MUST start with a shebang line: `#!/usr/bin/env bash`
- It MUST begin by sourcing `$SUBSTRATE_HOME/env.sh` if it exists:
  - If `$SUBSTRATE_HOME/env.sh` is missing, `manager_env.sh` MUST continue without failing.

Required behavior (exhaustive for this ADR):

1. Source `$SUBSTRATE_HOME/env.sh` (stable exports).
2. Source the runtime-generated manager init snippet (canonical path: `$SUBSTRATE_HOME/manager_init.sh`) if it exists.
3. Source the user’s original `BASH_ENV` if Substrate captured it (`SUBSTRATE_ORIGINAL_BASH_ENV`) and it exists.
4. Source the legacy bashenv file at `~/.substrate_bashenv` if it exists.

`SUBSTRATE_MANAGER_ENV` override (removed):

`SUBSTRATE_MANAGER_ENV` MUST NOT exist and MUST NOT be consulted for resolving the manager env script path. The manager env path is always `$SUBSTRATE_HOME/manager_env.sh`.

Bash preexec script integration (required):

The bash preexec script (currently written to `~/.substrate_preexec`) MUST source `$SUBSTRATE_HOME/manager_env.sh` and MUST NOT consult `SUBSTRATE_MANAGER_ENV`.

### Top-level CLI flags (exhaustive subset for this ADR)

The following top-level flags MUST exist:

- `--world` / `--no-world`
- `--caged` / `--uncaged`
- `--anchor-mode <workspace|follow-cwd|custom>`
- `--anchor-path <PATH>`
- `--policy-mode <disabled|observe|enforce>`

No other flags are permitted by this ADR for these concerns.

### World enable home semantics (required)

`substrate world enable` MUST have unambiguous “home” semantics:

Command:

`substrate world enable [--home <PATH>] [--profile <NAME>] [--dry-run] [--verbose] [--force] [--timeout <SECONDS>]`

Flags:

- `--home <PATH>`: sets `$SUBSTRATE_HOME` for the operation.
  - If omitted, `$SUBSTRATE_HOME` is resolved normally (see “Substrate Home”).
- `--profile <NAME>`: provisioning profile label passed to the helper script.
- `--dry-run`: show actions without executing provisioning and without writing any files.
- `--verbose`: stream helper output to stdout/stderr in addition to the log.
- `--force`: rerun provisioning even if metadata indicates it is already enabled.
- `--timeout <SECONDS>`: maximum seconds to wait for world socket/doctor checks.

`--prefix` MUST NOT exist. `SUBSTRATE_PREFIX` MUST NOT exist.

Filesystem effects (when not `--dry-run`):

`substrate world enable` MUST treat `--home` as the only state root:

- Logs MUST be written under: `<home>/logs/`
- Config MUST be read/written at: `<home>/config.yaml`
- Environment scripts MUST be read/written at:
  - `<home>/env.sh`
  - `<home>/manager_env.sh` (runtime-owned; world enable MUST NOT modify this file)

`substrate world enable` MUST NOT read or write `<home>/policy.yaml`.

Metadata update guarantee:

On successful provisioning, `substrate world enable` MUST:

- Set `world.enabled=true` in `<home>/config.yaml` (creating the file if missing).
- Regenerate `<home>/env.sh` to reflect the new effective global defaults.

Helper invocation guarantee:

The world enable helper script MUST be invoked with consistent home semantics:

- The helper script MUST be passed `--home <PATH>` (not `--prefix`).
- The helper process environment MUST include `SUBSTRATE_HOME=<PATH>` set to the same value.

### World-sync implications (required)

World-sync commands MUST gate on workspace initialization:

- `substrate sync`
- `substrate checkpoint`
- `substrate rollback`

Gating rule:

- If no `<workspace_root>` exists for `cwd`, these commands MUST exit `2` and MUST direct the user to `substrate workspace init`.

## Architecture Shape

### Components affected (non-exhaustive but required map)

Primary hot spots (current code reality that must be rewritten to satisfy this ADR):

- Global-only config commands:
  - `crates/shell/src/execution/config_cmd.rs`
- Launch-dir-only directory settings:
  - `crates/shell/src/execution/settings/builder.rs`
  - `crates/shell/src/execution/settings/runtime.rs`
- Legacy enforcement profile discovery:
  - `crates/broker/src/profile.rs`
  - `crates/broker/src/broker.rs` (holds `ProfileDetector`)
- Approvals policy writer (must be workspace/global only):
  - `crates/broker/src/approval.rs`
- Policy schema:
  - `crates/broker/src/policy.rs` (`world_fs.cage` -> `world_fs.isolation`)
- World env plumbing and parsing:
  - `crates/world-agent/src/service.rs`
  - `crates/world/src/exec.rs`
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
- CLI flags:
  - `crates/shell/src/execution/cli.rs` (remove `world-root-*` alias; add `policy-mode`; add `workspace` + `policy` subcommands)

### End-to-end flow (inputs → derived state → actions → outputs)

Inputs:

- CLI flags (world selection, anchor selection, policy mode, config/policy subcommands)
- Environment variables (subset defined by this ADR)
- Global files under `$SUBSTRATE_HOME/`
- Workspace files under `<workspace_root>/.substrate/` when in a workspace

Derived state:

- Resolved `$SUBSTRATE_HOME`
- Resolved `<workspace_root>` (or “no workspace”)
- Effective config (layered)
- Effective policy (selected)
- Policy-derived “requires world” constraints (observe/enforce only)
- Anchor root (resolved)

Actions:

- Evaluate policy and enforce/observe/skip per `policy.mode`
- Select host vs world execution per the rules in this ADR
- Enforce roaming guard per `world.caged`
- Generate/maintain `$SUBSTRATE_HOME/env.sh` and `$SUBSTRATE_HOME/manager_env.sh` per ownership rules

Outputs:

- Trace/telemetry records (including policy decisions in observe/enforce)
- Updated config/policy files when requested by CLI or approval “save” action
- World enable logs under `<home>/logs/`

## Sequencing / Dependencies

- Sequencing entry: `docs/project_management/next/sequencing.json` → sprint id `policy_and_config_mental_model_simplification` (order `25`).
- This sprint MUST land before `world_sync` to avoid implementing world-sync against legacy config/policy/env semantics.
- Before implementing `world_sync` code, world-sync planning docs MUST be updated to match this ADR (at minimum `docs/project_management/next/world-sync/C0-spec.md` and `docs/project_management/next/world-sync/C1-spec.md`).

## Security / Safety Posture

- Strict parsing is fail-closed:
  - Unknown keys and type mismatches are hard errors for config/policy.
- Policy-derived “requires world” behavior is fail-closed in `enforce` mode:
  - If world is required and unavailable, Substrate fails closed and does not silently fall back.
- Protected paths:
  - `sync.exclude` always includes `.git/**`, `.substrate/**`, `.substrate-git/**` and they cannot be removed.
- Policy invariants are fail-closed:
  - `world_fs.isolation=full` and `world_fs.mode=read_only` require `world_fs.require_world=true`.
- Drift prevention:
  - Stable exports MUST be in `$SUBSTRATE_HOME/env.sh` only; runtime must not clobber that file.

## Validation Plan (Authoritative)

### Tests (required)

Implementation MUST include automated tests that lock in all precedence rules and all “removed” behaviors from this ADR.

Test suite locations (required):

- `crates/shell/tests/` (workspace discovery, config precedence, world-enable home semantics, env scripts behavior)
- `crates/broker/src/tests.rs` or `crates/broker/tests/` (policy discovery/updates, enforce/observe/disabled semantics)

Workspace discovery guardrails (required):

Tests MUST verify:

1. **Walk-up discovery**:
   - Given nested directories, `<workspace_root>` resolves to the nearest ancestor containing `.substrate/workspace.yaml`.
2. **No-workspace behavior**:
   - If no `.substrate/workspace.yaml` exists in any ancestor, Substrate treats `cwd` as “not in a workspace”.
3. **Nested workspace init refusal**:
   - `substrate workspace init` exits `2` and makes no writes when a parent workspace already exists.

Config precedence guardrails (required):

Tests MUST verify the precedence order exactly as specified:

- CLI > env > workspace > global > defaults

For each key below, tests MUST cover at least one scenario where each layer overrides the next layer:

- `world.enabled`
- `world.anchor_mode`
- `world.anchor_path`
- `world.caged`
- `policy.mode`
- `sync.auto_sync`
- `sync.direction`
- `sync.conflict_policy`
- `sync.exclude` (including the `+=` and `-=` operators)

Protected excludes tests MUST verify:

- The protected patterns `.git/**`, `.substrate/**`, `.substrate-git/**` are always effectively present.
- Attempts to remove protected patterns via env or CLI do not remove them.

Policy discovery + write target guardrails (required):

Tests MUST verify effective policy selection order exactly:

1. `<workspace_root>/.substrate/policy.yaml` if it exists
2. `$SUBSTRATE_HOME/policy.yaml` if it exists
3. built-in default policy otherwise

Approvals “save to policy” tests MUST verify write target selection exactly:

- If `<workspace_root>` exists: writes to `<workspace_root>/.substrate/policy.yaml` (creating it if missing).
- Else: writes to `$SUBSTRATE_HOME/policy.yaml` (creating it if missing).

All tests MUST verify `$SUBSTRATE_HOME` is honored (i.e., no implicit `dirs::home_dir()` fallback).

Policy mode semantics guardrails (required):

Tests MUST verify:

- `disabled`: policy is not evaluated (no allow/deny decisions computed).
- `observe`: policy is evaluated, but execution is not blocked.
- `enforce`: policy is evaluated and blocks/approves according to policy.

Tests MUST include at least one denied-command case and assert:

- In `observe`, the command is allowed and a “would deny” result is produced/recorded.
- In `enforce`, the command is denied.

Environment scripts guardrails (required):

Tests MUST verify:

1. `$SUBSTRATE_HOME/env.sh` is not rewritten by runtime `substrate` execution.
2. `$SUBSTRATE_HOME/manager_env.sh` is rewritten by runtime when shims are enabled and MUST source `$SUBSTRATE_HOME/env.sh` when it exists.
3. `substrate world deps *` ensures `$SUBSTRATE_HOME/manager_env.sh` exists before invoking guest-side managers.
4. `substrate world enable`:
   - regenerates `<home>/env.sh` (see “Metadata update guarantee”),
   - does not modify `<home>/manager_env.sh`,
   - and uses consistent `--home` semantics (see “Helper invocation guarantee”).

Legacy removal guardrails (required; no compatibility):

Because this ADR forbids all compatibility layers, tests MUST verify that legacy artifacts are rejected and/or ignored in a way that prevents silent partial behavior.

At minimum, tests MUST verify:

- `--prefix` is not accepted by `substrate world enable`.
- `SUBSTRATE_PREFIX` has no effect on any behavior.
- `SUBSTRATE_MANAGER_ENV` has no effect on any behavior.
- Config keys `install.world_enabled`, `world.root_mode`, `world.root_path` are rejected by `substrate config * set` and by config parsing.
- Policy key `world_fs.cage` is rejected by policy parsing.
- Presence of `.substrate/settings.yaml` MUST cause a hard error with an actionable message directing the user to use `.substrate/workspace.yaml`.

### Manual validation (explicit)

Manual playbook: `docs/project_management/next/policy_and_config_mental_model_simplification/manual_testing_playbook.md`

### Smoke scripts (explicit)

- Linux: `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility

- Policy: greenfield breaking is allowed
- Compat work: none (explicitly forbidden by this ADR)

## Decision Summary

Decision Register: `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`

Entries used by this ADR:

- DR-0001, DR-0002, DR-0003, DR-0004, DR-0005, DR-0006, DR-0007, DR-0008, DR-0009, DR-0010

## Expected Breaking Changes (Explicit)

These are required outcomes of implementing this ADR:

- Workspace settings file rename:
  - Remove `.substrate/settings.yaml`
  - Add `.substrate/workspace.yaml` (walk-up discovered)
- Enforcement profile removal:
  - Remove `.substrate-profile` and `.substrate-profile.d/*`
  - Enforcement policy is `.substrate/policy.yaml` (workspace) with `$SUBSTRATE_HOME/policy.yaml` fallback
- Policy file name cleanup:
  - Remove `.substrate-policy.yaml`
- Config key cleanup:
  - Remove `install.world_enabled`
  - Add `world.enabled`
- “root” removal:
  - Remove `world.root_mode`, `world.root_path`
  - Remove `--world-root-mode`, `--world-root-path`
  - Remove `SUBSTRATE_WORLD_ROOT_MODE`, `SUBSTRATE_WORLD_ROOT_PATH`
- Filesystem isolation rename:
  - Remove `world_fs.cage` and `SUBSTRATE_WORLD_FS_CAGE`
  - Add `world_fs.isolation` and `SUBSTRATE_WORLD_FS_ISOLATION`
- CLI restructuring:
  - `substrate config` is workspace-scoped (effective view + writes to `.substrate/workspace.yaml`)
  - `substrate config global` is global-scoped (reads/writes `$SUBSTRATE_HOME/config.yaml`)
  - `substrate config init` MUST NOT exist
  - `substrate workspace init` is the only workspace initializer
  - Add `substrate policy` and `substrate policy global` as specified
- Environment file split:
  - `$SUBSTRATE_HOME/env.sh` is the canonical stable export file
  - `$SUBSTRATE_HOME/manager_env.sh` is runtime-generated and MUST source `env.sh`
- World enable home semantics:
  - `substrate world enable --prefix` MUST NOT exist
  - `substrate world enable --home` MUST exist and MUST update state under the chosen `$SUBSTRATE_HOME`
