# WCU3 Spec — CLI scopes + reset semantics + config editor support for `world.deps.enabled` (ADR-0012 Phase B)

References:
- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Implement explicit `current|global|workspace` scope CLIs for config and policy (ADR-0008).
- Implement `set`/`reset` semantics for patch files (sparse YAML mappings).
- Phase B: allowlist `world.deps.enabled` and support list-merge key mutations via the config editor.

## Non-goals
- Adding new scopes beyond `current|global|workspace`.
- Adding policy merge keys in this slice unless explicitly required by ADR-0008/ADR-0012.

## Config editor allowlist (Phase B; authoritative)
- `world.deps.enabled` MUST be recognized as a valid config key (not “unknown key”).
- Type MUST be `list[string]`.
- `world.deps.inventory_mode` MUST be recognized as a valid config key (not “unknown key”).
  - Type MUST be `enum[string]` (or equivalent “string with allowed set”).
  - Allowed values MUST be exactly: `merged`, `workspace_only`.
- `world.deps.builtins` MUST be recognized as a valid config key (not “unknown key”).
  - Type MUST be `enum[string]` (or equivalent “string with allowed set”).
  - Allowed values MUST be exactly: `enabled`, `disabled`.

## Patch mutation syntax (Phase B; authoritative)
The config editor MUST support the following mutation forms (exact spellings):
- Append item (global):
  - `substrate config global set world.deps.enabled+=<item>`
- Append item (workspace):
  - `substrate config workspace set world.deps.enabled+=<item>`
- Remove item (global):
  - `substrate config global set world.deps.enabled-=<item>`
- Remove item (workspace):
  - `substrate config workspace set world.deps.enabled-=<item>`
- Reset key (global; inherit-only):
  - `substrate config global reset world.deps.enabled`
- Reset key (workspace; inherit-only):
  - `substrate config workspace reset world.deps.enabled`
- Set enum (global):
  - `substrate config global set world.deps.inventory_mode=<merged|workspace_only>`
- Set enum (workspace):
  - `substrate config workspace set world.deps.inventory_mode=<merged|workspace_only>`
- Set enum (global):
  - `substrate config global set world.deps.builtins=<enabled|disabled>`
- Set enum (workspace):
  - `substrate config workspace set world.deps.builtins=<enabled|disabled>`
- Reset enum key (global; inherit-only):
  - `substrate config global reset world.deps.inventory_mode`
- Reset enum key (workspace; inherit-only):
  - `substrate config workspace reset world.deps.inventory_mode`
- Reset enum key (global; inherit-only):
  - `substrate config global reset world.deps.builtins`
- Reset enum key (workspace; inherit-only):
  - `substrate config workspace reset world.deps.builtins`

Rules:
- `+=<item>` appends `<item>` to the patch list for that scope.
- The patch list MUST be de-duplicated in-order at write time (re-appending an existing item is a no-op for the on-disk patch list).
- `-=<item>` removes `<item>` from the patch list for that scope (exact match).
  - If `<item>` is not present: no-op (exit `0`).
  - If the resulting list is empty: the key remains present with value `[]` (explicit empty list; see DR-0015).
- `reset <key>` removes `<key>` from the patch mapping at that scope (does not write an explicit null/empty sentinel).
- Invalid enum values for `world.deps.inventory_mode` or `world.deps.builtins` MUST:
  - exit `2`, and
  - perform no writes (patch file bytes unchanged; comment header preserved).

## `config current show --explain` visibility (Phase B; authoritative)
When both global and workspace patches contribute to `world.deps.enabled`:
- `merge_strategy` MUST be `concat_dedupe_ordered_set`
- `sources` MUST include both `global_patch` and `workspace_patch` (in application order)

For `world.deps.inventory_mode` and `world.deps.builtins` (both `merge_strategy=replace`):
- `merge_strategy` MUST be `replace`.
- `sources` MUST contain exactly one contributing layer in the effective view (`workspace_patch` OR `global_patch` OR `default`, as applicable).
- Workspace-disable interaction:
  - When the workspace is disabled via `.substrate/workspace.disabled`, current views MUST ignore workspace patch contribution, so effective values and `--explain` provenance fall back to global/default.

## Exit codes (authoritative)
Use `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.
- Unknown key / type mismatch / invalid mutation syntax: exit `2` and perform no writes.

## Validation requirements (authoritative; Phase B evidence)
- Integration tests MUST cover:
  - global and workspace appends to `world.deps.enabled` via `+=`
  - global and workspace removals from `world.deps.enabled` via `-=`
  - global/workspace `reset world.deps.enabled` removes the key from the patch mapping
  - global and workspace `set`/`reset` for `world.deps.inventory_mode` and `world.deps.builtins` with valid values
  - invalid enum values for `world.deps.inventory_mode` and `world.deps.builtins` are exit `2` and perform no writes (patch bytes unchanged; comment header preserved)
  - workspace disabled marker ignores workspace patch contribution for these keys (effective values and `--explain` provenance fall back to global/default)
  - effective view reflects concat+dedupe semantics
  - `--explain` includes `merge_strategy` and both contributing sources when both scopes contribute
