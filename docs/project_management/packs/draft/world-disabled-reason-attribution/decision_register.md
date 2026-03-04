# Decision Register — world-disabled-reason-attribution

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:

- This decision register supports `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Disable-attribution contract source (ADR reuse vs replay-local taxonomy)

**Decision owner(s):** Shell + Replay maintainers  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`

**Problem / Context**

- Replay must attribute *why* isolation is disabled (`world.enabled=false`) without creating a second, inconsistent taxonomy for the same concepts (tokens, precedence, field names).

**Option A — Define replay-local tokens/precedence for world-disable attribution**

- **Pros:** replay can tune strings/ordering without waiting on ADR changes.
- **Cons:** creates contract drift vs ADR-0037; increases misattribution risk; forces docs and tooling to reconcile multiple “disable reason” systems.

**Option B — Reuse ADR-0037 tokens/precedence verbatim**

- **Pros:** single source of truth for disable attribution; reduces drift; easier to test and document.
- **Cons:** replay has less freedom to rename/reorder.

**Recommendation**

- **Selected:** Option B — Reuse ADR-0037 tokens/precedence verbatim.
- **Rationale (crisp):** the feature is attribution only; reusing ADR-0037 avoids taxonomy drift and makes cross-surface consistency mechanically testable.

**Follow-up tasks (explicit)**

- WDRA0: enforce ADR-0037 precedence and verbatim strings in both stderr and telemetry.

---

### DR-0002 — Telemetry strategy for world-disable attribution (new fields vs overloading existing fields)

**Decision owner(s):** Replay + Trace maintainers  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`, `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`

**Problem / Context**

- `replay_strategy` already has origin selection fields; world-disable attribution must be additive-only, structured, and safe by default (no secret/path leaks).

**Option A — Overload existing origin selection fields (`origin_reason(_code)`) with world-disable provenance**

- **Pros:** no new schema fields.
- **Cons:** conflates replay-local origin selection with effective-config provenance; harder to reason about omission/null semantics; higher long-term drift risk.

**Option B — Add optional `world_disable_reason` / `world_disable_source` fields (ADR-0037 schema)**

- **Pros:** preserves semantic separation; additive-only; enables strict redaction/absence semantics; aligns to ADR-0037 exactly.
- **Cons:** introduces new optional fields that require doc/test updates.

**Recommendation**

- **Selected:** Option B — Add `world_disable_reason` / `world_disable_source` (optional; omit when not applicable).
- **Rationale (crisp):** structured attribution belongs in structured fields; origin selection fields remain replay-local and stable.

**Follow-up tasks (explicit)**

- WDRA0: implement emission + omission semantics and redaction guarantees per `telemetry-spec.md`.

---

### DR-0003 — CI checkpoint cadence (single checkpoint vs multi-checkpoint)

**Decision owner(s):** Shell + Replay maintainers  
**Date:** 2026-03-04  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`

**Problem / Context**

- The draft slice skeleton contains a single slice (`WDRA0`), but the cross-platform surfaces touched by replay attribution make regressions expensive.

**Option A — Add an earlier checkpoint before completing WDRA0**

- **Pros:** earlier CI signal if the implementation splits into multiple slices later.
- **Cons:** extra operational overhead with no incremental slice boundary today.

**Option B — Single end-of-feature checkpoint after WDRA0 (`CP1`)**

- **Pros:** aligns to the single-slice skeleton; minimizes overhead; still gates the full cross-platform seam.
- **Cons:** no intermediate checkpoint signal.

**Recommendation**

- **Selected:** Option B — Single end-of-feature checkpoint after WDRA0 (`CP1`).
- **Rationale (crisp):** one slice means one coherent cross-platform seam; checkpointing at the seam is sufficient.

**Follow-up tasks (explicit)**

- Wire `CP1-ci-checkpoint` to depend on `WDRA0-integ-core` once schema v4 task graph is enabled.
