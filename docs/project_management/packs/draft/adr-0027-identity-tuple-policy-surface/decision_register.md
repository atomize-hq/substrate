# Decision Register — adr-0027-identity-tuple-policy-surface

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:

- This decision register supports `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-ITPS-01 — Tuple-policy publication family (`identity_tuple` reuse vs trace-only tuple shape)

**Decision owner(s):** Shell + Broker + Trace maintainers  
**Date:** 2026-04-23  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`, `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Tuple-aware policy denials and status surfaces need one stable publication family. A second trace-only tuple object would duplicate semantics that ADR-0042 already names.

**Option A — Reuse `identity_tuple` and `placement_posture` across status, diagnostics, and trace**

- **Pros:**
  - Keeps one tuple vocabulary across status, deny explanations, and trace records.
  - Preserves ADR-0042 as the semantic owner and ADR-0028 as the trace-envelope owner.
  - Avoids translation logic between status objects and trace-only objects.
- **Cons:**
  - Requires later telemetry work to extend existing publication surfaces instead of inventing a fresh trace-local schema.
- **Cascading implications:**
  - `telemetry-spec.md` must define additive publication around the existing object names.
  - Human-readable diagnostics must use the same tuple labels as status and trace.
- **Risks:**
  - Later slices must not smuggle backend-id semantics into tuple fields.
- **Unlocks:**
  - One deterministic vocabulary for policy denial, status output, and trace publication.
- **Quick wins / low-hanging fruit:**
  - Reuse current gateway status object names without a migration step.

**Option B — Add a second trace-only tuple schema**

- **Pros:**
  - Gives trace writers a format they can tailor without touching existing status objects.
- **Cons:**
  - Duplicates tuple semantics under two names.
  - Forces documentation and tests to keep two publication shapes aligned.
- **Cascading implications:**
  - `telemetry-spec.md`, `docs/TRACE.md`, and gateway status docs would need a translation contract.
- **Risks:**
  - Drift between status publication and trace publication.
- **Unlocks:**
  - No unique unlock that Option A does not already provide.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Reuse `identity_tuple` and `placement_posture` across status, diagnostics, and trace.
- **Rationale (crisp):** One tuple vocabulary preserves semantic ownership and prevents schema drift across deny, status, and trace surfaces.

**Follow-up tasks (explicit)**

- `ITPS2` must publish tuple-aware telemetry using `identity_tuple` and `placement_posture` instead of inventing a trace-only object.
- `ITPS2` must keep `backend_id` as a separate selector and correlation field.

---

### DR-ITPS-02 — Authoritative inspection surface for `llm.constraints.*` (`policy current show --explain` vs config view)

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-04-23  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`, `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

**Problem / Context**

- The draft ADR and pre-planning inputs mixed config and policy inspection language for `llm.constraints.*`. Operators need one authoritative merged view for tuple-policy constraints.

**Option A — `substrate policy current show --explain` is the authoritative merged inspection surface**

- **Pros:**
  - Matches the existing effective-policy command and patch-layer explanation path.
  - Keeps tuple-axis keys on the policy ladder where they already live.
  - Prevents config and policy from presenting conflicting effective values.
- **Cons:**
  - Draft ADR text that mentions config explain surfaces must be corrected.
- **Cascading implications:**
  - `contract.md`, `policy-spec.md`, and later manual validation docs must treat the policy view as authoritative for `llm.constraints.*`.
  - `substrate config ...` remains the config-root inspection family and must not claim tuple-policy ownership.
- **Risks:**
  - None beyond documentation cleanup.
- **Unlocks:**
  - One deterministic operator story for tuple-policy provenance.
- **Quick wins / low-hanging fruit:**
  - Reuse the existing default policy-patch headers and explain plumbing already present in the shell.

**Option B — Treat `substrate config show --explain` and `substrate policy current show --explain` as interchangeable**

- **Pros:**
  - Gives operators two places to look.
- **Cons:**
  - Creates two incompatible ownership stories for the same keys.
  - Encourages stale examples and contradictory acceptance criteria.
- **Cascading implications:**
  - Every doc and test would need to explain why the same policy key appears authoritative on two roots.
- **Risks:**
  - Operator confusion and future drift between config and policy docs.
- **Unlocks:**
  - No unique unlock.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — `substrate policy current show --explain` is the authoritative merged inspection surface.
- **Rationale (crisp):** Tuple-axis constraints are policy keys, and the policy effective view is already the merged explain surface that carries their provenance.

**Follow-up tasks (explicit)**

- `ITPS1` must define policy-deny explanations against policy-key ownership, not config-key ownership.
- A later documentation writer must remove the stale config-view assignment from the ADR and the pre-planning manifest.
