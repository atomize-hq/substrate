# ADR-0005 — Workspace Config Precedence Over Env Exports

## Status
- Status: Accepted
- Date (UTC): 2026-01-02
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/next/policy_and_config_precedence/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Intended branch: `feat/policy_and_config_precedence`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

## Related Docs
- Prior ADR (baseline semantics): `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
- Plan: `docs/project_management/next/policy_and_config_precedence/plan.md`
- Tasks: `docs/project_management/next/policy_and_config_precedence/tasks.json`
- Specs:
  - `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Decision Register:
  - `docs/project_management/next/policy_and_config_precedence/decision_register.md`
  - Baseline env scripts split: `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md` (DR-0005)
- Integration Map: `docs/project_management/next/policy_and_config_precedence/integration_map.md`
- Manual Playbook: `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: c67df2c96a14a7c15b622e29729fb3c9cb8a0936792552e4b0c8ef8cc23119a6

### Changes (operator-facing)
- Workspace config always wins over sourced exports
  - Existing: If your shell environment contains `SUBSTRATE_*` values (often because `$SUBSTRATE_HOME/env.sh` is sourced), those env vars override `.substrate/workspace.yaml`, so workspace settings like `world.caged: false` can appear “ignored”.
  - New: When a workspace exists, `.substrate/workspace.yaml` takes precedence over `SUBSTRATE_*` env exports for all config keys; env vars still apply when not in a workspace (and CLI flags remain highest precedence).
  - Why: Prevent “global config → env.sh → env overrides workspace config” confusion and eliminate the effective-precedence footgun caused by stable export scripts.
  - Links:
    - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md#L255` (effective config precedence in ADR-0003)
    - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md#L605` (env scripts and `env.sh` purpose)
    - `crates/shell/src/execution/config_model.rs#L220` (current effective-config merge order)
    - `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md` (implementation slice; authoritative acceptance criteria)

## Problem / Context
- `ADR-0003` introduced stable exports via `$SUBSTRATE_HOME/env.sh` (sourced by `$SUBSTRATE_HOME/manager_env.sh`) to stabilize cached state for Substrate-owned shells and tooling.
- `PCM0/ADR-0003` also defined an effective-config precedence that places environment variables above workspace config.
- In practice, when a user’s shell is running with `SUBSTRATE_*` already exported (via sourcing `env.sh` directly, sourcing `manager_env.sh`, or inheriting an environment from a Substrate-owned shell), those values are indistinguishable from intentional operator overrides.
- This produces a confusing operator experience: workspace config appears not to take effect, even though “workspace overrides global” is the expected mental model for config files.

## Goals
- Ensure workspace config (`<workspace_root>/.substrate/workspace.yaml`) overrides any `SUBSTRATE_*` exports that are present in the environment due to stable export scripts.
- Preserve stable export scripts (`env.sh` + `manager_env.sh`) for Substrate-owned shells and managers without making them change the workspace-scoped effective config.
- Keep CLI flag precedence unchanged (CLI flags remain the highest precedence layer).
- Keep no-workspace behavior intuitive (env overrides remain available when there is no workspace marker).

## Non-Goals
- Redesign `$SUBSTRATE_HOME/env.sh` / `$SUBSTRATE_HOME/manager_env.sh` ownership rules (ADR-0003 DR-0005 remains in force).
- Introduce new environment variable namespaces (e.g., `SUBSTRATE_DEFAULT_*`) or rename existing `SUBSTRATE_*` variables.
- Change workspace discovery (`.substrate/workspace.yaml` walk-up semantics).
- Change policy precedence or policy file selection (this ADR is config-only).

## User Contract (Authoritative)

### CLI
- `substrate config show [--json]`
  - Requires a workspace (nearest ancestor containing `.substrate/workspace.yaml`), otherwise exits `2`.
  - Prints the effective config for `cwd` (YAML default, JSON with `--json`).
  - Effective-config precedence when `<workspace_root>` exists (highest to lowest):
    1. CLI flags (subset of keys with CLI flags)
    2. Workspace config (`<workspace_root>/.substrate/workspace.yaml`)
    3. Environment variables `SUBSTRATE_*`
    4. Global config (`$SUBSTRATE_HOME/config.yaml` or built-in defaults)
    5. Built-in defaults
- `substrate config set [--json] UPDATE...`
  - Applies updates to `<workspace_root>/.substrate/workspace.yaml` only.
  - Prints the effective config for `cwd` after applying updates (same precedence as `config show`).
