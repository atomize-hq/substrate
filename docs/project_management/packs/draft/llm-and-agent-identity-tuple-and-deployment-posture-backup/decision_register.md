# Decision Register — llm-and-agent-identity-tuple-and-deployment-posture

---

### DR-0001 — Overloaded backend labels vs explicit tuple fields

**Decision owner(s):** Shell + Gateway + Agent Hub maintainers  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

**Problem / Context**

- A single backend label is not enough to explain caller origin, routing authority, upstream
  provider, credential authority, and protocol surface independently.

**Option A — Keep backend labels as the main operator story**

- **Pros:**
  - fewer visible fields
- **Cons:**
  - forces operators to infer multiple meanings from one label
  - drifts into backend-id overload

**Option B — Lock an explicit five-field identity tuple**

- **Pros:**
  - separates origin, routing, fulfillment, authorization, and protocol cleanly
  - gives later policy and agent-hub work one shared vocabulary
- **Cons:**
  - requires more explicit operator-facing documentation

**Recommendation**

- **Selected:** Option B — explicit tuple
- **Rationale:** later routing and agent-orchestration work needs one stable operator vocabulary
  that does not overload `backend_id`.

---

### DR-0002 — Placement posture: two postures plus bridge vs multiple standing routers

**Decision owner(s):** Shell + World + Gateway maintainers  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

**Problem / Context**

- Host execution, world execution, and host-to-world transport need operator-visible language, but
  that language must not create a second standing router or second control plane by accident.

**Option A — Describe host and world paths as separate standing routers**

- **Pros:**
  - superficially simple wording
- **Cons:**
  - conflicts with gateway boundary ownership
  - implies multiple authorities where only one should exist

**Option B — Describe `in_world`, `host_only`, and `host_to_world_bridge` separately**

- **Pros:**
  - keeps execution mode, transport, and router identity separate
  - preserves the single-router interpretation
- **Cons:**
  - requires explicit wording around transport-only bridge behavior

**Recommendation**

- **Selected:** Option B — two postures plus a transport adjunct
- **Rationale:** execution posture and transport reachability must not be mistaken for router
  identity.

---

### DR-0003 — Where tuple semantics live for downstream ADRs

**Decision owner(s):** Planning + Gateway + Agent Hub maintainers  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`

**Problem / Context**

- ADR-0043 and ADR-0044 both depend on tuple terminology. If each one restates tuple meanings, the
  repo gets multiple semantic owners.

**Option A — Let each follow-on ADR restate the tuple locally**

- **Pros:**
  - each ADR appears self-contained
- **Cons:**
  - high drift risk
  - conflicting field definitions become likely

**Option B — Make ADR-0042 plus this pack the semantic source of truth**

- **Pros:**
  - later ADRs can consume one shared contract
  - ownership split stays crisp: semantics here, policy in ADR-0043, agent-hub behavior in ADR-0044
- **Cons:**
  - requires explicit cross-links from follow-on ADRs

**Recommendation**

- **Selected:** Option B — ADR-0042 pack is the semantic source of truth
- **Rationale:** later lanes should build on one tuple contract, not each create their own.
