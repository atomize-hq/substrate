# world-disabled-reason-attribution — decision register

This file records the fine-grained decisions required to make ADR-0038 deterministic and testable.

## Inputs
- ADR: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- Prerequisite ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Contract: `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- Telemetry: `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`

---

### DR-0001 — shared classifier helper vs replay-local duplication

**Decision owner(s):** WDRA-PWS-contract  
**Date:** 2026-03-31  
**Status:** Accepted

**Problem / context**
- Replay needs the same winning-layer explanation for `world.enabled=false` that doctor and health use.
- Duplicating precedence and redaction logic in replay raises drift risk.

**Option A — reuse a shared ADR-0037 classifier helper**
- **Pros:** one winning-layer mapping, one tokenization rule, one redaction seam, one test seam.
- **Cons:** helper extraction touches replay and config-explain seams together.
- **Risks:** helper placement needs a narrow API so replay does not inherit unrelated doctor or health rendering logic.

**Option B — duplicate the attribution logic inside replay routing**
- **Pros:** replay code path stays local.
- **Cons:** precedence drift and redaction drift become likely.
- **Risks:** replay and doctor/health can disagree on the winning layer.

**Recommendation**
- **Selected:** Option A.
- **Rationale:** shared winning-layer semantics are the main point of this ADR, so the precedence and tokenization logic stays single-source.

**Follow-up tasks**
- Expose a replay-safe helper that returns the winning layer, tokenized display path, fixed env token when allowed, and unknown-source fallback.

---

### DR-0002 — telemetry shape for effective-disable attribution

**Decision owner(s):** WDRA-PWS-contract  
**Date:** 2026-03-31  
**Status:** Accepted

**Problem / context**
- Replay already emits `origin_reason` and `origin_reason_code`.
- Effective-disable attribution needs structured non-secret provenance for `replay_strategy`.

**Option A — extend `origin_reason_code` and add `world_disable_source` object**
- **Pros:** keeps existing fields stable, adds structured provenance, and preserves human copy in `origin_reason`.
- **Cons:** adds one new top-level object to the trace event.
- **Risks:** docs must stay aligned across `telemetry-spec.md` and `docs/TRACE.md`.

**Option B — put all new information into free-form `origin_reason` only**
- **Pros:** smallest implementation surface.
- **Cons:** trace consumers must parse human text.
- **Risks:** redaction regressions and brittle downstream tooling.

**Recommendation**
- **Selected:** Option A.
- **Rationale:** additive structured telemetry keeps the trace contract machine-readable without breaking the current replay event shape.

**Follow-up tasks**
- Extend `origin_reason_code` with `world_disabled_override_env`, `world_disabled_workspace_patch`, `world_disabled_global_patch`, and `world_disabled_unknown`.
- Emit `world_disable_source` only for those reason codes.

---

### DR-0003 — formatting rule for the recorded-host case

**Decision owner(s):** WDRA-PWS-contract  
**Date:** 2026-03-31  
**Status:** Accepted

**Problem / context**
- Replay already prints `[replay] origin: host (recorded)` when recorded and target origins are both host.
- ADR-0038 needs the disable reason without adding a second origin line.

**Option A — append the reason after `recorded` in the same origin line**
- **Pros:** minimal surface change, easy to scan, no new replay line type.
- **Cons:** exact punctuation needs a single lock.
- **Risks:** docs and tests must match the punctuation exactly.

**Option B — replace the recorded-host line with a different sentence**
- **Pros:** shorter line.
- **Cons:** breaks the existing `recorded` cue and raises more doc churn.
- **Risks:** downstream expectations drift.

**Recommendation**
- **Selected:** Option A.
- **Rationale:** keep the existing `recorded` cue and append the reason fragment as `host (recorded; <reason>)`.

**Follow-up tasks**
- Keep the recorded-host format exact in `contract.md`, slice specs, tests, and `docs/REPLAY.md`.
