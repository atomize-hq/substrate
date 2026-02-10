# Decision Register — orchestration_mcp_toolbox

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Bind transport for the toolbox server endpoint (UDS-first vs TCP-only)

**Decision owner(s):** Shell + Agent Hub + World maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

**Problem / Context**
- The internal toolbox server (MCP protocol) must be reachable by the orchestrator agent, but must not be exposed publicly by default.
- Transport must work in-world across Linux/macOS/WSL and have a clear “local-only” story.

**Option A — UDS-first (preferred), with loopback TCP fallback only where UDS is not available**
- **Pros:**
  - Strong “local-only” posture by default (filesystem permissions on the socket).
  - Works naturally inside Linux worlds (Linux host, Lima guest, WSL).
  - Avoids port management and accidental exposure.
- **Cons:**
  - Requires defining a fallback for environments that cannot use UDS for the orchestrator scope.
- **Cascading implications:**
  - `substrate agent toolbox env` must emit an endpoint string that can represent both `unix://` and `tcp://`.
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
- Define the endpoint formats emitted by `substrate agent toolbox env` for UDS and TCP.
- Define the default filesystem permissions for the UDS socket and the default bind rules for TCP.

---

### DR-0002 — Toolbox server lifecycle (per-orchestration-session vs global singleton)

**Decision owner(s):** Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Tool authorization depends on the orchestrator identity and its session context (`orchestration_session_id`, `world_id`).
- A global singleton toolbox server increases the risk of cross-session confusion and stale authorization context.

**Option A — Per-orchestration-session toolbox server (recommended)**
- **Pros:**
  - Session context is explicit and naturally bound to authorization and attribution.
  - Simplifies “tools inherit the orchestrator’s boundary” semantics.
  - Easier to ensure local-only endpoints are scoped and cleaned up.
- **Cons:**
  - More server lifecycles to manage (start/stop).
- **Cascading implications:**
  - `substrate agent toolbox status` must report per-session instances.
  - The hub must define cleanup rules when sessions end or crash.
- **Risks:**
  - Endpoint churn; mitigated by stable env emission per session and good diagnostics.
- **Unlocks:**
  - Clean multi-session concurrency without a shared mutable tool context.
- **Quick wins / low-hanging fruit:**
  - Tie server lifecycle to the orchestrator session lifecycle.

**Option B — One global toolbox server for all sessions**
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
- **Selected:** Option A — Per-orchestration-session toolbox server.
- **Rationale (crisp):** Avoids cross-session authorization confusion and makes attribution deterministic by construction.

**Follow-up tasks (explicit)**
- Define deterministic endpoint naming/paths for per-session servers and cleanup rules on crash.

---

### DR-0003 — Tool namespace + versioning strategy (server-level version vs tool-name version)

**Decision owner(s):** Agent toolbox maintainers  
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
  - `substrate agent toolbox status --json` must expose `toolbox_version`.
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
- Define the `toolbox_version` field in `substrate agent toolbox status --json`.
- Record `toolbox_version` on every tool-call trace event.

---

### DR-0004 — Authorization enforcement location (agent-hub role gate vs per-tool ad-hoc checks)

**Decision owner(s):** Agent Hub + Agent toolbox maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Executor agents must not have access to orchestration-only tools.
- Authorization must be deterministic and auditable.

**Option A — Central role gate in the toolbox server (deny if not orchestrator), plus per-tool policy checks**
- **Pros:**
  - Single, consistent enforcement point for role-based access control.
  - Clear audit trail: one deny reason for “wrong role”.
  - Per-tool checks remain for data sensitivity (redaction, policy overlays).
- **Cons:**
  - Requires the toolbox server to know the caller identity and role reliably.
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
- **Selected:** Option A — Central role gate in the toolbox server, plus per-tool policy checks.
- **Rationale (crisp):** Role gating is a security boundary; centralizing it prevents inconsistent enforcement.

**Follow-up tasks (explicit)**
- Define and test caller identity propagation so the toolbox server can enforce role gates deterministically.

---

### DR-0005 — Tool execution boundary inheritance (tools run in orchestrator scope vs always in-world)

**Decision owner(s):** Agent toolbox + World maintainers  
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

---

### DR-0006 — CLI namespace for internal orchestration tools (`mcp` vs `agent toolbox`)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Substrate will likely support external MCP servers in the future, where `substrate mcp …` is the natural namespace.
- This ADR focuses on an *internal* orchestration toolbox used by the agent hub’s orchestrator; reusing `mcp` risks confusing “external server management” with “internal privileged tool surface”.

