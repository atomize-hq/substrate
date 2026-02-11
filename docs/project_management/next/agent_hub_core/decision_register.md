# Decision Register — agent_hub_core

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Agent backend id mapping (derived from agent file vs explicit backend_id field)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`, `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

**Problem / Context**
- Policy allowlists and trace attribution require a stable `<kind>:<name>` backend id format (ADR-0027).
- Agent inventory files must remain Windows-safe (no `:` in filenames).
- We need a deterministic mapping from an agent inventory item to a backend id without introducing another identifier that operators must keep in sync.

**Option A — Backend id is derived from the agent file (`<kind>:<agent_id>`)**
- **Pros:**
  - No additional identifier to maintain; deterministic and Windows-safe.
  - Aligns with existing conventions like `cli:codex` and `cli:claude_code`.
  - Keeps allowlists and diagnostics human-typable.
- **Cons:**
  - The backend id “name” becomes coupled to the agent file `id` (renames require updating allowlists and references).
- **Cascading implications:**
  - The Agent Hub MUST treat `backend_id = "{config.kind}:{id}"` as the canonical backend id for gating and attribution.
  - A future “multiple instances of the same backend kind/name” would require a new modeling layer (tracked separately).
- **Risks:**
  - Operators may want multiple `cli:codex` instances; v1 does not support that without an additional instance-id concept.
- **Unlocks:**
  - Simple, stable mapping across LLM gateway, CLI engine, and agent hub without extra schema churn.
- **Quick wins / low-hanging fruit:**
  - Reuse the same id set for `llm.allowed_backends[*]` and `agents.allowed_backends[*]`.

**Option B — Agent file carries an explicit `config.backend_id`**
- **Pros:**
  - Supports multiple agent instances mapping to the same backend id.
  - Decouples filename/id from backend naming.
- **Cons:**
  - Another identifier to keep consistent; more operator error surface.
  - Requires schema updates and additional validation rules (“backend_id matches allowlist format”, “no collisions”, etc.).
- **Cascading implications:**
  - The hub must validate uniqueness and define precedence if multiple agents claim the same `backend_id`.
- **Risks:**
  - Drift between `id` and `backend_id` makes troubleshooting harder.
- **Unlocks:**
  - Multi-instance support (not required for v1).
- **Quick wins / low-hanging fruit:**
  - None without additional schema work.

**Recommendation**
- **Selected:** Option A — Backend id is derived from the agent file (`<kind>:<agent_id>`).
- **Rationale (crisp):** Provides a deterministic, Windows-safe mapping that aligns with ADR-0027’s id format without introducing another operator-maintained identifier.

**Follow-up tasks (explicit)**
- Ensure Agent Hub uses derived backend ids for policy allowlist checks and trace attribution.
- Document the “no multi-instance per backend id” v1 constraint in ADR-0025.

---

### DR-0002 — Hub registry persistence (in-memory derived registry vs file-backed runtime registry)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- The Agent Hub needs a deterministic view of “what agents exist” and their current role assignments.
- Persisting runtime state can complicate correctness, introduce stale state, and create new protected-path concerns.

**Option A — In-memory registry derived from inventory + effective config/policy (recommended)**
- **Pros:**
  - No stale on-disk runtime state; the hub view is always recomputable.
  - Keeps persistence responsibilities in existing sinks (trace/event logs), not in ad-hoc registries.
  - Fewer protected-path and permissions concerns.
- **Cons:**
  - A process restart loses “live session” state (but historical state is still in trace).
- **Cascading implications:**
  - `substrate agent status` must present “current process view” and can optionally reference trace for historical lookback (future).
- **Risks:**
  - Operators may want “last known live state” across restarts; defer to router daemon/workflow engine persistence tracks.
- **Unlocks:**
  - Fast, correct v1 implementation with fewer failure modes.
- **Quick wins / low-hanging fruit:**
  - Build registry from inventory scan + capability validation at startup.

**Option B — File-backed runtime registry (sqlite/jsonl)**
- **Pros:**
  - Can persist session inventory and status across restarts.
- **Cons:**
  - Stale state risks; requires concurrency/locking; needs migration/compat policy.
  - Introduces additional “state sink” beyond trace with its own redaction and safety rules.
- **Cascading implications:**
  - Requires documented schema, rotation, cleanup, and consistency rules.
- **Risks:**
  - Becomes an accidental “source of truth” that drifts from trace and effective config.
- **Unlocks:**
  - Durable operational dashboards (but router daemon is a better home).
- **Quick wins / low-hanging fruit:**
  - None without significant correctness work.

