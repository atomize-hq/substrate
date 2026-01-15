# WCU2 Spec — Patch parsing + merge (ADR-0008) + per-key merge strategies (ADR-0012 Phase A)

References:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Extend the config schema registry to include an explicit per-key `merge_strategy`.
- Implement effective config resolution that applies the schema-defined merge strategy per key.
- Implement deterministic multi-source provenance for `substrate config current show --explain` (Phase A).

## Non-goals
- Adding new config scopes beyond `current|global|workspace`.
- Any migrations/backwards compatibility beyond what ADR-0008/ADR-0012 explicitly describe.

## Merge strategy taxonomy (authoritative)
The config schema MUST support, at minimum:
- `replace` (default for keys that do not declare a strategy)
- `concat_dedupe_ordered_set` (list[string] only)

### `replace` (default)
- Effective value is taken from the highest-precedence layer that defines the key.
- `--explain` provenance MUST report exactly one source layer for this key.

### `concat_dedupe_ordered_set`
- Applies to list[string] keys (not nested objects).
- Effective value is computed by:
  1) concatenating contributing lists in precedence order (lower → higher), then
  2) de-duplicating in-order where the **first occurrence wins**.
- Contribution semantics:
  - key omitted at a layer: contributes nothing.
  - key present as `[]` at a layer: contributes an explicit empty list (still produces no new items, but counts as a contributing layer for `--explain`).

## Phase A key assignment (authoritative)
The schema MUST assign:
- `world.deps.enabled`: type `list[string]`, `merge_strategy=concat_dedupe_ordered_set`.
- `world.deps.inventory_mode`: type `enum[string]` (or equivalent “string with allowed set”), allowed values exactly `merged` and `workspace_only`, `merge_strategy=replace`.
- `world.deps.builtins`: type `enum[string]` (or equivalent “string with allowed set”), allowed values exactly `enabled` and `disabled`, `merge_strategy=replace`.

## `config current show --explain` provenance contract (authoritative)

### Output channel and format
- `--explain` MUST emit a machine-readable JSON object to **stderr**.
- `stdout` output MUST remain the effective view output (`--json` controls stdout format).
- Determinism requirement:
  - For identical inputs, the stderr JSON bytes MUST be identical.
  - The output MUST NOT include timestamps, random ordering, or non-deterministic fields.

### Provenance payload (selected contract)
The stderr JSON object MUST have this shape:
```json
{
  "kind": "substrate.config.explain.v1",
  "keys": {
    "<dotpath>": {
      "merge_strategy": "<replace|concat_dedupe_ordered_set>",
      "sources": [
        { "layer": "<source_label>", "path": "<optional path>" }
      ]
    }
  }
}
```
Rules:
- `keys` is keyed by **dotpath** (e.g. `world.deps.enabled`, `world.caged`).
- The `keys` object MUST be serialized with keys in lexicographic order (dotpath sort) to guarantee deterministic bytes.
- `sources` is an ordered list of contributing layers in the exact order they were applied to build the effective value (lower → higher for patch layers).
- Each `sources[]` entry MUST include:
  - `layer`: one of the ADR-0012 required labels (`cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`, `injected_protected`)
  - `path`: present only for patch-file layers (`global_patch`, `workspace_patch`), and equal to the resolved file path.

Replace-key provenance requirements (authoritative):
- For `world.deps.inventory_mode` and `world.deps.builtins` (both `merge_strategy=replace`), `sources` MUST contain exactly one entry in the effective view:
  - `workspace_patch` when the workspace patch provides the effective value (workspace exists and is enabled),
  - otherwise `global_patch` when the global patch provides the effective value,
  - otherwise `default`.

## Exit codes (authoritative)
Use `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.
- Invalid YAML / schema/type mismatches / unknown keys: exit `2` and perform no writes.

## Validation requirements (authoritative)

### Unit tests (required; Phase A evidence)
- `replace` keys:
  - effective value is taken from the single highest-precedence layer that defines the key
  - provenance `sources` contains exactly one entry
- `concat_dedupe_ordered_set` keys:
  - effective list is concatenated + de-duped in-order
  - provenance `sources` includes all contributing layers in deterministic order
- Determinism:
  - For identical inputs, `--explain` stderr JSON bytes are identical

### Integration tests / goldens (required; Phase A evidence)
`config current show --explain` must be covered for `world.deps.enabled` with:
- global-only contribution
- workspace-only contribution
- global + workspace contribution (concat + dedupe correctness; provenance includes both)
- workspace disabled marker present (workspace contribution ignored; provenance excludes `workspace_patch`)
