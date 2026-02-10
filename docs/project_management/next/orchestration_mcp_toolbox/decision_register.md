# Decision Register — orchestration_mcp_toolbox

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Bind transport for the internal MCP server (UDS-first vs TCP-only)

**Decision owner(s):** Shell + Agent Hub + World maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

**Problem / Context**
- The internal MCP server must be reachable by the orchestrator agent, but must not be exposed publicly by default.
- Transport must work in-world across Linux/macOS/WSL and have a clear “local-only” story.

**Option A — UDS-first (preferred), with loopback TCP fallback only where UDS is not available**
- **Pros:**
  - Strong “local-only” posture by default (filesystem permissions on the socket).
  - Works naturally inside Linux worlds (Linux host, Lima guest, WSL).
  - Avoids port management and accidental exposure.
- **Cons:**
  - Requires defining a fallback for environments that cannot use UDS for the orchestrator scope.
- **Cascading implications:**
  - `substrate mcp env` must emit an endpoint string that can represent both `unix://` and `tcp://`.
  - When falling back to TCP, bind MUST be loopback-only and randomized port by default.
- **Risks:**
  - TCP fallback can be misconfigured; mitigated by strict defaults and explicit `--explain`/status output.
- **Unlocks:**
  - Minimal exposure surface while keeping tool access usable in-world.
- **Quick wins / low-hanging fruit:**
  - Start with UDS inside world; add host TCP fallback only if needed for specific clients.

**Option B — TCP-only everywhere (loopback)**
- **Pros:**
  - Uniform transport and easier client compatibility.
- **Cons:**
  - Larger accidental-exposure surface; port management complexity.
  - Harder to express “permissioned local-only” without additional auth.
- **Cascading implications:**
  - Requires stronger auth story to compensate for the broader listener surface.
- **Risks:**
  - Confusion about whether this is a “public” service; misconfiguration can widen access.
- **Unlocks:**
  - Some clients may be simpler to configure.
- **Quick wins / low-hanging fruit:**
  - Fastest to prototype, but least aligned with Substrate posture.

**Recommendation**
- **Selected:** Option A — UDS-first with loopback TCP fallback only where needed.
- **Rationale (crisp):** Keeps the default exposure surface minimal and aligns with “internal-only” semantics while still allowing a controlled fallback path.

**Follow-up tasks (explicit)**
- Define the endpoint formats emitted by `substrate mcp env` for UDS and TCP.
- Define the default filesystem permissions for the UDS socket and the default bind rules for TCP.

---

### DR-0002 — MCP server lifecycle (per-orchestration-session vs global singleton)

**Decision owner(s):** Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Tool authorization depends on the orchestrator identity and its session context (`orchestration_session_id`, `world_id`).
- A global singleton MCP server increases the risk of cross-session confusion and stale authorization context.

**Option A — Per-orchestration-session MCP server (recommended)**
- **Pros:**
  - Session context is explicit and naturally bound to authorization and attribution.
  - Simplifies “tools inherit the orchestrator’s boundary” semantics.
  - Easier to ensure local-only endpoints are scoped and cleaned up.
- **Cons:**
  - More server lifecycles to manage (start/stop).
- **Cascading implications:**
  - `substrate mcp status` must report per-session instances.
  - The hub must define cleanup rules when sessions end or crash.
- **Risks:**
  - Endpoint churn; mitigated by stable env emission per session and good diagnostics.
- **Unlocks:**
  - Clean multi-session concurrency without a shared mutable tool context.
- **Quick wins / low-hanging fruit:**
  - Tie server lifecycle to the orchestrator session lifecycle.

**Option B — One global MCP server for all sessions**
- **Pros:**
  - Fewer processes; simpler operationally.
- **Cons:**
  - Requires multiplexing session identities and authorization across calls.
  - Higher risk of “wrong session” confusion and misattribution.
- **Cascading implications:**
  - Must define a “session selection” mechanism in every tool call (more surface area).
- **Risks:**
  - Security footguns due to incorrect session targeting.
- **Unlocks:**
  - Slightly simpler to keep always-on.
- **Quick wins / low-hanging fruit:**
  - Prototype speed only; not a good long-term contract.

**Recommendation**
- **Selected:** Option A — Per-orchestration-session MCP server.
- **Rationale (crisp):** Avoids cross-session authorization confusion and makes attribution deterministic by construction.

**Follow-up tasks (explicit)**
- Define deterministic endpoint naming/paths for per-session servers and cleanup rules on crash.

---

### DR-0003 — Tool namespace + versioning strategy (server-level version vs tool-name version)

**Decision owner(s):** MCP toolbox maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

**Problem / Context**
- Orchestrator agents need stable tool names; we also need a way to evolve schemas.

**Option A — Stable tool names under `substrate.*` with a server-level `toolbox_version=1`**
- **Pros:**
  - Tool names remain stable and readable.
  - Versioning is centralized; easier to roll forward as a set.
- **Cons:**
  - Requires clients to surface/record toolbox version for debugging.
- **Cascading implications:**
  - `substrate mcp status --json` must expose `toolbox_version`.
  - Trace events for tool calls must record the toolbox version.
- **Risks:**
  - Schema evolution must be disciplined; mitigated by additive-only policy until a major bump.