**Recommendation**
- **Selected:** Option A — In-memory registry derived from inventory + effective config/policy.
- **Rationale (crisp):** Avoids stale runtime state and keeps persistence in the canonical trace/event sinks.

**Follow-up tasks (explicit)**
- Define the exact “live status” fields for `substrate agent status` and ensure they are derivable without new on-disk state.

---

### DR-0003 — Role assignment source (explicit config selection vs implicit heuristics)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

**Problem / Context**
- Role assignment for privileged orchestration tool access is a security boundary (tool gating) and must be deterministic and auditable.
- Implicit selection (first agent, “most capable” agent) is prone to drift and surprises.

**Option A — Explicit orchestrator selection via config (fail-closed if missing)**
- **Pros:**
  - Deterministic and operator-controlled.
  - Easy to audit: one key defines the orchestrator identity.
  - Prevents surprise “orchestrator drift” when inventory changes.
- **Cons:**
  - Requires operators to set one additional key to enable orchestration.
- **Cascading implications:**
  - Agent Hub MUST fail closed (exit code `2` / config error) if `agents.hub.orchestrator_agent_id` refers to a missing/disabled agent.
- **Risks:**
  - Minor setup friction; mitigated by good `substrate agent doctor` output.
- **Unlocks:**
  - Safe tool gating and consistent operator expectations.
- **Quick wins / low-hanging fruit:**
  - Add `substrate agent doctor --json` to explain orchestrator selection and eligibility.

**Option B — Implicit orchestrator selection (heuristics)**
- **Pros:**
  - Less configuration upfront.
- **Cons:**
  - Non-deterministic as inventory changes; surprising; hard to audit.
  - Increases security risk by accidentally granting orchestration tools to the wrong agent.
- **Cascading implications:**
  - Requires documenting and maintaining selection heuristics (capability ranking, tie-breakers).
- **Risks:**
  - Drift and misassignment during upgrades or when adding new backends.
- **Unlocks:**
  - None compatible with Substrate’s “auditable enforcement” goals.
- **Quick wins / low-hanging fruit:**
  - None without security tradeoffs.

**Recommendation**
- **Selected:** Option A — Explicit orchestrator selection via config (fail-closed if missing).
- **Rationale (crisp):** Role assignment is a security boundary; deterministic config selection avoids drift and surprises.

**Follow-up tasks (explicit)**
- Introduce `agents.hub.orchestrator_agent_id: <agent_id>` as an additive config key (strict parsing).
- Ensure role assignment decisions are emitted as structured events (and traced).
- Treat `role` as a taxonomy label (not a closed enum); reserve `orchestrator` for toolbox gating and use `member` as the v1 default for non-orchestrators.

---

### DR-0004 — World session reuse semantics for multi-agent operation (shared per orchestration_session vs per-agent worlds)

**Decision owner(s):** Shell + World + Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`, `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- Multi-agent operation needs an explicit statement about whether concurrently running in-world agents share a filesystem/isolation boundary (`world_id`).
- Sharing a world improves collaboration (shared workspace view); isolating a world improves containment and determinism but increases cost and complexity.

**Option A — Default: one shared `world_id` per `orchestration_session_id` for all world-scoped agents**
- **Pros:**
  - Matches operator expectations for “one session, one workspace view”.
  - Efficient: fewer world startups and fewer duplicated mounts/overlays.
  - Makes cross-agent coordination feasible (member edits are visible to orchestrator).
- **Cons:**
  - Agents can interfere with each other via shared filesystem state.
- **Cascading implications:**
  - The hub MUST surface `world_id` on structured events so operators can verify sharing.
  - World restart conditions must be explicit and logged (config/policy drift, workspace change, backend change).
- **Risks:**
  - A misbehaving agent can affect others; mitigated by policy, protected paths, and (future) per-agent isolation mode.
- **Unlocks:**
  - Practical multi-agent collaboration for v1 without heavy world lifecycle complexity.
- **Quick wins / low-hanging fruit:**
  - Reuse the existing “world session” abstraction keyed by orchestration session identity.

**Option B — One `world_id` per agent (always isolated worlds)**
- **Pros:**
  - Stronger containment between agents; less shared-state interference.
- **Cons:**
  - Harder to coordinate multi-agent work; higher resource usage; more world startups.
  - Requires explicit artifact sharing between worlds (file sync strategy becomes mandatory).
- **Cascading implications:**
  - Requires a new contract for cross-world state sharing (future world-sync track).
- **Risks:**
  - Complexity explosion for v1; increased flakiness in integration and local UX.
