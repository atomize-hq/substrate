# world-disabled-diagnostics — decision register

This file records the A/B decisions required to make ADR-0036 deterministic and testable.

## Inputs (non-authoritative links)
- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
- Minimal spec draft (pre-planning): `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md`
- Impact map (pre-planning): `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`

---

### DR-0001 — JSON field paths + enum spellings for world/world-deps statuses

**Decision owner(s):** WDD-PWS-contract (contract)  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`  
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`  

**Problem / Context**
- ADR-0036 requires additive, stable status enums for world + world-deps for both `substrate shim doctor --json` and `substrate health --json`.
- Existing payloads do not have a stable “disabled/skipped” classifier; prior behavior can mislead operators and tooling.

**Option A — snake_case enums at `.world.status` / `.world_deps.status` (single source of truth)**
- **Pros:** matches existing JSON naming style (`missing_prereqs`, `not_provisioned`, etc); easy to test; stable tokens for scripts; minimal schema churn.
- **Cons:** requires updating report structs + rendering logic to consume status enums instead of legacy booleans/strings alone.
- **Cascading implications:** health summary and shim doctor text rendering MUST branch on the new status enums.
- **Risks:** older consumers reading only legacy booleans/strings may still misclassify; mitigated by documenting status fields as canonical.
- **Unlocks:** deterministic “disabled vs broken” classification; enables future attribution work to layer without changing semantics.
- **Quick wins / low-hanging fruit:** add fields without removing anything; update tests to assert enums.

**Option B — kebab-case enums (e.g., `needs-attention`, `skipped-disabled`)**
- **Pros:** closer to some human-friendly phrasing in planning docs.
- **Cons:** inconsistent with existing Substrate JSON enum style; more annoying to work with in some shells/scripts; increased drift risk across docs/code.
- **Cascading implications:** update tests/docs to match non-standard enum style.
- **Risks:** accidental mixed spelling (`needs-attention` vs `needs_attention`) across surfaces.
- **Unlocks:** none beyond Option A.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — snake_case enums at `.world.status` / `.world_deps.status`
- **Rationale (crisp):** Minimizes drift with existing JSON conventions and keeps one canonical classifier for both text + JSON behavior.

**Follow-up tasks (explicit)**
- Implement additive fields:
  - `substrate shim doctor --json`: add `.world.status` and `.world_deps.status` with these spellings:
    - world: `healthy | needs_attention | disabled | unknown`
    - world_deps: `ok | error | skipped_disabled | unknown`
  - `substrate health --json`: statuses are available under `.shim.world.status` / `.shim.world_deps.status` (embedded shim doctor payload).
- Update `WDD0` implementation + tests to assert these field paths and spellings.
- Update operator docs (`docs/USAGE.md`) to describe the new enum fields as the stable machine-readable contract.

---

### DR-0002 — Legacy JSON error-field behavior when disabled/skipped applies

**Decision owner(s):** WDD-PWS-contract (contract)  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`  
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`  

**Problem / Context**
- ADR-0036 requires “skipped because disabled” to be machine-detectable via status enums.
- Existing payloads use legacy error strings for snapshot failures; if we encode “skipped” as an error string, health can misclassify disabled as broken.

**Option A — omit legacy error fields for disabled/skipped (treat as non-error)**
- **Pros:** avoids false-negative “unavailable/error” interpretations; clean separation between “skipped” vs “failed”; simpler health failure aggregation.
- **Cons:** less inline explanation in JSON when disabled (explanation is owned by operator text + future attribution work).
- **Cascading implications:** text output must remain explicit/actionable since JSON will not carry an error message for disabled/skipped.
- **Risks:** some tooling expects an `error` field to exist for every non-OK state; mitigated by documenting status enums as canonical.
- **Unlocks:** deterministic “disabled is quiet” posture across all platforms.
- **Quick wins / low-hanging fruit:** use existing `skip_serializing_if` behavior to omit fields when set to `None`.

**Option B — keep legacy error fields populated even when disabled/skipped**
- **Pros:** provides immediate “why” context without reading text output.
- **Cons:** encourages consumers to treat disabled as an error; risks reintroducing noisy “unavailable” summaries.
- **Cascading implications:** requires special-casing across health summary and any consumers to ignore error strings when status indicates disabled/skipped.
- **Risks:** high drift risk; duplicated conditional logic across commands and tests.
- **Unlocks:** none beyond convenience.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — omit legacy error fields for disabled/skipped
- **Rationale (crisp):** Keeps disabled/skipped as a non-error state and prevents accidental “unavailable/error” regressions.

**Follow-up tasks (explicit)**
- When effective `world.enabled=false`:
  - `shim doctor --json` MUST omit:
    - `.world.error`, `.world.stderr`, `.world.exit_code`, `.world.details`
    - `.world_deps.error`, `.world_deps.report`
  - `health --json` MUST omit:
    - `.summary.world_error`, `.summary.world_deps_error`
    - and MUST set `.summary.world_ok=null` (no probe performed).
- Update `WDD0` tests to ensure disabled/skipped does not populate legacy error fields.

---

### DR-0003 — Operator-facing copy standardization for disabled/skipped

**Decision owner(s):** WDD-PWS-contract (contract)  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`  

**Problem / Context**
- ADR-0036 explicitly allows copy variation, but tests and operator expectations require a deterministic, cross-command contract.
- Disabled-mode guidance must be explicit/actionable, and must not imply backend breakage.

**Option A — exact line templates for disabled/skipped (recommended)**
- **Pros:** strongest determinism; stable tests; consistent UX across `health` and `shim doctor`.
- **Cons:** copy changes become contract changes; requires deliberate coordination with docs/tests.
- **Cascading implications:** implementation must update renderers to match the templates exactly; docs must use the same phrasing.
- **Risks:** minor formatting changes can break tests if they assert full lines; mitigated by limiting templates to a small set of lines.
- **Unlocks:** reliable machine+human parity: text output and JSON enums align without ambiguity.
- **Quick wins / low-hanging fruit:** specify a small required subset of lines and keep everything else flexible.

**Option B — required substrings + ordering rules**
- **Pros:** less brittle; easier to iterate on prose.
- **Cons:** harder to make tests unambiguous; increased drift across commands/platforms.
- **Cascading implications:** more complex test assertions (substring/order matching).
- **Risks:** copy divergence over time (“disabled” vs “off” vs “skipped”) despite tests passing.
- **Unlocks:** faster iteration on messaging.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — exact line templates (small required subset)
- **Rationale (crisp):** Deterministic, cross-platform operator messaging is the point of the feature; brittle copy is avoided by scoping templates to a few lines.

**Follow-up tasks (explicit)**
- Update `contract.md` to define the required exact lines for:
  - `substrate health` (disabled world) and `substrate shim doctor` (disabled world)
  - and to define what MUST NOT appear (world-deps remediation when disabled).
- Update `WDD0` tests to assert the required lines (and absence constraints) in text output.

