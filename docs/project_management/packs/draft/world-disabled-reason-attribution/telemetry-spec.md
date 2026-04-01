# world-disabled-reason-attribution — telemetry spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope
- This spec is authoritative for replay trace and log changes introduced by ADR-0038.
- Surface in scope:
  - `replay_strategy` entries emitted by `crates/replay/src/replay/executor.rs`

## Stability guarantees
- Field stability:
  - existing `replay_strategy` fields remain unchanged
  - new fields are additive only
- Consumer compatibility:
  - consumers that ignore unknown fields keep working
  - consumers that parse `origin_reason_code` gain new enum values only in the effective-disable case

## Trace and log schema changes

### Existing fields reused
- `origin_reason`
  - Type: string
  - When emitted: replay has a reason fragment for host or world selection
  - Rule for ADR-0038: when effective-disable attribution applies, this field equals the exact reason fragment from `contract.md`

- `origin_reason_code`
  - Type: string enum
  - New enum values:
    - `world_disabled_override_env`
    - `world_disabled_workspace_patch`
    - `world_disabled_global_patch`
    - `world_disabled_unknown`
  - When emitted: same condition as `origin_reason`

### New additive field
- `world_disable_source`
  - Type: object
  - Cardinality: optional, emitted only when `origin_reason_code` is one of the `world_disabled_*` values above
  - Keys:
    - `key` — string, always `world.enabled`
    - `layer` — string enum, one of `override_env`, `workspace_patch`, `global_patch`, `unknown`
    - `env` — optional string, only `SUBSTRATE_OVERRIDE_WORLD` when `layer=override_env`
    - `path_display` — optional string, only `<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml`
    - `value_display` — string, always `false`
  - Redaction rules:
    - no absolute host paths
    - no raw env values
    - no extra keys

## Metrics
- None.

## Acceptance criteria
- Schema assertions:
  - `replay_strategy` keeps existing fields intact.
  - `world_disable_source` appears only for effective-disable attribution.
  - `origin_reason_code` uses the new `world_disabled_*` values only for effective-disable attribution.
- Redaction assertions:
  - `path_display` is tokenized.
  - `env` is the variable name only.
  - no absolute host path and no raw env value appears in the emitted object.
