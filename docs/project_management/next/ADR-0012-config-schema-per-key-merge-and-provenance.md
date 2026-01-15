# ADR-0012 — Config Schema: Per-key Merge Strategies + Multi-source Provenance

## Status
- Status: Approved
- Date (UTC): 2026-01-14
- Owner(s): Shell maintainers

## Scope
- Feature directories (impacted):
  - `docs/project_management/next/` (this ADR; cross-cutting contract)
  - `docs/project_management/next/workspace-config-policy-unification/` (ADR-0008 implementation)
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Patch files + scope model:
  - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- World-deps consumer contract:
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
- World-deps ADR (consumer workstream; not modified by this ADR):
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 2f2164b8a8ec3e2a97d84b88fd530550c7ec9e2fc819239c8feaf91d49a89410

ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md` after drafting>

### Changes (operator-facing)
- Config “effective/current” views can explain multi-layer keys
  - Existing: `current show --explain` assumes each effective key comes from exactly one source layer.
  - New: `current show --explain` supports keys whose effective value is derived from multiple layers (e.g. global + workspace), and reports those contributing sources deterministically.
  - Why: enables additive config keys (like `world.deps.enabled`) without creating a second parallel config system or confusing precedence.
  - Links:
    - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
    - `docs/project_management/next/world_deps_packages_bundles_contract.md`

- Schema defines merge behavior per key
  - Existing: patch files imply a single merge rule (“workspace overrides global”) for all keys.
  - New: patch files remain the container format, but the schema defines the effective merge strategy per key; most keys remain “replace”, while selected keys are explicitly “merge”.
  - Why: keeps patch files simple while allowing a small number of keys to be intentionally additive and still explainable.
  - Links:
    - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Problem / Context
- ADR-0008 establishes:
  - patch files as sparse YAML mappings,
  - explicit scopes (`current|global|workspace`), and
  - `current show --explain` provenance for debugging precedence.
- Upcoming features need at least one config key whose effective value is intentionally derived from multiple layers (e.g. a merged enabled list for world-deps).
- Without an explicit “per-key merge strategy” contract:
  - we either force all list keys to be overriding-only (hurting UX), or
  - we introduce ad-hoc special-casing that can’t be surfaced in `--explain`, undermining the “explainable config” goal.

## Goals
- Preserve ADR-0008 patch files as the canonical container format (no new config file formats).
- Define a small merge-strategy taxonomy for config keys.
- Define an operator-visible `--explain` contract that supports multi-layer derived values.
- Make `world.deps.enabled` explicitly additive via a deterministic ordered-set merge.

## Non-Goals
- Defining new policy merge behavior beyond what ADR-0008 already specifies (this ADR primarily targets config keys).
- Backwards compatibility guarantees for any pre-existing `--explain` output; this is a greenfield contract refinement prior to implementation.
- Adding new scopes beyond `current|global|workspace`.

## User Contract (Authoritative)

### Patch files remain the container format
- Patch files remain sparse YAML mappings as defined by ADR-0008:
  - omitted keys mean “inherit”,
  - explicit values override at that scope,
  - mutating CLIs preserve comment headers.
- This ADR adds one rule:
  - The **schema** is authoritative for how a key’s effective value is computed across layers (merge strategy), even when the patch file representation is just YAML.

### Merge strategy taxonomy (effective resolution)

Each config key has a schema-defined merge strategy that governs how the **effective (current)** value is derived from layers.

Required strategies:
- `replace` (default):
  - Effective value comes from the highest-precedence layer that defines the key (workspace patch overrides global patch overrides defaults), consistent with ADR-0008’s “patch override” mental model.
- `concat_dedupe_ordered_set`:
  - Applicable to list[string] keys where the effective value should be additive across scopes.
  - Effective value is computed by concatenating contributing lists in precedence order (lower → higher), then de-duplicating in-order (first occurrence wins).
  - “Contributes nothing” vs “explicit empty list”:
    - key omitted at a scope: contributes nothing,
    - key present as `[]` at a scope: contributes an explicit empty list (still contributes nothing to the merge result, but is visible in patch views and is treated as an explicit setting).

### `current show --explain` supports multi-source keys

This ADR updates ADR-0008’s “per-key source” assumption to support multi-layer derived values.

#### Scope
- Applies to:
  - `substrate config current show --explain`
  - (Optional but recommended for symmetry) `substrate policy current show --explain`

#### Output contract (machine-readable, stderr)
- `--explain` MUST emit a machine-readable provenance object to stderr that includes, for every effective key:
  - `merge_strategy`: the schema merge strategy used for that key.
  - `sources`: an ordered list of one or more contributing layers.

Minimum required source labels:
- `cli_flag`
- `override_env`
- `workspace_patch`
- `global_patch`
- `default`
- `injected_protected`

Rules:
- For `replace` keys, `sources` MUST contain exactly one entry.
- For merge keys (e.g. `concat_dedupe_ordered_set`), `sources` MUST contain every contributing layer, in the order they were applied to build the effective value.
- The explain output MUST be deterministic for the same inputs (no timestamps / random ordering).

Recommended (non-required) fields for each source entry:
- `path`: the resolved file path when a patch file layer contributed.
- `value_excerpt`: a best-effort small representation of what the layer contributed (especially useful for merge keys).

### Explicit schema assignment for world-deps

#### `world.deps.enabled`
- Type: list[string]
- Merge strategy: `concat_dedupe_ordered_set`
- Notes:
  - This is required to match the world-deps contract’s “Enabled list merge” semantics and the operator-visible “still enabled via other scope” messaging.

## Architecture Shape
- Schema registry (config):
  - Extend the config schema model to include a per-key merge strategy field.
- Effective config resolution:
  - Apply merge strategies during `current` resolution (not during patch-file parsing).
- Explain/provenance emission:
  - Emit `merge_strategy` and a list of `sources` per key (not a single source enum) so derived values are explainable.

## Sequencing / Dependencies
- Sequenced with ADR-0008 implementation work:
  - This ADR is an additive refinement to the ADR-0008 contract and should land before (or as part of) implementing `current show --explain` and config patch merging.
- Consumer dependency:
  - World-deps enabled-list semantics depend on having multi-source explainability for a merged key.

## Security / Safety Posture
- Determinism:
  - Explain/provenance output must be stable and deterministic to support debugging, scripting, and traceability.
- Fail-closed on invalid config:
  - Invalid YAML / type mismatches remain actionable user errors (exit `2`) and must not cause file mutations (ADR-0008 posture).

## Validation Plan (Authoritative)
- Unit tests:
  - `replace` resolution produces single-source provenance.
  - `concat_dedupe_ordered_set` produces correct effective output and multi-source provenance ordering.
  - Determinism tests for provenance output (same inputs → identical `--explain` payload).
- Integration tests:
  - Golden outputs for `config current show --explain` covering:
    - only global defined,
    - both global + workspace defined,
    - workspace disabled marker present (workspace ignored),
    - duplicate list values across scopes (dedupe correctness).

## Rollout / Backwards Compatibility
- Greenfield contract refinement prior to implementation; no migrations.

## Decision Summary
- No decision register exists yet for this refinement.
- If execution introduces additional non-trivial choices (e.g. exact `--explain` payload shape, whether policy adopts multi-source provenance in the same release), introduce a decision register under the implementing planning pack and link it here.
