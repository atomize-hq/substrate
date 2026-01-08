# ADR-0008 — Workspace Config/Policy Scopes + `.substrate/` Unification (Patch Files)

## Status
- Status: Draft
- Date (UTC): 2026-01-08
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/next/workspace-config-policy-unification/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Prior ADRs:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`
- Decision Register: `docs/project_management/next/workspace-config-policy-unification/decision_register.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 5e8c1102f4ba213b1d0dffa4f300ed0d9f7f44fd30b61b5cdb12f8b2599c94c8
ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md` after drafting>

### Changes (operator-facing)
- Config and policy commands become explicit about scope and effective views
  - Existing: `config show` vs `config global show` can disagree because `config show` is an effective/merged view and can be overridden by persistent `SUBSTRATE_OVERRIDE_*` env exports from install/dev scripts.
  - New: `current show` is the effective/merged view; `global|workspace show` shows exactly what is set at that scope (patch file). Install/dev scripts do not export `SUBSTRATE_OVERRIDE_*` by default.
  - Why: Eliminate “why didn’t my global set take effect?” confusion and make config and policy semantics symmetric.
  - Links:
    - `docs/project_management/next/workspace-config-policy-unification/decision_register.md`
    - `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`
    - `crates/shell/src/execution/config_model.rs#L220`

- Workspace state is unified under a single canonical `.substrate/` directory
  - Existing: Workspace state is split across `.substrate/` and `.substrate-git/`.
  - New: All workspace-scoped state lives under `<workspace_root>/.substrate/`, including internal git at `<workspace_root>/.substrate/git/repo.git/`.
  - Why: Reduce protected-path surface area and simplify onboarding and sync exclusions.
  - Links:
    - `crates/shell/src/execution/workspace.rs#L3`
    - `crates/shell/src/execution/workspace.rs#L26`
    - `crates/shell/src/execution/config_model.rs#L14`

## Problem / Context
- Users expect config and policy to behave symmetrically:
  - global scope is the baseline,
  - workspace scope overrides global when a workspace exists,
  - “effective/current” views explain what will actually happen for the current `cwd`.
- Recent work introduced `SUBSTRATE_OVERRIDE_*` for one-off operator overrides (`ADR-0006`), but dev/installer scripts exporting overrides by default causes:
  - `config global set` appearing to “do nothing” in no-workspace directories,
  - policy behaving differently than config (policy has no env override layer),
  - persistent confusion about where truth lives.
- Workspace state is currently split across multiple directories (`.substrate/` and `.substrate-git/`), which:
  - expands protected-path handling requirements for sync and tooling,
  - makes onboarding harder (“which directory is the canonical workspace state?”).

## Goals
- Provide a symmetric CLI for **config** and **policy** with explicit scopes:
  - `current` (effective/merged),
  - `global` (global patch),
  - `workspace` (workspace patch).
- Make scope files **patch files** (sparse YAML mappings) so “reset/unset” can mean “inherit” without copying values.
- Unify all workspace state under `<workspace_root>/.substrate/`:
  - workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
  - workspace policy patch: `<workspace_root>/.substrate/policy.yaml`
  - workspace disable marker: `<workspace_root>/.substrate/workspace.disabled`
  - internal git: `<workspace_root>/.substrate/git/repo.git/`
- Ensure install/dev env scripts do not export `SUBSTRATE_OVERRIDE_*` by default; overrides remain supported only as explicit one-off operator inputs.

## Non-Goals
- Backwards compatibility, migrations, or warnings for legacy directory layouts or marker names (greenfield; see DR-0008).
- Any support for alternative workspace state directories beyond `<workspace_root>/.substrate/`.
- Any feature that modifies user repo `.git` internals (workspace operations do not write to `.git/`).
- Changing the world backend isolation model, broker enforcement model, or policy semantics beyond file discovery and patch/merge behavior defined here.

## User Contract (Authoritative)

### Terminology
- **Workspace root**: the nearest ancestor directory containing `<dir>/.substrate/workspace.yaml` such that `<dir>/.substrate/workspace.disabled` does not exist.
- **Workspace disabled**: a workspace root that has `<workspace_root>/.substrate/workspace.disabled` present; it is treated as non-existent for discovery and effective resolution.
- **Patch file**: a YAML mapping that may omit any keys; omitted keys mean “inherit from the next lower precedence layer”.

### CLI

#### `substrate config current show [--json] [--explain]`
- Prints the **effective config** for the current `cwd` (YAML by default, JSON with `--json`).
- Stdout:
  - YAML or JSON payload of the effective config.
- Stderr (always):
  - A single line notice:
    - `substrate: note: showing effective merged config; use --explain to view per-key sources`