**Option A — Use `substrate agent toolbox …` for the internal toolbox (recommended)**
- **Pros:**
  - Keeps `mcp` reserved for external MCP server management and discovery.
  - Places internal tools under the agent hub subsystem, which owns role gating and attribution.
  - Avoids implying that operators should manage this like a standalone server.
- **Cons:**
  - Slightly less obvious that the underlying protocol is MCP.
- **Cascading implications:**
  - Canonical commands become:
    - `substrate agent toolbox status [--json]`
    - `substrate agent toolbox env [--json]`
  - Env exports should avoid `SUBSTRATE_MCP_*` naming to prevent collisions with external MCP wiring.
- **Risks:**
  - If an operator searches for “mcp” they may not find this; mitigated by docs references and `help` text.
- **Unlocks:**
  - Clear conceptual separation between internal privileged toolbox and future external MCP management.
- **Quick wins / low-hanging fruit:**
  - Add `help` text that explicitly states “protocol is MCP; CLI namespace is `agent toolbox`”.

**Option B — Use `substrate mcp …` for the internal toolbox**
- **Pros:**
  - Short, conventional label for MCP.
- **Cons:**
  - Collides with future external MCP management semantics.
  - Encourages treating the internal toolbox as a generic server rather than a privileged agent-hub surface.
- **Cascading implications:**
  - Requires later renaming or deconflicting once external MCP support lands.
- **Risks:**
  - Long-term UX drift and confusing docs.
- **Unlocks:**
  - Short-term naming convenience only.
- **Quick wins / low-hanging fruit:**
  - None worth the eventual rename.

**Recommendation**
- **Selected:** Option A — Use `substrate agent toolbox …` for the internal toolbox.
- **Rationale (crisp):** Reserves `mcp` for external server management and keeps internal privileged tools anchored to the agent hub mental model.

**Follow-up tasks (explicit)**
- Rename CLI docs from `substrate mcp …` to `substrate agent toolbox …`.
- Use `SUBSTRATE_AGENT_TOOLBOX_*` env output keys for internal toolbox wiring.

---

### DR-0007 — Toolbox UDS endpoint placement + permissions (deterministic per-session path vs temp/random)

**Decision owner(s):** Agent toolbox + Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

**Problem / Context**
- We chose UDS-first transport, but we must lock where the UDS socket lives, what permissions it uses, and how stale sockets are handled.
- This impacts determinism (“where do I find it”), cleanup guarantees, and cross-platform parity.

**Option A — Deterministic per-session socket path with user-only permissions (recommended)**
- **Pros:**
  - Predictable and debuggable; easy to report in `substrate agent toolbox status`.
  - Clear permissions story (user-only by default).
  - Cleanup can be deterministic (unlink stale sockets at startup).
- **Cons:**
  - Requires explicit cleanup behavior on crash/restart.
- **Cascading implications:**
  - Default UDS permissions:
    - parent dir: `0700`
    - socket file: `0600`
  - Stale-socket handling:
    - On startup, if the target socket path exists, the toolbox server MUST attempt to detect staleness and MUST unlink stale sockets before binding.
- **Risks:**
  - If cleanup logic is buggy, stale sockets could block startup; mitigated by “detect + unlink stale” and clear errors.
- **Unlocks:**
  - Stable operational semantics and reproducible diagnostics.
- **Quick wins / low-hanging fruit:**
  - Implement “ensure dir + unlink stale + bind” logic for all platforms.

**Option B — Temp directory + randomized socket path**
- **Pros:**
  - Lower chance of collisions; fewer long-lived artifacts.
- **Cons:**
  - Less discoverable; temp-dir behavior varies across OSes.
  - Harder to reason about lifecycle and cleanup across sessions.
- **Cascading implications:**
  - Operators must always consult `env/status` to discover the endpoint.
- **Risks:**
  - Debuggability regressions and inconsistent behavior.
- **Unlocks:**
  - Slightly simpler collision avoidance, at the cost of determinism.
- **Quick wins / low-hanging fruit:**
  - Prototype quickly, but likely needs redesign later.

**Recommendation**
- **Selected:** Option A — Deterministic per-session socket path with user-only permissions.
- **Rationale (crisp):** Determinism + debuggability are worth the small amount of explicit cleanup logic; this also enables a clear permissions posture.

**Follow-up tasks (explicit)**
- Document the canonical per-session UDS socket paths (host-scoped vs world-scoped) in ADR-0026.
- Ensure `substrate agent toolbox status/env` report the exact endpoint string.

---

### DR-0008 — Toolbox caller identity + auth (per-session token vs implicit trust)

**Decision owner(s):** Agent toolbox + Agent Hub maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- The toolbox server must enforce `role=orchestrator` gating and attribute tool calls to the correct `(agent_id, role, orchestration_session_id)`.
- UDS permissions reduce exposure, but do not fully bind calls to the intended orchestrator identity (any process with socket access could connect).
- The auth mechanism must not require a manual operator step in normal operation.

