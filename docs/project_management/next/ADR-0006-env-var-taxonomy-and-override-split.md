# ADR-0006 — Env Var Taxonomy + Override Split

## Status
- Status: Accepted
- Date (UTC): 2026-01-04
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/_archived/env_var_taxonomy_and_override_split/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

## Related Docs
- Prior ADRs:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
- Plan: `docs/project_management/_archived/env_var_taxonomy_and_override_split/plan.md`
- Tasks: `docs/project_management/_archived/env_var_taxonomy_and_override_split/tasks.json`
- Session log: `docs/project_management/_archived/env_var_taxonomy_and_override_split/session_log.md`
- Spec: `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`
- Decision Register: `docs/project_management/_archived/env_var_taxonomy_and_override_split/decision_register.md`
- Integration Map: `docs/project_management/_archived/env_var_taxonomy_and_override_split/integration_map.md`
- Manual Playbook: `docs/project_management/_archived/env_var_taxonomy_and_override_split/manual_testing_playbook.md`
- Smoke:
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/linux-smoke.sh`
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/macos-smoke.sh`
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/windows-smoke.ps1`
- Canonical env-var catalog: `docs/ENVIRONMENT_VARIABLES.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: eddd1c11e664d6ac18e81d2f897fff24c43e91fa218583edf32e1ea5575d0f07
### Changes (operator-facing)
- Split exported state from override inputs
  - Existing: `SUBSTRATE_*` values can be present in the environment because they are exported by `$SUBSTRATE_HOME/env.sh` (stable exports), but some of those same variables are also treated as operator overrides by config resolution. This creates “stale export” surprises where config edits appear not to take effect without re-sourcing.
  - New: Exported state variables remain `SUBSTRATE_*`, but config resolution stops treating those state exports as override inputs. Operator override inputs move to a dedicated namespace: `SUBSTRATE_OVERRIDE_*`.
  - Why: Prevent confusion and eliminate the dual-use ambiguity where “cached state exports” look like intentional overrides.
  - Links:
    - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md#L608` (`env.sh` / `manager_env.sh` ownership model)
    - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md#L33` (prior mitigation for workspace vs env)
    - `crates/shell/src/execution/env_scripts.rs` (exported state generation)
    - `crates/shell/src/execution/config_model.rs#L220` (effective config resolution)
    - `docs/ENVIRONMENT_VARIABLES.md` (canonical taxonomy + catalog)

## Problem / Context
- Substrate uses stable exports (`$SUBSTRATE_HOME/env.sh`) so shims/subprocesses can observe consistent state.
- Substrate also consults environment variables as configuration inputs (override layer) when resolving effective config.
- Today, several `SUBSTRATE_*` variables participate in both roles:
  - They are written into `env.sh` as exported state, and
  - They are read as override inputs by config resolution.
- This dual-use makes it easy to end up with “stale exports” in a parent shell and observe behavior that does not appear to match the config files on disk.

## Goals
- Define a single, repo-wide environment variable taxonomy and catalog that classifies:
  - exported state,
  - override inputs,
  - internal plumbing variables,
  - diagnostics/test-only toggles.
- Remove the dual-use ambiguity for config-shaped `SUBSTRATE_*` keys by:
  - keeping `SUBSTRATE_*` for exported state, and
  - introducing `SUBSTRATE_OVERRIDE_*` for operator override inputs.
- Ensure that editing `config.yaml` / `workspace.yaml` is reflected immediately in:
  - `substrate -c ...` executions, and
  - interactive Substrate shell/REPL entry points,
  without requiring users to re-source `env.sh`.
- Align dev installer behavior with production semantics: dev setup MUST NOT require sourcing stable exports into the parent shell to validate config behavior.

## Non-Goals
- Backwards compatibility for legacy environment variable semantics (greenfield).
- Changes to workspace discovery semantics or workspace marker locations.
- Redesign of policy schema, policy file discovery, or policy evaluation semantics.
- Trace schema changes (unless required by implementation triads).

## User Contract (Authoritative)

### CLI
- Commands and behavior are unchanged unless explicitly noted here; precedence semantics change as described below.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `1`: unexpected failure
  - `2`: actionable user error (invalid config, invalid override value, missing workspace for workspace-scoped commands)
  - `3`: required dependency unavailable (playbooks/smoke only)

### Config
- Scope:
  - Global config: `$SUBSTRATE_HOME/config.yaml`
  - Workspace config: `<workspace_root>/.substrate/workspace.yaml`
- Effective config precedence (highest to lowest) when `<workspace_root>` exists:
  1. CLI flags (subset)
  2. `<workspace_root>/.substrate/workspace.yaml`
  3. `SUBSTRATE_OVERRIDE_*` environment variables (subset)
  4. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
  5. Built-in defaults
- Effective config precedence (highest to lowest) when no workspace exists:
  1. CLI flags (subset)
  2. `SUBSTRATE_OVERRIDE_*` environment variables (subset)
  3. `$SUBSTRATE_HOME/config.yaml` (or built-in defaults)
  4. Built-in defaults
- Exported state variables (`SUBSTRATE_*`) are output-only for config resolution:
  - They MUST NOT be consulted as override inputs for effective config.
  - They may be written by `env.sh` and by runtime Substrate components for propagation.