- `--explain`:
  - Emits an additional machine-readable provenance map to **stderr** that indicates the source of every effective key:
    - `cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`, `injected_protected`.
- Exit codes:
  - Taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `2`: invalid config file YAML, invalid override value, or other actionable user error
  - `1`: unexpected failure

#### `substrate config global show [--json]`
- Prints the **global config patch** at `$SUBSTRATE_HOME/config.yaml`.
- If the file does not exist, it prints an empty mapping:
  - YAML: `{}` (plus newline)
  - JSON: `{}` (pretty-printed)
- It MUST NOT incorporate workspace patches, override env vars, protected-exclude injection, or CLI flag overrides.
- Exit codes: `0` on success; `2` on invalid YAML; `1` on unexpected failure.

#### `substrate config global set [--json] UPDATE...`
- Applies updates to the global config patch at `$SUBSTRATE_HOME/config.yaml`, creating the file if missing.
- `UPDATE` syntax:
  - `key=value`
  - `key+=value` (list append)
  - `key-=value` (list remove; exact match)
- Allowed update keys are exactly the config keys described under **Config schema** below.
- On success:
  - Prints the **effective config for `cwd`** after applying the update (same output contract as `config current show` but without the merged notice line).
  - Writes `$SUBSTRATE_HOME/env.sh` exported state based on the new effective config.
- Exit codes: `0` success; `2` invalid update/value/YAML; `1` unexpected.

#### `substrate config global reset [KEY ...]`
- If no `KEY` arguments are provided:
  - Resets the global config patch to an empty mapping (`{}`), meaning “no global overrides; defaults apply”.
- If one or more `KEY` arguments are provided:
  - Removes those keys from the global patch (so they inherit from defaults).
- It MUST NOT modify any workspace patch.
- Exit codes: `0` success (including no-op); `2` invalid key; `1` unexpected.

#### `substrate config workspace show [--json]`
- Requires a workspace root for current `cwd`.
- Prints the **workspace config patch** at `<workspace_root>/.substrate/workspace.yaml`.
- If the file does not exist, it prints `{}` (the workspace has no overrides).
- It MUST NOT incorporate global patch, override env vars, protected-exclude injection, or CLI flag overrides.
- Exit codes:
  - `0`: success
  - `2`: no workspace root found
  - `1`: unexpected failure

#### `substrate config workspace set [--json] UPDATE...`
- Requires a workspace root for current `cwd`.
- Applies updates to `<workspace_root>/.substrate/workspace.yaml`, creating the file if missing.
- `UPDATE` syntax matches `config global set`.
- On success:
  - Prints the **effective config for `cwd`** after applying the update (same output contract as `config current show` but without the merged notice line).
  - Writes `$SUBSTRATE_HOME/env.sh` exported state based on the new effective config.
- Exit codes: `0` success; `2` actionable user error; `1` unexpected.

#### `substrate config workspace reset [KEY ...]`
- Requires a workspace root for current `cwd`.
- If no `KEY` arguments are provided:
  - Resets the workspace config patch to `{}` (meaning “inherit all config from global/default layers”).
- If one or more `KEY` arguments are provided:
  - Removes those keys from the workspace patch (so they inherit from global/default).
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected.

---

#### `substrate policy current show [--json] [--explain]`
- Prints the **effective policy** for the current `cwd`:
  - If a workspace root exists and is enabled and `<workspace_root>/.substrate/policy.yaml` exists: apply workspace patch over the global policy patch over defaults.
  - Otherwise: apply global policy patch over defaults.
- Stderr (always):
  - `substrate: note: showing effective merged policy; use --explain to view per-key sources`
- `--explain`:
  - Emits a per-key provenance breakdown to stderr (`workspace_patch`, `global_patch`, `default`).
- Exit codes: `0` success; `2` invalid YAML / invalid policy; `1` unexpected.

#### `substrate policy global show|set|reset` and `substrate policy workspace show|set|reset`
- These commands mirror the config commands at the corresponding scope, with the policy schema described below.
- `global show` prints `$SUBSTRATE_HOME/policy.yaml` as a patch; missing file prints `{}`.
- `workspace show` prints `<workspace_root>/.substrate/policy.yaml` as a patch; missing file prints `{}`.
- `set` applies dotted updates to the patch file and prints the effective policy after update.
- `reset` removes keys from the patch file (or clears it to `{}`) so the effective policy inherits from lower layers.

---

