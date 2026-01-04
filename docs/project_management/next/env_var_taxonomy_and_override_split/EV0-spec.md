# EV0 Spec — Override Split for Effective Config (ADR-0006)

## Scope
This slice changes the effective-config resolver to use a dedicated override-input namespace:
- Override inputs: `SUBSTRATE_OVERRIDE_*`
- Exported state: `SUBSTRATE_*` (output-only for config resolution)

This slice applies wherever `crates/shell/src/execution/config_model.rs` resolves effective config.

## Non-Scope
- Backwards compatibility for legacy config-shaped `SUBSTRATE_*` override inputs.
- Changes to policy discovery or policy schema.
- Changes to world backend transports or shim tracing.

## User Contract (Authoritative)
Authoritative contract: `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

### Effective config precedence (workspace present)
When `<workspace_root>` exists for `cwd`, the effective config precedence (highest to lowest) is:
1. CLI flags (subset of keys with CLI flags)
2. `<workspace_root>/.substrate/workspace.yaml`
3. Environment variables `SUBSTRATE_OVERRIDE_*` (subset)
4. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
5. Built-in defaults

Observable requirement:
- When `<workspace_root>/.substrate/workspace.yaml` exists, `SUBSTRATE_OVERRIDE_*` values do not change effective config values.

### Effective config precedence (no workspace)
When no workspace exists for `cwd`, the effective config precedence (highest to lowest) is:
1. CLI flags (subset of keys with CLI flags)
2. Environment variables `SUBSTRATE_OVERRIDE_*` (subset)
3. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
4. Built-in defaults

Observable requirement:
- When no workspace exists, `SUBSTRATE_OVERRIDE_*` values override `$SUBSTRATE_HOME/config.yaml` for the supported subset of keys.

### Override input mapping (supported subset)
The effective-config resolver MUST consult only the following override-input env vars:
- `SUBSTRATE_OVERRIDE_WORLD`: `"enabled" | "disabled"`
- `SUBSTRATE_OVERRIDE_ANCHOR_MODE`: `workspace | follow-cwd | custom` (parsed via `WorldRootMode::parse`)
- `SUBSTRATE_OVERRIDE_ANCHOR_PATH`: string (may be empty)
- `SUBSTRATE_OVERRIDE_CAGED`: boolean (`true|false|1|0|yes|no|on|off`)
- `SUBSTRATE_OVERRIDE_POLICY_MODE`: `disabled | observe | enforce`
- `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC`: boolean
- `SUBSTRATE_OVERRIDE_SYNC_DIRECTION`: `from_world | from_host | both`
- `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY`: `prefer_host | prefer_world | abort`
- `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE`: comma-separated string list

All other env vars are ignored by the effective-config resolver.

### Exported state is output-only for config resolution
The effective-config resolver MUST NOT consult the following config-shaped exported-state env vars as override inputs:
- `SUBSTRATE_WORLD`
- `SUBSTRATE_ANCHOR_MODE`
- `SUBSTRATE_ANCHOR_PATH`
- `SUBSTRATE_CAGED`
- `SUBSTRATE_POLICY_MODE`
- `SUBSTRATE_SYNC_AUTO_SYNC`
- `SUBSTRATE_SYNC_DIRECTION`
- `SUBSTRATE_SYNC_CONFLICT_POLICY`
- `SUBSTRATE_SYNC_EXCLUDE`

### Strict parsing
- If a supported `SUBSTRATE_OVERRIDE_*` variable is present with a non-empty value and parsing fails, the resolver MUST return a user/config error.
- Error messages MUST name the specific `SUBSTRATE_OVERRIDE_*` variable and list the allowed values/shape.

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `1`: unexpected failure
- `2`: user/config error (invalid override values when parsed by a command that maps user errors to `2`)
- `3`: required dependency unavailable (playbook/smoke only)

## Acceptance Criteria (Authoritative)
- `crates/shell/src/execution/config_model.rs` reads only `SUBSTRATE_OVERRIDE_*` for config-shaped override inputs.
- `crates/shell/src/execution/config_model.rs` does not read config-shaped `SUBSTRATE_*` exported-state variables as override inputs.
- Errors for invalid override values mention the `SUBSTRATE_OVERRIDE_*` variable name and allowed values.
- Feature-local smoke scripts validate the override split on all required platforms:
  - Linux: `docs/project_management/next/env_var_taxonomy_and_override_split/smoke/linux-smoke.sh`
  - macOS: `docs/project_management/next/env_var_taxonomy_and_override_split/smoke/macos-smoke.sh`
  - Windows: `docs/project_management/next/env_var_taxonomy_and_override_split/smoke/windows-smoke.ps1`

## Validation Requirements

### Tests (required)
- Add/update tests to lock the override split behavior:
  - A `SUBSTRATE_POLICY_MODE=<value>` exported-state value must not override config policy mode resolution.
  - A `SUBSTRATE_OVERRIDE_POLICY_MODE=<value>` override input must override config policy mode resolution when no workspace exists.
  - A workspace config value must override any `SUBSTRATE_OVERRIDE_*` values.

### Manual testing (required)
- Follow `docs/project_management/next/env_var_taxonomy_and_override_split/manual_testing_playbook.md`.

