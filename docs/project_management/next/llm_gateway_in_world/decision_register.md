# Decision Register — llm_gateway_in_world

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — In-world bind/transport: UDS proxy vs loopback TCP inside world

**Decision owner(s):** Shell + World-agent + Gateway maintainers  
**Date:** 2026-02-08  
**Status:** Draft  
**Related docs:** ADR-0023, ADR-0027

**Problem / Context**
- The gateway must run “in world” by default, but host-side CLIs and tools still need a stable way to reach it.

**Option A — World-agent proxies gateway over existing transport (recommended)**
- **Pros:** No new host listener; avoids “host gateway” confusion; single audited transport path.
- **Cons:** Requires HTTP proxying support in world-agent/shell plumbing; more moving parts.

**Option B — Host loopback forwarder (host listens on 127.0.0.1 and forwards into world)**
- **Pros:** Simple client compatibility; easy for tools expecting `http://127.0.0.1:<port>`.
- **Cons:** Must be documented carefully as transport-only; risk of being mistaken for a host-level egress gateway.

**Recommendation**
- **Selected:** TBD.
- **Rationale (crisp):** TBD.

---

### DR-0002 — Default request/response logging: metadata-only vs body logging

**Decision owner(s):** Gateway maintainers + Security  
**Date:** 2026-02-08  
**Status:** Draft  

**Problem / Context**
- LLM requests can contain secrets. Default logging must be safe while still supporting debugging.

**Option A — Metadata-only logs (default)**
- **Pros:** Lowest risk; aligns with “no secrets in logs” posture; easier to ship by default.
- **Cons:** Harder to debug prompt/response issues.

**Option B — Body logging available via explicit opt-in**
- **Pros:** Better debug; supports reproducing routing/translation bugs.
- **Cons:** Higher risk; requires rigorous redaction + caps + operator warnings.

**Recommendation**
- **Selected:** TBD.

---

### DR-0003 — Policy gate location: broker-first vs gateway-local checks

**Decision owner(s):** Broker + Gateway maintainers  
**Date:** 2026-02-08  
**Status:** Draft  

**Problem / Context**
- We need deterministic allow/deny behavior with `--explain` provenance while minimizing duplicated policy logic.

**Option A — Broker-first policy evaluation**
- **Pros:** Centralizes policy reasoning; consistent explain/provenance story; fewer policy implementations.
- **Cons:** Gateway must call into broker (or consume a snapshot) on the hot path.

**Option B — Gateway-local checks against an embedded snapshot**
- **Pros:** Fast and self-contained; simpler in-world deployment.
- **Cons:** Snapshot lifecycle and provenance must be carefully specified to avoid drift.

**Recommendation**
- **Selected:** TBD.

