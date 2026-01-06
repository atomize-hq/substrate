# Decision Register — Policy + Config Precedence (ADR-0005)

### DR-0001 — Implementation strategy for workspace-over-env precedence

**Decision owner(s):** shell/config maintainers  
**Date:** 2026-01-02  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`, `docs/project_management/_archived/policy_and_config_precedence/PCP0-spec.md`

**Problem / Context**
- Effective config precedence must change so workspace config overrides `SUBSTRATE_*` env vars when a workspace exists.
- The implementation must preserve strict parsing and keep the change localized to the effective-config resolver.

**Option A — Field-level layer merge (future-proof layering)**
- **Pros:** Preserves the full precedence stack even if workspace YAML omits keys; supports future schema evolution with partial configs.
- **Cons:** Requires new merge code or a parallel “partial config” type; increases code surface area.
- **Cascading implications:** More test cases and more maintenance burden for every new config key.
- **Risks:** Implementation drift across nested structs and list semantics.
- **Unlocks:** Allows later adoption of `#[serde(default)]` without rewriting layering.
- **Quick wins / low-hanging fruit:** None; this is structural work.

**Option B — Reorder resolver steps without new merge types (minimal change)**
- **Pros:** Minimal code change; keeps resolver logic centralized; matches existing “full config file” model from `substrate workspace init`.
- **Cons:** Does not preserve env as a fallback for missing workspace keys if workspace YAML becomes partial in the future.
- **Cascading implications:** Future “partial config” work would require revisiting layering.
- **Risks:** None for the current strict schema contract.
- **Unlocks:** Fast fix for the operator footgun described in ADR-0005.
- **Quick wins / low-hanging fruit:** Update precedence tests and smoke scripts in the same slice.

**Recommendation**
- **Selected:** Option B — Reorder resolver steps without new merge types (minimal change)
- **Rationale (crisp):** The current strict schema contract requires complete config files, so a minimal resolver reordering is sufficient and keeps the change scoped and low-risk.

**Follow-up tasks (explicit)**
- Implement resolver change: `PCP0-code`
- Update tests: `PCP0-test`
- Validate and reconcile: `PCP0-integ`