**Option A — Per-orchestration-session auth token minted by Agent Hub (recommended)**
- **Pros:**
  - Binds toolbox access to the orchestrator session deterministically.
  - Keeps role-gated tools from being callable by unrelated local processes that can reach the socket.
  - Clear auditability: the hub mints identity + token; the toolbox enforces.
- **Cons:**
  - Requires secure token distribution and careful redaction (never log/print by default).
- **Cascading implications:**
  - The Agent Hub MUST mint a token at orchestration session start.
  - The toolbox server MUST deny requests with missing/invalid tokens.
  - Debug/manual wiring may exist, but MUST be explicit and MUST NOT print secrets by default.
- **Risks:**
  - Token leakage via operator workflows if printed; mitigated by “not printed by default” and redaction rules.
- **Unlocks:**
  - A legitimate security posture for internal privileged tools that can coexist with untrusted/unknown processes in the same user/world context.
- **Quick wins / low-hanging fruit:**
  - Implement a simple handshake/metadata token check and treat the token as a secret (redaction/caps apply).

**Option B — No auth token; rely on UDS permissions + implicit trust**
- **Pros:**
  - Simplest to implement.
- **Cons:**
  - Any process with socket access can call privileged tools; weak identity story.
  - Harder to defend against accidental or malicious invocation from unrelated processes.
- **Cascading implications:**
  - The toolbox becomes effectively a privileged local API with no caller binding.
- **Risks:**
  - Privilege escalation within the same user/world context.
- **Unlocks:**
  - Short-term convenience only.
- **Quick wins / low-hanging fruit:**
  - None worth the security tradeoff.

**Recommendation**
- **Selected:** Option A — Per-orchestration-session auth token minted by Agent Hub.
- **Rationale (crisp):** UDS permissions are necessary but not sufficient; token binding provides deterministic, auditable access control without requiring manual operator steps.

**Follow-up tasks (explicit)**
- Define the token injection mechanism for orchestrator sessions and ensure it is not printed/logged by default.

---

### DR-0009 — Toolbox token injection mechanism (env var vs inherited one-time FD)

**Decision owner(s):** Agent toolbox + Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

**Problem / Context**
- DR-0008 requires a per-session token, but we must decide how the Agent Hub injects it into the orchestrator session.
- The mechanism must not require manual operator steps in normal operation and should minimize accidental disclosure in logs/diagnostics.

**Option A — Inject token via environment variable**
- **Pros:**
  - Simple to implement for any spawned process.
  - Easy to debug with `env` output.
- **Cons:**
  - Higher accidental exposure risk (debug logs, crash dumps, operator workflows).
  - Encourages secret env var proliferation.
- **Cascading implications:**
  - Requires a canonical env var name and strict redaction rules everywhere.
- **Risks:**
  - Secret leakage via diagnostics or operator workflows.
- **Unlocks:**
  - Fastest path to ship, with weaker posture.
- **Quick wins / low-hanging fruit:**
  - Straightforward for v1, but likely needs hardening later.

**Option B — Inject token via inherited one-time pipe/FD (recommended)**
- **Pros:**
  - No secret in env; no on-disk secret artifact.
  - Only the orchestrator process that inherits the FD can read the token.
  - Stronger default posture against accidental disclosure.
- **Cons:**
  - More plumbing: requires a small “secret channel” abstraction for spawning and for orchestrator adapters.
  - Not universally applicable for non-spawned/remote orchestrators (those require a different auth story).
- **Cascading implications:**
  - Define a standard “toolbox token FD” contract for orchestrator backends spawned by the Agent Hub.
  - Ensure the FD is read-once and not forwarded to child processes unless explicitly intended.
  - `substrate agent toolbox env --include-token` is debug-only and exists for manual wiring; it MUST NOT be required for normal operation.
- **Risks:**
  - Platform abstraction complexity if/when Windows handle passing is required.
- **Unlocks:**
  - A reusable secure channel pattern for other internal secrets between Substrate-managed processes.
- **Quick wins / low-hanging fruit:**
  - Implement for Unix first (Linux/macOS/WSL), with explicit “not supported” behavior elsewhere.

**Recommendation**
- **Selected:** Option B — Inject token via inherited one-time pipe/FD.
- **Rationale (crisp):** Provides a stronger default security posture without adding file artifacts or requiring manual operator steps.

**Follow-up tasks (explicit)**
- Introduce a shared helper (crate/module) for “secret via FD” injection for Substrate-managed child processes.
- Define `substrate agent toolbox env --include-token` as a debug-only escape hatch for manual wiring.