- **Unlocks:**
  - Cleaner client configs (`substrate.list_agents` vs `substrate.v1.list_agents`).
- **Quick wins / low-hanging fruit:**
  - Start at version `1` and keep schemas additive for v1.

**Option B — Encode version in tool names (`substrate.v1.*`)**
- **Pros:**
  - Multiple versions can coexist.
- **Cons:**
  - Noisy tool names; encourages drift and duplication.
  - Harder for operators to know “which version is active”.
- **Cascading implications:**
  - Requires naming policy and deprecation strategy per tool.
- **Risks:**
  - Version sprawl and confusing client configurations.
- **Unlocks:**
  - Parallel version rollouts (not needed for v1).
- **Quick wins / low-hanging fruit:**
  - None worth the naming cost.

**Recommendation**
- **Selected:** Option A — Stable tool names under `substrate.*` with server-level `toolbox_version=1`.
- **Rationale (crisp):** Keeps client configuration simple and makes the toolbox a cohesive, versioned unit.

**Follow-up tasks (explicit)**
- Define the `toolbox_version` field in `substrate mcp status --json`.
- Record `toolbox_version` on every tool-call trace event.

---

### DR-0004 — Authorization enforcement location (agent-hub role gate vs per-tool ad-hoc checks)

**Decision owner(s):** Agent Hub + MCP toolbox maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Executor agents must not have access to orchestration-only tools.
- Authorization must be deterministic and auditable.

**Option A — Central role gate in the MCP server (deny if not orchestrator), plus per-tool policy checks**
- **Pros:**
  - Single, consistent enforcement point for role-based access control.
  - Clear audit trail: one deny reason for “wrong role”.
  - Per-tool checks remain for data sensitivity (redaction, policy overlays).
- **Cons:**
  - Requires the MCP server to know the caller identity and role reliably.
- **Cascading implications:**
  - The caller identity and role must be explicit in the connection context (not inferred heuristically).
- **Risks:**
  - If identity propagation is wrong, access control breaks; mitigated by strict wiring and tests.
- **Unlocks:**
  - Safe exposure of orchestration tools to multiple possible orchestrator backends.
- **Quick wins / low-hanging fruit:**
  - Implement a single “role gate” middleware layer.

**Option B — Per-tool ad-hoc checks only (no central gate)**
- **Pros:**
  - Less shared infrastructure.
- **Cons:**
  - Easy to miss a check; inconsistent deny semantics; harder to audit.
- **Cascading implications:**
  - Every tool must replicate the same authorization logic.
- **Risks:**
  - Security bugs via missing checks.
- **Unlocks:**
  - None desirable.
- **Quick wins / low-hanging fruit:**
  - Prototype speed, but not acceptable as a contract.

**Recommendation**
- **Selected:** Option A — Central role gate in the MCP server, plus per-tool policy checks.
- **Rationale (crisp):** Role gating is a security boundary; centralizing it prevents inconsistent enforcement.

**Follow-up tasks (explicit)**
- Define and test caller identity propagation so the MCP server can enforce role gates deterministically.

---

### DR-0005 — Tool execution boundary inheritance (tools run in orchestrator scope vs always in-world)

**Decision owner(s):** MCP toolbox + World maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Some tools are read-only queries (agent inventory, policy view). Others may perform actions (cancel session, request execution).
- The toolbox must not silently change the enforcement boundary relative to the orchestrator’s configured scope.

**Option A — Tools inherit the orchestrator’s execution boundary (recommended)**
- **Pros:**
  - Matches operator expectations: “orchestrator scope governs its tools”.
  - Avoids silent boundary changes (no unexpected host execution).
  - Cleanly supports both host-only and in-world orchestrators (where policy allows).
- **Cons:**
  - Some tools may require extra plumbing when orchestrator is in-world (e.g., access to host-only state).
- **Cascading implications:**
  - If a tool requires host-only access and the orchestrator is in-world, the tool must:
    - either be explicitly unsupported in that posture (fail closed with exit code `4` / not supported), or
    - be implemented via an auditable brokered call path (future).
- **Risks:**
  - Feature gaps for in-world orchestrators in v1; acceptable if clearly documented and fail-closed.
- **Unlocks:**
  - Deterministic security posture with clear failure modes.
- **Quick wins / low-hanging fruit:**
  - Start with read-only tools that are scope-agnostic.

**Option B — All tools always execute in-world**
- **Pros:**
  - Strong enforcement boundary for side-effectful operations.
- **Cons:**
  - Requires moving or proxying host-owned state into the world, risking drift and complexity.
  - Makes host-only setups harder.
- **Cascading implications:**
  - Requires a new host<->world state proxy contract for policy/trace access.
- **Risks:**
  - High complexity and correctness risk for v1.
- **Unlocks:**
  - Strong in-world posture for all tools (but at high cost).
- **Quick wins / low-hanging fruit:**
  - None without a broader proxy architecture.

**Recommendation**
- **Selected:** Option A — Tools inherit the orchestrator’s execution boundary.
- **Rationale (crisp):** Prevents silent boundary changes and keeps the security posture aligned with the orchestrator’s configured scope.

**Follow-up tasks (explicit)**
- Enumerate which tools are scope-agnostic in v1 and which are explicitly unsupported for in-world orchestrators (with fail-closed errors).