- `substrate config global show [--json]`
  - Prints the contents of `$SUBSTRATE_HOME/config.yaml` if present; otherwise prints built-in defaults.
  - It MUST NOT print “effective config for `cwd`”, and it MUST NOT incorporate workspace config, env vars, or CLI flags.

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: success (including no-op)
- `2`: actionable user error (missing workspace for workspace-scoped commands; invalid update syntax; invalid value/type)
- `1`: unexpected failure

### Config
- Files and locations (precedence for effective config when `<workspace_root>` exists):
  1. CLI flags (subset)
  2. `<workspace_root>/.substrate/workspace.yaml`
  3. `SUBSTRATE_*` environment variables
  4. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
  5. Built-in defaults
- Strict parsing:
  - Unknown keys are a hard error.
  - Type mismatches are a hard error.
  - Presence of `<workspace_root>/.substrate/settings.yaml` is a hard error with an actionable message directing the user to `.substrate/workspace.yaml`.

### Platform guarantees
- Linux/macOS/Windows: identical precedence semantics for the config keys covered by ADR-0003/PCM0/PCM3; platform-specific behavior must be behind `#[cfg]` guards where needed, but precedence is cross-platform contract.

## Architecture Shape
- Components:
  - `crates/shell/src/execution/config_model.rs`
    - Computes effective config for a given `cwd`.
    - Change: apply workspace config before env overrides when `<workspace_root>` exists.
  - `crates/shell/src/execution/config_cmd.rs`
    - `substrate config show|set|global show|global set` entry points; no contract changes beyond the precedence update.
  - `crates/shell/tests/config_show.rs` (and/or other shell integration tests)
    - Add/adjust tests to lock the new precedence order for workspace-scoped commands when `SUBSTRATE_*` is present.
- End-to-end flow:
  - Inputs:
    - `cwd`
    - `$SUBSTRATE_HOME/config.yaml` (optional)
    - `<workspace_root>/.substrate/workspace.yaml`
    - process env (`SUBSTRATE_*`)
    - CLI overrides (subset)
  - Derived state:
    - `<workspace_root>` via walk-up marker discovery
    - effective config per precedence rules above
  - Actions:
    - `config show`: render effective config
    - `config set`: parse/apply updates to workspace config, then render effective config
  - Outputs:
    - YAML or JSON config mapping
    - Exit code per taxonomy

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `policy_and_config_precedence`
- Prerequisite integration task IDs:
  - None (this ADR is a follow-up correction to ADR-0003’s precedence contract; it can be implemented as a small dedicated triad under a new branch, e.g. `feat/policy_and_config_precedence`).

## Security / Safety Posture
- Fail-closed rules:
  - Invalid config YAML, unknown keys, or type mismatches remain hard errors.
  - Legacy `.substrate/settings.yaml` presence remains a hard error.
- Safety invariants:
  - Workspace-scoped config commands never write outside `<workspace_root>/.substrate/workspace.yaml`.
  - Global config commands never write outside `$SUBSTRATE_HOME/config.yaml` (and `$SUBSTRATE_HOME/env.sh` if applicable per existing ownership rules).
- Observability:
  - No new trace schema required by this ADR.

## Validation Plan (Authoritative)

### Tests
- Integration tests (required):
  - Update/add coverage in `crates/shell/tests/config_show.rs` to verify:
    - When `<workspace_root>` exists and `SUBSTRATE_CAGED=1` is present, `substrate config show` returns `world.caged` from workspace config (not env).
    - CLI flags still override workspace config for the subset of CLI-covered keys.
    - No-workspace behavior is unchanged (workspace-scoped commands still exit `2`).

### Manual validation
- Manual playbook (required):
  - `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`
  - Includes a repro that sources `$SUBSTRATE_HOME/env.sh` and confirms `substrate config show` reflects `.substrate/workspace.yaml` values for overlapping keys.
  - Repro: source `$SUBSTRATE_HOME/env.sh`, then verify `substrate config show` reflects `.substrate/workspace.yaml` values for overlapping keys.

### Smoke scripts
- Feature-local scripts (required):
  - `docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh`
  - `docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh`
  - `docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed, but this ADR is targeted to reduce surprising behavior.
- Compatibility:
  - This is a behavioral change: environment overrides no longer override workspace config when a workspace exists.
  - No environment variable names change; only precedence changes.

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/policy_and_config_precedence/decision_register.md` (DR-0001)