- **Unlocks:**
  - Strong isolation mode for high-risk multi-agent scenarios (future).
- **Quick wins / low-hanging fruit:**
  - None without a broader world-sync story.

**Recommendation**
- **Selected:** Option A — Default: one shared `world_id` per `orchestration_session_id` for all world-scoped agents.
- **Rationale (crisp):** v1 needs a usable collaboration model; shared-world semantics are the simplest deterministic contract that matches operator expectations and avoids forcing a world-sync story.

**Follow-up tasks (explicit)**
- Define the explicit world restart triggers and emit a structured `world_restarted` event with `reason`.
- Ensure structured agent events include `world_id` when `execution.scope=world`.

---

### DR-0005 — Backend event streaming model (push vs pull)

**Decision owner(s):** Agent Hub maintainers  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`, `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- The hub must ingest structured events from multiple backends concurrently and route them deterministically.
- Backends include CLI wrappers and API agents; both can naturally emit an event stream.

**Option A — Push: backends emit events into the hub (recommended)**
- **Pros:**
  - Natural for streaming: events arrive as they happen.
  - Avoids polling overhead and complexity; supports bursty output.
  - Works well with bounded buffers and drop-with-summary semantics (ADR-0017).
- **Cons:**
  - Requires a well-defined backpressure policy (hub must not block backends indefinitely).
- **Cascading implications:**
  - Hub must bound per-session queues and define overflow behavior (rendering vs persistence are separate concerns).
- **Risks:**
  - Event bursts can overwhelm the UI path; mitigated by ADR-0017 buffering decisions.
- **Unlocks:**
  - Clean alignment with streaming LLM/gateway behavior.
- **Quick wins / low-hanging fruit:**
  - Use async channels for event ingestion and a central router for attribution.

**Option B — Pull: hub polls backends for events**
- **Pros:**
  - Hub controls pacing.
- **Cons:**
  - Adds latency and complexity; can miss high-frequency events or require tight polling loops.
  - Harder to support multiple backends efficiently.
- **Cascading implications:**
  - Requires defining per-backend polling APIs and scheduling fairness.
- **Risks:**
  - Either wastes CPU or introduces unpredictable delays.
- **Unlocks:**
  - None needed for v1.
- **Quick wins / low-hanging fruit:**
  - None relative to push streaming.

**Recommendation**
- **Selected:** Option A — Push: backends emit events into the hub.
- **Rationale (crisp):** Streaming correctness and simplicity favor push; polling adds complexity and latency without improving auditability.

**Follow-up tasks (explicit)**
- Define a per-session bounded queue and explicit overflow behavior for hub ingestion (distinct from UI rendering buffers).

---

### DR-0006 — CLI command placement for Agent Hub (top-level `substrate agent` vs `substrate host|world agent`)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`, `crates/shell/src/execution/cli.rs`

**Problem / Context**
- Substrate’s CLI already has a stable mental model with top-level subsystems (`config`, `policy`, `shim`, `health`, `workspace`) and scoped operational planes (`host`, `world`).
- Agent Hub operations are primarily control-plane (inventory, routing, role assignment) and can span both host-scoped and world-scoped agent executions.
- We need a discoverable command surface that does not conflate “host vs world operational plane” with “filtering agents by their configured execution scope”.

**Option A — Top-level `substrate agent …` (recommended), with filters**
- **Pros:**
  - Matches existing CLI structure: Agent Hub is a subsystem similar to `config`/`policy`.
  - Avoids expanding `host` and `world` into general “command namespaces” beyond their current operational scopes.
  - Supports a single “list everything” view with explicit filters (`--scope host|world`, `--role <role>`).
- **Cons:**
  - Requires adding filter flags for convenience if operators want “world-only” or “host-only” views.
- **Cascading implications:**
  - `substrate agent list/status/doctor` become the canonical entrypoints.
  - `--scope` filtering MUST be defined as a view filter only; it MUST NOT change global `--world/--no-world` execution toggles.
- **Risks:**
  - Operators may initially expect `substrate world agents …` to exist; mitigated via help text and documentation.
- **Unlocks:**
  - Clear separation of concerns: plane (`host|world`) vs agent config (`execution.scope`).
- **Quick wins / low-hanging fruit:**
  - Implement `--scope` filter for `agents list` and `agents status`.

**Option B — Nested under planes: `substrate host agent …` and `substrate world agent …`**
- **Pros:**
  - “Reads” cleanly if interpreted as plane-scoped commands.
  - Could provide short defaults (world-only list vs host-only list).
