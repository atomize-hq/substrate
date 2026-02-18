# ADR-0008 Planning Pack — Phase A/B Gates (ADR-0012) (Non-negotiable)

This file is an **additive execution constraint** for the Planning Pack under:
- `docs/project_management/_archived/workspace-config-policy-unification/`

It layers on top of all repo planning standards and must be treated as **non-negotiable acceptance criteria** for this body of work.

## Why this exists
`ADR-0012` introduces per-key merge strategies and multi-source provenance. `ADR-0008` is the queued body of work that will implement the config plumbing and CLI surfaces; therefore, this Planning Pack must explicitly include and complete the Phase A/B work defined here so downstream work (notably world-deps) can safely depend on it.

Authoritative ADRs:
- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`

Primary downstream consumer contract:
- `docs/project_management/next/world_deps_packages_bundles_contract.md`

## Phase A (must complete) — Per-key merge strategies + multi-source provenance

### A1) Schema supports per-key merge strategies
Requirements:
- Config schema registry includes an explicit per-key merge strategy.
- Minimum required strategies:
  - `replace` (default)
  - `concat_dedupe_ordered_set` (for `world.deps.enabled`)

Evidence required (integration task must record):
- Unit tests verifying:
  - `replace` yields single-source provenance
  - `concat_dedupe_ordered_set` yields correct merged list and deterministic multi-source provenance ordering

### A2) `config current show --explain` supports multi-source keys
Requirements:
- `--explain` output supports, for every effective key:
  - `merge_strategy`
  - `sources[]` (ordered list; supports >1 entry for merge keys)

Evidence required:
- Integration tests/golden coverage for:
  - global-only contributes to `world.deps.enabled`
  - workspace-only contributes to `world.deps.enabled`
  - global + workspace both contribute; effective list is concatenated and de-duped in-order
  - workspace disabled marker causes workspace contribution to be ignored

## Phase B (must complete) — Config editor supports `world.deps.*` keys

### B1) Allowlist includes `world.deps.enabled`
Requirements:
- `world.deps.enabled` is treated as a valid config key (not “unknown key”) and is editable via `substrate config ... set/reset`.

### B2) Mutations behave correctly for list merge keys
Requirements (minimum):
- `substrate config global set world.deps.enabled+=<item>` adds the item to the global patch list.
- `substrate config workspace set world.deps.enabled+=<item>` adds the item to the workspace patch list.
- `substrate config global reset world.deps.enabled` removes the key from the global patch (inherit-only).
- `substrate config workspace reset world.deps.enabled` removes the key from the workspace patch (inherit-only).

Evidence required:
- Integration tests asserting the above, plus that `config current show --explain` surfaces:
  - `merge_strategy=concat_dedupe_ordered_set`
  - `sources` includes both `global_patch` and `workspace_patch` when both contribute

## Phase B extension (must complete) — World-deps enum keys (contract parity)

Source of truth:
- `docs/project_management/next/world_deps_packages_bundles_contract.md`

This Planning Pack MUST also gate schema allowlisting, strict enum validation, config editor set/reset, and deterministic `--explain` provenance for these additional world-deps config keys:
- `world.deps.inventory_mode` (allowed values exactly: `merged`, `workspace_only`)
- `world.deps.builtins` (allowed values exactly: `enabled`, `disabled`)

### E1) Schema allowlisting + type/enum validation (required)
Requirements:
- The config schema allowlist includes:
  - `world.deps.inventory_mode` with type `enum[string]` (or equivalent “string with allowed set”) and allowed values exactly: `merged`, `workspace_only`.
  - `world.deps.builtins` with type `enum[string]` and allowed values exactly: `enabled`, `disabled`.
- Unknown key handling remains exit `2` with no writes.
- Invalid enum values are exit `2` with no writes (patch file bytes unchanged; comment header preserved).

### E2) Merge strategy + provenance (replace) (required)
Requirements:
- For both keys, effective merge strategy is `replace` (highest-precedence layer that sets the key wins).
- `substrate config current show --explain` MUST:
  - report `merge_strategy=replace` for these keys when present in the effective view,
  - report exactly one contributing source layer for these keys (`workspace_patch` OR `global_patch` OR `default`, as applicable),
  - remain deterministic/idempotent byte-for-byte for identical inputs.

### E3) Config editor mutation surface (required)
The config editor MUST support, at minimum:
- `substrate config global set world.deps.inventory_mode=<merged|workspace_only>`
- `substrate config workspace set world.deps.inventory_mode=<merged|workspace_only>`
- `substrate config global set world.deps.builtins=<enabled|disabled>`
- `substrate config workspace set world.deps.builtins=<enabled|disabled>`
- `substrate config global reset world.deps.inventory_mode` (removes the key from the patch mapping; inherit-only)
- `substrate config workspace reset world.deps.inventory_mode`
- `substrate config global reset world.deps.builtins`
- `substrate config workspace reset world.deps.builtins`

Workspace-disable interaction requirement:
- When the workspace is disabled via `.substrate/workspace.disabled`, current views MUST ignore workspace patch contribution, so effective values and `--explain` provenance fall back to global/default.

## Manual + smoke validation requirements (must be wired into tasks)

### Manual playbook
The feature `manual_testing_playbook.md` MUST include a section that validates:
- a merged `world.deps.enabled` across global + workspace via the config editor, and
- multi-source provenance via `config current show --explain`.

### Smoke scripts
The feature smoke scripts MUST include a corresponding automated journey that:
- creates a scratch workspace and scratch `SUBSTRATE_HOME`,
- applies global + workspace edits to `world.deps.enabled`,
- validates:
  - effective merged list,
  - `--explain` multi-source provenance,
  - determinism/idempotence (re-running yields identical results).
The feature smoke scripts MUST also validate the “World-deps enum keys” requirements above (E1–E3), including strict invalid-enum behavior and workspace-disable fallbacks.

## Task wiring requirements (must be reflected in `tasks.json`)
- At least one execution slice must explicitly own Phase A+B completion (recommended: the config merge/CLI slices).
- Slice integration tasks MUST:
  - reference this file,
  - run feature smoke scripts for behavior platforms, and
  - record results/run ids/URLs in the slice closeout report.
