# Integration Map — Env Var Taxonomy + Override Split

## Scope
- Implement the ADR-0006 override split for effective config resolution:
  - Override inputs: `SUBSTRATE_OVERRIDE_*`
  - Exported state: `SUBSTRATE_*` (output-only for config resolution)
- Ensure the canonical env-var catalog and configuration docs reflect the taxonomy and split.

## Non-Scope
- Changes to world backend transports (`world*`, `forwarder`, `host-proxy`).
- Policy schema changes or policy discovery changes.
- Trace schema changes.

## End-to-end data flow (inputs → derived state → actions → outputs)

### Inputs
- CLI flags (subset): `--world|--no-world`, `--anchor-mode`, `--anchor-path`, `--caged|--uncaged`
- Config files:
  - Global: `$SUBSTRATE_HOME/config.yaml`
  - Workspace: `<workspace_root>/.substrate/workspace.yaml`
- Override env vars (inputs): `SUBSTRATE_OVERRIDE_*` (subset; config-shaped only)
- Exported state env vars (outputs): `SUBSTRATE_*` exported via `env.sh` and runtime

### Derived state
- Effective config (`SubstrateConfig`) resolved by `crates/shell/src/execution/config_model.rs`

### Actions
- Shell invocation plan consumes effective config to determine world enablement and policy mode propagation.
- `substrate config` commands read/write config files and render config.

### Outputs
- Stable exports (`$SUBSTRATE_HOME/env.sh`) continue to export `SUBSTRATE_*` state.
- Effective config resolution ignores config-shaped `SUBSTRATE_*` values as override inputs.
- Docs:
  - `docs/ENVIRONMENT_VARIABLES.md` remains the canonical catalog.
  - `docs/CONFIGURATION.md` references the catalog and the override split.

## Required implementation audit (no bypass reads)

ADR-0006’s intent is repo-wide: “stable exports” must be safe to have in the environment without silently acting as overrides. EV0 therefore requires an explicit audit to catch bypass reads outside the resolver.

Audit rule:
- Non-test code MUST NOT treat config-shaped legacy `SUBSTRATE_*` values as operator override inputs. If a component needs config, it MUST consult effective config and/or the dedicated override namespace (`SUBSTRATE_OVERRIDE_*`).

Required audit commands (run from repo root):
```bash
rg -n "SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)" -S crates src scripts
rg -n "env::var(_os)?\\(\"SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)\"\\)" -S crates
```

Evidence requirement:
- The final EV0 integration MUST include a summary of hits + disposition (fixed / derived-state-only / test-only) in `EV0-closeout_report.md`.

## Component map (what changes where)

### `crates/shell`
- `crates/shell/src/execution/config_model.rs`
  - Read `SUBSTRATE_OVERRIDE_*` for config-shaped override inputs.
  - Stop reading config-shaped `SUBSTRATE_*` values as override inputs.
  - Preserve strict parsing behavior for override inputs.
- `crates/shell/src/execution/invocation/plan.rs`
  - No new contract surface; continues to consume effective config for world enablement decisions.

### `crates/shim`, `crates/world*`
- No contract changes are required by this feature; they continue to treat `SUBSTRATE_*` as runtime state.

### Docs
- `docs/ENVIRONMENT_VARIABLES.md`
  - Canonical taxonomy and catalog; includes the reserved `SUBSTRATE_OVERRIDE_*` names.
- `docs/CONFIGURATION.md`
  - Must treat `SUBSTRATE_*` as exported state and direct override guidance to `SUBSTRATE_OVERRIDE_*` for config-shaped keys.

## Composition with adjacent tracks (dependencies)
- `docs/project_management/_archived/policy_and_config_precedence/` (ADR-0005 / PCP0):
  - PCP0 is sequenced first; it defines the workspace-over-env precedence correction in the same resolver layer.
  - This feature builds on that resolver layer and replaces the env override namespace for config-shaped keys.

## Sequencing alignment (final)
- Sequencing entry: `docs/project_management/next/sequencing.json` → sprint id `env_var_taxonomy_and_override_split`
- Sequenced after: `policy_and_config_precedence` (order 26)
