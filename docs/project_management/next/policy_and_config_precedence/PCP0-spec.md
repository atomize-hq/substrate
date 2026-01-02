# PCP0 Spec — Workspace Config Precedence Over Env (ADR-0005)

## Scope
This slice changes effective config precedence when a workspace exists so that workspace config overrides `SUBSTRATE_*` environment variables.

This slice applies to all commands that consume effective config (not only `substrate config show`), because they share the same effective-config resolver.

## Non-Scope
- Workspace discovery semantics and marker locations.
- Config schema changes.
- Policy precedence and policy file selection.
- Env script ownership rules (`$SUBSTRATE_HOME/env.sh`, `$SUBSTRATE_HOME/manager_env.sh`).
- Trace schema changes.

## User Contract (Authoritative)
Authoritative contract: `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`

This spec is the implementation slice and defines acceptance criteria and validation expectations for PCP0.

### Effective config precedence (workspace present)
When `<workspace_root>` exists for `cwd`, the effective config precedence (highest to lowest) is:
1. CLI flags (subset of keys with CLI flags)
2. `<workspace_root>/.substrate/workspace.yaml`
3. Environment variables `SUBSTRATE_*`
4. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
5. Built-in defaults

Observable requirements:
- When `<workspace_root>` exists and a key is set in `<workspace_root>/.substrate/workspace.yaml`, an environment variable for the same key MUST NOT change the effective value.
- CLI flags MUST override `<workspace_root>/.substrate/workspace.yaml` for the subset of keys that have CLI flags.

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `1`: unexpected failure
- `2`: user/config error (including missing workspace for workspace-scoped config commands)
- `3`: required dependency unavailable (test/playbook/smoke only)

## Acceptance Criteria (Authoritative)
- `crates/shell/src/execution/config_model.rs` resolves effective config such that, when `<workspace_root>` exists, workspace config overrides any `SUBSTRATE_*` env var values for all config keys.
- `crates/shell/tests/config_show.rs` tests are updated to lock the new precedence:
  - In a workspace context, env values do not override workspace values for any config key.
  - In a workspace context, CLI flags override workspace values for the CLI-covered subset of keys.
- Feature-local smoke scripts validate the new precedence behavior:
  - Linux: `docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh`
  - macOS: `docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh`
  - Windows: `docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1`

## Validation Requirements
### Tests (required)
- Update `crates/shell/tests/config_show.rs`:
  - `config_show_resolves_effective_config_with_precedence` MUST assert that `policy.mode` and all `sync.*` keys come from workspace config (not env) when `<workspace_root>` exists.
  - The same test MUST continue to assert that CLI overrides win for `world.*` keys that have CLI flags.

### Manual testing (required)
- Follow `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`.

