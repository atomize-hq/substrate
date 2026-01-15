# Integration Map — Workspace Config/Policy Unification (ADR-0008) + ADR-0012

## Scope
This integration map covers the implementation work queued off:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

And the cross-cutting refinement that must be implemented as part of this work:
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`

Non-negotiable gate file for this Planning Pack:
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`

## Adjacent / downstream consumers (dependency awareness)
- World deps packages/bundles contract depends on:
  - schema-defined per-key merge strategies
  - multi-source provenance in `config current show --explain`
  - config editor allowlisting/edit support for `world.deps.enabled`
- Reference:
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`

## End-to-end flow (config)

### Inputs
- CLI flags (subset; ADR-0008)
- `SUBSTRATE_OVERRIDE_*` (explicit operator one-off inputs; ADR-0008)
- Workspace config patch:
  - `<workspace_root>/.substrate/workspace.yaml` (when workspace exists and is enabled)
- Global config patch:
  - `$SUBSTRATE_HOME/config.yaml`
- Built-in defaults
- Injected protected excludes:
  - `.git/**`
  - `.substrate/**`

### Derived state
- Workspace root discovery (walk up from `cwd`, ignoring disabled workspaces)
- Patch parsing (sparse YAML mapping semantics)
- Schema validation (unknown keys/type mismatches are exit `2`)
- Effective config resolution:
  - For most keys: `replace` merge strategy (highest precedence wins)
  - For selected keys: schema-defined merge strategies (ADR-0012), including multi-layer derived values

### Outputs
- Patch views:
  - `substrate config global show` prints exactly `$SUBSTRATE_HOME/config.yaml` (or `{}` if missing)
  - `substrate config workspace show` prints exactly `<workspace_root>/.substrate/workspace.yaml`
- Effective views:
  - `substrate config current show` prints effective config for `cwd`
  - `substrate config current show --explain` prints deterministic provenance (ADR-0012: supports multi-source keys)

## Components (code landing zones)
- `crates/shell/src/execution/config_model.rs`:
  - config patch parsing + validation
  - schema registry for allowed keys/types
  - effective config resolution using schema-defined per-key merge strategy (ADR-0012)
  - provenance emission for `--explain` (ADR-0012: multi-source keys)
- `crates/shell/src/execution/policy_model.rs`:
  - policy patch parsing + validation + merge (ADR-0008); optional future parity with ADR-0012 provenance shape
- `crates/shell/src/execution/config_cmd.rs` / `policy_cmd.rs`:
  - `current|global|workspace` command surfaces
  - `set/reset/init` write semantics and header preservation
- `crates/shell/src/execution/workspace.rs` / `workspace_cmd.rs`:
  - workspace root discovery + `.substrate/` layout + disable marker behavior
- Install/dev scripts:
  - stop exporting `SUBSTRATE_OVERRIDE_*` by default (ADR-0008 WCU4)

## Phase A/B ownership (ADR-0012)
This Planning Pack must complete:
- Phase A (merge strategies + multi-source provenance): owned by WCU2
- Phase B (config editor supports `world.deps.enabled`): owned by WCU3

Explicit gate file:
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