- **Cons:**
  - Expands `host/world` from “operational plane” into a general namespace, unlike the current CLI shape.
  - Risks conflating plane selection with agent filtering (an agent’s `execution.scope` may differ from the command prefix semantics).
- **Cascading implications:**
  - Requires defining duplication/precedence if both `substrate agent list` and `substrate world agent list` exist.
  - Requires careful docs to prevent “world prefix means this command runs in-world” confusion (many Agent Hub operations are host control-plane).
- **Risks:**
  - Mental model drift: `host/world` no longer primarily mean “doctor + world lifecycle”.
- **Unlocks:**
  - Shortcut commands, at the cost of conceptual ambiguity.
- **Quick wins / low-hanging fruit:**
  - None without adding new nested subcommand trees in clap.

**Recommendation**
- **Selected:** Option A — Top-level `substrate agent …` with filters.
- **Rationale (crisp):** Agent Hub is a subsystem; keeping it top-level preserves the existing CLI mental model and avoids conflating plane selection with agent execution-scope filtering.

**Follow-up tasks (explicit)**
- Add `--scope host|world|any` to `substrate agent list` and `substrate agent status` as view-only filters.
- Ensure help text explicitly distinguishes global `--world/--no-world` toggles (execution isolation) from `--scope` (agent inventory filter).

---

### DR-0007 — Orchestrator execution scope posture (host-scoped orchestrator vs allow in-world orchestrator)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`, `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`, `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md`

**Problem / Context**
- The orchestrator is a privileged control-plane role (toolbox access, policy/trace introspection, routing decisions).
- Separately, task execution is intended to happen inside a world boundary via world-scoped member agents.
- The ADRs currently imply `orchestrator.execution.scope` could be `host|world`, which creates ambiguity about where orchestration runs and how it “executes in world”.

**Option A — Require the orchestrator to be host-scoped in v1 (recommended)**
- **Pros:**
  - Clean separation: host control-plane (orchestrator) vs world data-plane (member agents).
  - Avoids cross-boundary complexity for toolbox/policy/trace introspection in v1.
  - Matches the intended shape: orchestrator executes in-world *indirectly* by dispatching world-scoped agents.
- **Cons:**
  - Operators must ensure the orchestrator agent inventory sets `config.execution.scope=host`.
- **Cascading implications:**
  - The agent selected by `agents.hub.orchestrator_agent_id` MUST have `config.execution.scope=host` in its agent inventory file.
  - If the selected orchestrator has `config.execution.scope=world`, Agent Hub MUST fail closed with an actionable config error (exit code `2`).
  - “Orchestrator executes in world” is achieved by routing/dispatching tasks to world-scoped member agents with their own toolsets/policy overlays (not by moving the orchestrator process into the world).
- **Risks:**
  - Confusion if operators assume “everything must be in-world”; mitigated by `substrate agent doctor` output and docs that explicitly describe host-control-plane + world-data-plane.
- **Unlocks:**
  - A stable v1 posture that keeps orchestration privileged surfaces simpler and easier to secure.
- **Quick wins / low-hanging fruit:**
  - Enforce the orchestrator scope check at selection time and surface it in `substrate agent doctor --json`.

**Option B — Allow the orchestrator to be host-scoped or in-world**
- **Pros:**
  - More flexible for environments that prefer everything to run in-world.
- **Cons:**
  - Ambiguity and complexity: toolbox endpoint placement, host-only state access, and diagnostics become cross-boundary problems.
  - Increased risk of subtle drift and “works on one platform, fails on another” behavior.
- **Cascading implications:**
  - Must define an explicit cross-boundary contract for toolbox/policy/trace access when orchestrator is in-world.
  - Requires per-tool “supported in-world vs not supported” matrices (v1 overhead).
- **Risks:**
  - Security/correctness footguns if any host-only state leaks or is inconsistently proxied.
- **Unlocks:**
  - In-world-first orchestration posture.
- **Quick wins / low-hanging fruit:**
  - None without a larger proxying story.

**Recommendation**
- **Selected:** Option A — Require the orchestrator to be host-scoped in v1.
- **Rationale (crisp):** Preserves the intended control-plane/data-plane split and avoids cross-boundary complexity while still allowing orchestrators to drive in-world execution by dispatching world-scoped member agents.

**Follow-up tasks (explicit)**
- Update ADR-0025 `doctor` checks and config semantics to reflect “host orchestrator + world members” posture.
- Update ADR-0026 toolbox endpoint semantics to be host-scoped in v1 and remove/mark any in-world orchestrator implications as future work.