#### `substrate workspace init [PATH] [--force] [--examples]`
- Initializes a workspace at `PATH` (default `.`).
- It MUST ensure these paths exist (create if missing):
  - `<workspace_root>/.substrate/`
  - `<workspace_root>/.substrate/workspace.yaml` (patch file; must be valid YAML mapping; default `{}` plus comment header)
  - `<workspace_root>/.substrate/policy.yaml` (patch file; must be valid YAML mapping; default `{}` plus comment header)
  - `<workspace_root>/.substrate/git/repo.git/` (internal git directory)
- It MUST ensure the workspace `.gitignore` at `<workspace_root>/.gitignore` contains these exact ignore rules (order does not matter; duplicates allowed):
  - `.substrate/`
  - `!.substrate/workspace.yaml`
  - `!.substrate/policy.yaml`
- It MUST refuse nested workspace creation:
  - If any parent directory of `PATH` contains `.substrate/workspace.yaml`, `workspace init` exits `2` and performs no writes outside `PATH/.substrate/` and `PATH/.gitignore`.
- `--force`:
  - Repairs missing entries only; does not overwrite existing non-empty patch files.
- `--examples`:
  - Creates these non-active template files:
    - `<workspace_root>/.substrate/workspace.example.yaml`
    - `<workspace_root>/.substrate/policy.example.yaml`
  - Substrate MUST NOT read these example files for any behavior.

#### `substrate workspace disable [PATH]`
- Requires an enabled workspace root resolved from `PATH` (default `.`).
- Creates `<workspace_root>/.substrate/workspace.disabled` (idempotent).
- Once disabled, workspace discovery treats this workspace as non-existent for config/policy/broker evaluation.

#### `substrate workspace enable [PATH]`
- Requires a workspace root resolved from `PATH` (default `.`).
- Removes `<workspace_root>/.substrate/workspace.disabled` if present (idempotent).

#### `substrate workspace reset [PATH]`
- Requires a workspace root resolved from `PATH` (default `.`).
- Resets both workspace patch files to `{}` while preserving the comment headers:
  - `<workspace_root>/.substrate/workspace.yaml`
  - `<workspace_root>/.substrate/policy.yaml`
- It MUST NOT delete or modify `<workspace_root>/.substrate/git/repo.git/`.
- It MUST NOT change workspace enabled/disabled state.

#### `substrate workspace remove [PATH]`
- Requires a workspace root resolved from `PATH` (default `.`).
- Deletes the entire workspace state directory:
  - `<workspace_root>/.substrate/` (including config, policy, disable marker, and internal git).
- It MUST NOT modify `<workspace_root>/.gitignore`.

### Config

#### Files and locations (precedence for effective config)
1. CLI flags (subset of keys with CLI flags)
2. `SUBSTRATE_OVERRIDE_*` override inputs (one-off operator input; never exported by install/dev scripts)
3. Workspace config patch: `<workspace_root>/.substrate/workspace.yaml` (when a workspace exists and is enabled)
4. Global config patch: `$SUBSTRATE_HOME/config.yaml`
5. Built-in defaults
6. Protected exclude injection (always applied):
   - `.git/**`
   - `.substrate/**`

#### Schema (config patch keys)
The config patch is a YAML mapping where keys may be omitted to inherit. Unknown keys are a hard error.

Allowed keys:
- `world.enabled` (bool)
- `world.anchor_mode` (`workspace`, `follow-cwd`, `custom`)
- `world.anchor_path` (string; required when `world.anchor_mode=custom` in the effective config)
- `world.caged` (bool)
- `policy.mode` (`disabled`, `observe`, `enforce`)
- `sync.auto_sync` (bool)
- `sync.direction` (`from_world`, `from_host`, `both`)
- `sync.conflict_policy` (`prefer_host`, `prefer_world`, `abort`)
- `sync.exclude` (list[string]; `+=` and `-=` are allowed; `=` requires a YAML list literal)

#### Environment variables
- `SUBSTRATE_OVERRIDE_*` variables are accepted as override inputs for config resolution.
- `SUBSTRATE_*` exported state variables MUST NOT be read as override inputs.
- Install/dev env scripts MUST NOT export `SUBSTRATE_OVERRIDE_*` by default.

### Policy

#### Files and locations (precedence for effective policy)
1. Workspace policy patch: `<workspace_root>/.substrate/policy.yaml` (when a workspace exists and is enabled)
2. Global policy patch: `$SUBSTRATE_HOME/policy.yaml`
3. Built-in defaults

#### Schema (policy patch keys)
The policy patch is a YAML mapping where keys may be omitted to inherit. Unknown keys are a hard error.

