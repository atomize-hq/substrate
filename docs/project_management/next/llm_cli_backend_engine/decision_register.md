# Decision Register — llm_cli_backend_engine

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — CLI session strategy: persistent vs per-request

**Decision owner(s):** Shell + Engine maintainers  
**Date:** 2026-02-08  
**Status:** Draft  
**Related docs:** ADR-0024, ADR-0027

**Problem / Context**
- CLIs vary: some are fast to spawn, others are expensive; some support structured streaming, others do not.

**Option A — Persistent session by default**
- **Pros:** Lower latency for repeated requests; more feasible for streaming; amortizes auth/setup.
- **Cons:** Harder lifecycle management; requires robust isolation and cleanup; more state to audit.

**Option B — Per-request spawn by default**
- **Pros:** Simpler; fewer long-lived processes; easier to fail closed.
- **Cons:** Higher latency; streaming may be worse or unavailable.

**Recommendation**
- **Selected:** TBD.

---

### DR-0002 — Streaming fallback when CLI lacks streaming: buffer+rechunk vs non-stream

**Decision owner(s):** Engine maintainers  
**Date:** 2026-02-08  
**Status:** Draft  

**Problem / Context**
- The gateway exposes streaming dialects, but a CLI backend may only produce a final output blob.

**Option A — Buffer then re-chunk into a synthetic stream**
- **Pros:** Preserves a streaming surface for clients expecting SSE/chunked output.
- **Cons:** Can mislead clients about latency; must be clearly labeled in trace/event metadata.

**Option B — Return non-streaming response when backend lacks streaming**
- **Pros:** Honest semantics; simpler.
- **Cons:** Breaks some clients and reduces compatibility.

**Recommendation**
- **Selected:** TBD.

---

### DR-0003 — CLI prompt contract: JSON envelope vs plain text template

**Decision owner(s):** Engine + Agent adapters maintainers  
**Date:** 2026-02-08  
**Status:** Draft  

**Problem / Context**
- We need a deterministic, testable transformation from canonical request to CLI invocation.

**Option A — JSON envelope (structured)**
- **Pros:** More testable; explicit fields; easier to version; easier to redact/cap.
- **Cons:** Some CLIs may not accept structured input; requires adapter translation.

**Option B — Plain text template**
- **Pros:** Universal; easiest to send to any CLI.
- **Cons:** Harder to evolve; brittle parsing; more prompt-injection surface.

**Recommendation**
- **Selected:** TBD.