- Strict parsing:
  - Override env vars are strict: unknown keys are ignored (not supported) and known keys must parse correctly (type/enum constraints) or fail with exit `2`.

### Environment variable taxonomy
- Canonical catalog: `docs/ENVIRONMENT_VARIABLES.md`
- Required categories:
  - `State` (exported): `SUBSTRATE_*` variables written by Substrate-owned scripts/runtime for propagation.
  - `Override` (inputs): `SUBSTRATE_OVERRIDE_*` variables set by operators/tests to override config resolution.
  - `Internal`: `SHIM_*` and other internal coordination variables.
  - `Diagnostics/Test`: explicit toggles intended for debugging/tests.
- Namespace rule:
  - Operator overrides MUST use `SUBSTRATE_OVERRIDE_*`.
  - Exported state MUST use `SUBSTRATE_*` and must not be read as overrides by the config resolver.

### Policy
- Policy files remain file-backed and are reloaded from disk by the broker:
  - Workspace policy: `<workspace_root>/.substrate/policy.yaml`
  - Global policy: `$SUBSTRATE_HOME/policy.yaml`
- The env split does not change policy discovery or evaluation behavior.

### Platform guarantees
- Linux/macOS/Windows: taxonomy and precedence rules are identical; platform-specific behavior remains behind `#[cfg]` guards where needed.

## Architecture Shape
- Components (expected):
  - `crates/shell/src/execution/config_model.rs`: stop treating exported state `SUBSTRATE_*` as overrides; read `SUBSTRATE_OVERRIDE_*` for override inputs.
  - `crates/shell/src/execution/env_scripts.rs`: continue generating `env.sh` as exported state.
  - `crates/shell/src/execution/invocation/plan.rs`: ensure effective config is derived from config files + `SUBSTRATE_OVERRIDE_*` (not exported state).
  - Install scripts: no contract change is required by this ADR; correctness is achieved by the config resolver no longer consulting exported-state values as override inputs.
  - Docs:
    - `docs/CONFIGURATION.md`: reference the canonical catalog and taxonomy rules.
    - `docs/ENVIRONMENT_VARIABLES.md`: single source of truth catalog.
- End-to-end flow (effective config):
  - Inputs: CLI flags, `SUBSTRATE_OVERRIDE_*`, config files (workspace/global), built-in defaults
  - Derived state: workspace root (if any), effective config
  - Actions: command execution (shell/world/shim), config printing/updating
  - Outputs: runtime exported state env vars (`SUBSTRATE_*`), logs/trace

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `env_var_taxonomy_and_override_split`
- Prerequisite integration task IDs:
  - `docs/project_management/_archived/policy_and_config_precedence/tasks.json` → `PCP0-integ`

## Security / Safety Posture
- Fail-closed rules:
  - Invalid override values fail with exit `2` and actionable messages.
  - Unknown override keys are rejected by omission (only the documented subset is supported).
- Protected paths/invariants:
  - No new writes outside `$SUBSTRATE_HOME` and `<workspace_root>/.substrate/` for config/policy commands.
- Observability:
  - The canonical environment variable catalog must document which variables may contain sensitive data and whether they are logged/recorded.

## Validation Plan (Authoritative)

### Tests
- Update/add tests to ensure:
  - Exported state variables in `env.sh` do not influence effective config resolution.
  - `SUBSTRATE_OVERRIDE_*` values do influence effective config resolution per precedence rules.
  - Workspace config still wins over override env vars where applicable.
  - At least one non-policy key is covered (ex: `world.caged` and/or `world.anchor_mode`) so partial implementations can’t pass by only wiring policy mode.

### Exhaustive repo audit (required)
- During EV0 implementation review, perform a repo-wide grep/audit to confirm no commands bypass effective config resolution by reading config-shaped `SUBSTRATE_*` values directly as *inputs*.
- Audit command baseline (run from repo root):
  - `rg -n "SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)" -S crates src scripts`
  - `rg -n "env::var(_os)?\\(\\\"SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)\\\"\\)" -S crates`
- Required outcome:
  - Any non-test read of those legacy `SUBSTRATE_*` names that can change behavior MUST be eliminated or rewritten to consult effective config / `SUBSTRATE_OVERRIDE_*` instead.
  - Remaining reads MUST be justified as derived/exported-state consumption (value set earlier in-process from effective config), not as operator override inputs.
  - Evidence is recorded in `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-closeout_report.md`.

### Manual testing
- Follow `docs/project_management/_archived/env_var_taxonomy_and_override_split/manual_testing_playbook.md`.

### Smoke scripts
- Feature-local smoke scripts live under:
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/`
  - Smoke must validate policy mode plus multiple non-policy keys (ex: `world.caged`, `world.anchor_mode`) so behavior cannot regress via partial wiring.

## Rollout / Backwards Compatibility
- Greenfield breaking is allowed.
- No backwards compatibility is required: legacy semantics for config-shaped `SUBSTRATE_*` override inputs are removed in favor of `SUBSTRATE_OVERRIDE_*`.

## Decision Summary
- Decision Register:
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/decision_register.md`
    - DR-0001: Naming scheme for override inputs
    - DR-0002: Scope of override variables
    - DR-0003: Canonical environment variable catalog location