Allowed keys correspond to the broker policy schema, including:
- `id` (string)
- `name` (string)
- `world_fs.mode` (`writable`, `read_only`)
- `world_fs.isolation` (`workspace`, `full`)
- `world_fs.require_world` (bool)
- `world_fs.read_allowlist` (list[string])
- `world_fs.write_allowlist` (list[string])
- `net_allowed` (list[string])
- `cmd_allowed` (list[string])
- `cmd_denied` (list[string])
- `cmd_isolated` (list[string])
- `require_approval` (bool)
- `allow_shell_operators` (bool)
- `limits.*` (resource limits fields)
- `metadata` (mapping string→string)

Policy validation invariants are enforced on the effective merged policy:
- `world_fs.mode=read_only` requires `world_fs.require_world=true`
- `world_fs.isolation=full` requires `world_fs.require_world=true`

### Platform guarantees
- Linux/macOS/Windows:
  - Identical file discovery rules and precedence semantics.
  - Identical patch merge semantics and CLI output contracts.

## Architecture Shape

### Components
- `crates/shell/src/execution/config_model.rs`:
  - Add patch parsing and patch merge for config.
  - Implement provenance emission for `current show --explain`.
  - Remove `.substrate-git/**` from protected excludes; protected excludes become exactly `.git/**` and `.substrate/**`.
- `crates/shell/src/execution/policy_model.rs`:
  - Add patch parsing and patch merge for policy.
  - Keep validation invariants on the effective merged policy.
- `crates/shell/src/execution/config_cmd.rs` and `crates/shell/src/execution/policy_cmd.rs`:
  - Implement explicit `current|global|workspace` CLI surfaces for show/set/reset.
  - Ensure `current show` emits the merged notice line and `--explain` provenance to stderr.
- `crates/shell/src/execution/workspace_cmd.rs` and `crates/shell/src/execution/workspace.rs`:
  - Unify workspace directory layout under `.substrate/`.
  - Move internal git to `.substrate/git/repo.git/`.
  - Add disable marker behavior.
- `crates/broker/src/profile.rs`:
  - Ensure policy discovery reads from `<workspace_root>/.substrate/policy.yaml` and `$SUBSTRATE_HOME/policy.yaml` only.
  - Remove any `.substrate-profile*` behavior if present.
- Install/dev scripts:
  - `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` must not export `SUBSTRATE_OVERRIDE_*` by default.
- Docs:
  - Update `docs/CONFIGURATION.md` to reflect `current/global/workspace` semantics and patch-file behavior.
  - Update `docs/reference/paths/layout.md` and any `world-sync` specs that reference `.substrate-git`.

### End-to-end flow (effective config)
- Inputs:
  - CLI flags
  - `SUBSTRATE_OVERRIDE_*` env inputs (if set explicitly by the operator)
  - workspace config patch (if workspace enabled)
  - global config patch
  - defaults
- Derived state:
  - workspace root discovery with disable marker handling
  - merged config
  - injected protected excludes
- Actions:
  - compute world root settings (including `world.caged`)
  - execute commands, enforce caged root guard
- Outputs:
  - YAML/JSON outputs from CLI
  - exported state env vars via `$SUBSTRATE_HOME/env.sh` (never used as inputs)

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `workspace_config_policy_unification`
- Prerequisites:
  - None; this ADR is a cross-cutting contract consolidation that supersedes prior behavioral ambiguity between effective vs scope views.

## Security / Safety Posture
- Fail-closed rules:
  - Invalid YAML, unknown keys, or type mismatches in any patch file are actionable user errors (exit `2`) and produce no file mutations.
  - Policy invariant violations are actionable user errors (exit `2`) and produce no file mutations.
- Protected paths/invariants:
  - Workspace commands write only within `<workspace_root>/.substrate/` and `<workspace_root>/.gitignore` (for init only).
  - Workspace remove deletes only `<workspace_root>/.substrate/`.
  - No command in this ADR writes to the user repo’s `.git/`.
- Observability:
  - `current show --explain` provides deterministic provenance for debugging precedence issues.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Patch merge logic for config and policy (including list set/append/remove semantics).
  - Workspace disable marker discovery behavior.
- Integration tests:
  - CLI golden tests for:
    - `config current show` vs `config global show` vs `config workspace show`
    - `policy current show` vs `policy global show` vs `policy workspace show`
  - Tests ensuring `SUBSTRATE_OVERRIDE_*` affects `current show` only when explicitly set and is not required for correctness.
  - Tests ensuring protected excludes include `.git/**` and `.substrate/**` and no other injected excludes.

### Manual validation
- Manual playbook: `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed.
- Compat work: none.
- Legacy directory layouts and marker names are out of scope and are ignored without warnings or migrations.

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/workspace-config-policy-unification/decision_register.md`:
    - DR-0001, DR-0002, DR-0003, DR-0004, DR-0005, DR-0006, DR-0007, DR-0008
