# WCU3 Spec — CLI scopes + reset semantics + config editor support for `world.deps.enabled` (ADR-0012 Phase B)

References:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
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

Rules:
- `+=<item>` appends `<item>` to the patch list for that scope.
- The patch list MUST be de-duplicated in-order at write time (re-appending an existing item is a no-op for the on-disk patch list).
- `-=<item>` removes `<item>` from the patch list for that scope (exact match).
  - If `<item>` is not present: no-op (exit `0`).
  - If the resulting list is empty: the key remains present with value `[]` (explicit empty list; see DR-0015).
- `reset <key>` removes `<key>` from the patch mapping at that scope (does not write an explicit null/empty sentinel).

## `config current show --explain` visibility (Phase B; authoritative)
When both global and workspace patches contribute to `world.deps.enabled`:
- `merge_strategy` MUST be `concat_dedupe_ordered_set`
- `sources` MUST include both `global_patch` and `workspace_patch` (in application order)

## Exit codes (authoritative)
Use `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.
- Unknown key / type mismatch / invalid mutation syntax: exit `2` and perform no writes.

## Validation requirements (authoritative; Phase B evidence)
- Integration tests MUST cover:
  - global and workspace appends to `world.deps.enabled` via `+=`
  - global and workspace removals from `world.deps.enabled` via `-=`
  - global/workspace `reset world.deps.enabled` removes the key from the patch mapping
  - effective view reflects concat+dedupe semantics
  - `--explain` includes `merge_strategy` and both contributing sources when both scopes contribute
