# Decision Register — host_event_bus_router_daemon

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Daemon packaging (standalone binary vs `substrate` subcommand)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router daemon must be “always-on” on the host across Linux/macOS/Windows, but we want to minimize packaging complexity and keep operator UX consistent.

**Option A — Standalone daemon binary (e.g., `substrate-busd`)**
- **Pros:**
  - Clear process identity and separation from the CLI.
  - Service units can reference a dedicated executable.
- **Cons:**
  - Additional build/release surface (another binary to ship/version).
  - Increases operator confusion (“which binary do I run?”) and multiplies install pathways.
- **Cascading implications:**
  - Requires explicit installer + upgrade logic for a second binary across platforms.
- **Risks:**
  - Version drift between `substrate` and the daemon breaks schema compatibility.
- **Unlocks:**
  - Future split packaging (daemon-only install) if ever needed.
- **Quick wins / low-hanging fruit:**
  - None; requires build/install plumbing immediately.

**Option B — Daemon runs as a `substrate` subcommand (recommended)**
- **Pros:**
  - Single binary to ship/version; avoids drift.
  - Uniform operator mental model: “Substrate does X; daemon is a mode.”
  - Service managers can still run `substrate bus daemon` as the ExecStart.
- **Cons:**
  - Process name is `substrate` unless explicitly overridden in service definitions.
- **Cascading implications:**
  - CLI must implement a stable `substrate bus daemon [--foreground]` contract usable by service managers.
- **Risks:**
  - If `substrate` gains heavy startup cost, daemon cold-start may regress (mitigate via careful init and/or future socket-activation).
- **Unlocks:**
  - Simpler initial delivery of “always-on” behavior via platform units that all invoke the same command.
- **Quick wins / low-hanging fruit:**
  - Reuse existing CLI + config plumbing for locating `SUBSTRATE_HOME`, effective config/policy, and trace paths.

**Recommendation**
- **Selected:** Option B — Daemon runs as a `substrate` subcommand.
- **Rationale (crisp):** Keeps distribution and versioning simple while still supporting true always-on behavior via service managers invoking `substrate bus daemon`.

**Follow-up tasks (explicit)**
- Define `substrate bus daemon` flags required for service operation (`--foreground`, log destination, `--json` health).
- Add platform unit templates (systemd/launchd/Windows service) that execute `substrate bus daemon` with an explicit `SUBSTRATE_HOME`.

---

### DR-0002 — Event ingestion (tail `trace.jsonl` vs direct publish API)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router daemon must subscribe to execution outcomes and fs diffs without introducing a second, divergent event plane or weakening auditability.

**Option A — Tail the canonical `SUBSTRATE_HOME/trace.jsonl` (recommended)**
- **Pros:**
  - Trace remains the single canonical event stream; no new ingress surface required.
  - Replay/debugging stays simple (one file and consistent schema).
  - Fewer security surfaces; no new local socket API that needs auth.
- **Cons:**
  - Must handle trace rotation correctly (inode changes / file rename).
  - Slightly higher latency vs in-process publish.
- **Cascading implications:**
  - Cursor persistence MUST be robust across rotation (`trace.jsonl` → `trace.jsonl.1`) and truncation.
- **Risks:**
  - Incorrect tailing could miss or reprocess events; mitigated by durable cursor + dedupe.
- **Unlocks:**
  - Works even when world backends are down; no coupling to world-agent availability.
- **Quick wins / low-hanging fruit:**
  - Implement a minimal follower using `(inode, byte_offset)` and detect rotation via inode/size changes.

**Option B — Direct publish from core components to the bus over a local API (UDS/NamedPipe)**
- **Pros:**
  - Lower latency and avoids tailer edge cases.
  - Enables explicit backpressure and “bus is down” diagnostics at publish time.
- **Cons:**
  - New privileged ingress surface that must be secured and policy-gated.
  - Creates coupling between producers and bus schema/versioning.
- **Cascading implications:**
  - Requires an auth model for local publishing and a compat policy between producer versions and bus.
- **Risks:**
  - Becomes a second “source of truth” and drifts from trace unless carefully enforced.
- **Unlocks:**
  - Future remote ingress could re-use this API surface (but that is explicitly out of scope for v1).
- **Quick wins / low-hanging fruit:**
  - None without new transport/auth design work.

**Recommendation**
- **Selected:** Option A — Tail the canonical `trace.jsonl`.
- **Rationale (crisp):** Preserves trace as the single source of truth and avoids introducing a new privileged ingestion API in v1.

**Follow-up tasks (explicit)**
- Specify and implement a rotation-safe cursor model for the follower (inode + offset + “rewind window” for safety).
- Add tests that rotate trace while the follower is running and assert no missed events for the monitored window.

---

### DR-0003 — Derived bus events location (append to `trace.jsonl` vs separate bus log)

**Decision owner(s):** Trace + Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Rule matches, request enqueueing, allow/deny outcomes, and execution outcomes must be observable, replayable, and attributable.

**Option A — Append derived bus events into the canonical `SUBSTRATE_HOME/trace.jsonl` (recommended)**
- **Pros:**
  - Single canonical audit log; derived actions are visible to the same tooling as executions.
  - Replay tooling can reason about triggers and their consequences.
  - Operators can `tail -f trace.jsonl | jq ...` to debug routing.
- **Cons:**
  - Requires explicit recursion guard (bus must not trigger itself by default).
  - Adds additional event volume to trace.
- **Cascading implications:**
  - Derived bus events MUST be clearly marked (`component=bus`) and excluded from triggers by default.
- **Risks:**
  - Poorly designed derived events can leak payload details; must follow redaction rules and avoid including sensitive content.
- **Unlocks:**
  - Consistent observability across execution, routing, and policy outcomes.
- **Quick wins / low-hanging fruit:**
  - Emit a small, stable set of derived event types (rule_match, request_enqueued, request_denied, action_executed).

**Option B — Write derived bus events to a separate `SUBSTRATE_HOME/bus/derived.jsonl`**
- **Pros:**
  - Avoids recursion issues by default.
  - Keeps trace smaller and focused on execution spans.
- **Cons:**
  - Splits the audit log; requires additional tooling and docs.
  - Harder to correlate cause/effect without a unified stream.
- **Cascading implications:**
  - Requires join keys and additional documentation for operators to interpret multi-file flows.
- **Risks:**
  - Operators treat one log as canonical and miss critical routing outcomes.
- **Unlocks:**
  - Potentially simpler high-volume bus diagnostics, at the cost of coherence.
- **Quick wins / low-hanging fruit:**
  - None; still requires correlation and tooling work.

**Recommendation**
- **Selected:** Option A — Append derived bus events into `trace.jsonl`.
- **Rationale (crisp):** Keeps routing outcomes auditable and replayable without introducing a second “event truth.”

**Follow-up tasks (explicit)**
- Define derived bus event types and required correlation keys, aligned to ADR-0028.
- Implement a recursion guard so bus-derived events are non-triggerable by default.

---

### DR-0004 — Durable queue format (JSONL + state vs sqlite)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The bus requires durable “inbox” and “work queue” semantics, plus crash-safe cursors, bounded retries, and bounded dedupe state.

**Option A — JSONL queues + `state.json` (recommended)**
- **Pros:**
  - Inspectable with simple tooling (`jq`, `tail`).
  - Matches the project’s existing JSONL trace idioms.
  - Low dependency footprint.
- **Cons:**
  - Requires careful design for concurrency and compaction (state growth and queue cursor correctness).
- **Cascading implications:**
  - Status transitions are emitted as trace events (append-only); queue entries remain immutable.
  - Dedupe state MUST be bounded (age- or size-based) to prevent unbounded `state.json` growth.
- **Risks:**
  - Corruption or partial writes need explicit durability rules (atomic state writes, fsync posture).
- **Unlocks:**
  - Fast v1 delivery with minimal complexity and maximal operator inspectability.
- **Quick wins / low-hanging fruit:**
  - Use an atomic write strategy for `state.json` (write temp + rename) and keep queue writers append-only.

**Option B — sqlite state store (tables for requests, queue, cursors, dedupe)**
- **Pros:**
  - Built-in transactional durability and concurrency primitives.
  - Easier to implement claim/lease semantics and backoff scheduling.
- **Cons:**
  - Adds dependency + schema/migration surface (must define compat policy).
  - Less inspectable without tooling; harder to “tail” behavior.
- **Cascading implications:**
  - Requires schema versioning and migration policies from day 1.
- **Risks:**
  - Migration failures become operator incidents; must support repair tooling.
- **Unlocks:**
  - More robust multi-consumer scenarios (future).
- **Quick wins / low-hanging fruit:**
  - None without full schema/migration planning.

**Recommendation**
- **Selected:** Option A — JSONL queues + `state.json`.
- **Rationale (crisp):** Delivers durable routing with low dependency and maximum inspectability, while keeping correctness manageable via bounded dedupe and append-only semantics.

**Follow-up tasks (explicit)**
- Specify `state.json` schema including: trace cursor, inbox cursor, work queue cursor, bounded dedupe window, retry counters/backoff metadata.
- Implement atomic state persistence and tests for crash recovery + duplicate suppression.

---

### DR-0005 — Workspace identity (path-hash id vs explicit id stored in workspace metadata)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Cross-workspace routing requires a stable `workspace_id` that is safe to log and does not break when workspace paths move.

**Option A — `workspace_id = sha256(canonical_workspace_root_path)`**
- **Pros:**
  - Deterministic and requires no additional file writes.
  - Can be derived on-demand from a path.
- **Cons:**
  - Workspace moves change identity, breaking cross-workspace rules and history correlations.
  - Leaks information about the path (dictionary-attackable).
- **Cascading implications:**
  - Requires migration behavior for path moves (explicitly undesirable for v1).
- **Risks:**
  - Operators accidentally “fork identity” by moving directories, causing silent rule mismatch.
- **Unlocks:**
  - Simple initial implementation at the cost of long-term stability.
- **Quick wins / low-hanging fruit:**
  - None; path moves remain an incident class.

**Option B — Explicit `workspace_id` stored in workspace metadata (recommended)**
- **Pros:**
  - Stable across path moves; supports durable routing and history correlation.
  - Avoids path leakage by using a random identifier.
- **Cons:**
  - Requires defining and maintaining a workspace metadata file that carries the id.
- **Cascading implications:**
  - `substrate workspace init` MUST ensure `workspace_id` exists and is stable.
  - The global registry stores `(workspace_id, root, enabled, label)` for lookup, but the id source of truth is the workspace metadata.
- **Risks:**
  - Registry and workspace metadata drift must be detected and handled fail-closed.
- **Unlocks:**
  - Safe cross-workspace routing semantics without migrations.
- **Quick wins / low-hanging fruit:**
  - Generate a UUID (v7 preferred) on init and persist it.

**Recommendation**
- **Selected:** Option B — Explicit `workspace_id` stored in workspace metadata.
- **Rationale (crisp):** Prevents identity churn and path leakage while enabling durable cross-workspace routing.

**Follow-up tasks (explicit)**
- Define the authoritative workspace metadata file location and schema (including `workspace_id` and `enabled`).
- Implement `substrate workspace init|enable|disable` to keep registry and workspace metadata consistent.

---

### DR-0006 — Rule declarations (config) vs rule declarations (policy)

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Routing rules enable execution and cross-workspace actions, but we must preserve the “config selects behavior; policy authorizes behavior” separation.

**Option A — Rules live in config; policy gates execution (recommended)**
- **Pros:**
  - Keeps behavior selection in config (operator intent).
  - Policy remains the enforcement surface and can fail-closed without requiring rules to be re-authored.
  - Allows workspace-scoped rule customization while still enforcing workspace policy boundaries.
- **Cons:**
  - Requires new config schema entries and validation.
- **Cascading implications:**
  - Rules are parsed strictly (schema errors are surfaced in `substrate bus doctor`).
  - Policy adds explicit allowlists for rule ids and cross-workspace target allow/deny.
- **Risks:**
  - Misconfigured rules can spam requests; mitigated via rate limits + bounded retries + dedupe.
- **Unlocks:**
  - Flexible routing behavior without embedding execution-enabling logic into policy text.
- **Quick wins / low-hanging fruit:**
  - Start with a minimal rule schema that only supports a narrow trigger allowlist.

**Option B — Rules live entirely in policy**
- **Pros:**
  - Single “execution enabling” surface; operators cannot accidentally enable execution via config alone.
- **Cons:**
  - Policy becomes both behavior selection and enforcement, increasing complexity and drift risk.
  - Makes per-workspace customization harder and more brittle.
- **Cascading implications:**
  - Requires policy schema to grow complex match/action expressions.
- **Risks:**
  - Harder to debug because behavior changes require policy edits even for harmless routing.
- **Unlocks:**
  - Strong central control, at the cost of operator ergonomics.
- **Quick wins / low-hanging fruit:**
  - None without rethinking policy authoring UX.

**Recommendation**
- **Selected:** Option A — Rules live in config; policy gates execution.
- **Rationale (crisp):** Preserves the established separation of concerns while still ensuring all actions are fail-closed under policy.

**Follow-up tasks (explicit)**
- Define bus rule file locations + precedence and add schema to ADR-0027 surfaces additively (`bus.*` keys).
- Define policy gating keys for bus rule allowlisting and cross-workspace routing allow/deny.

---

### DR-0007 — Trigger taxonomy (strict allowlist vs general event matching)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Triggers must be safe, deterministic, and resistant to accidental sensitive-data coupling (stdout/pty bytes/env dumps).

**Option A — Strict allowlist of triggerable `event_type` families (recommended)**
- **Pros:**
  - Fail-closed by design; new trigger types require explicit additions.
  - Avoids accidental triggering on sensitive/high-volume event types.
  - Easier to test and document.
- **Cons:**
  - Slower feature expansion; new event families require schema + allowlist updates.
- **Cascading implications:**
  - Bus-derived events are non-triggerable by default to prevent recursion.
  - Rules can only match against a constrained subset of fields (no raw output payloads).
- **Risks:**
  - Operators ask for “trigger on everything”; must be declined in v1.
- **Unlocks:**
  - Safe baseline for future expansion (workflows/agents) once those event types are stable.
- **Quick wins / low-hanging fruit:**
  - Start with `command_complete` and fs-diff-derived `fs_change` events.

**Option B — General “match any event field” rules**
- **Pros:**
  - Maximum flexibility; fewer hard-coded event family restrictions.
- **Cons:**
  - High risk of accidentally making sensitive payloads triggerable.
  - Harder to ensure determinism and prevent recursion.
- **Cascading implications:**
  - Would require a full rule language plus redaction semantics at the rule layer.
- **Risks:**
  - Turns the bus into a policy-bypass vector (“trigger on stdout containing …”).
- **Unlocks:**
  - Powerful automation at the cost of safety and auditability.
- **Quick wins / low-hanging fruit:**
  - None that remain safe.

**Recommendation**
- **Selected:** Option A — Strict allowlist.
- **Rationale (crisp):** Ensures the trigger surface is safe-by-default and expands only via explicit, reviewable schema changes.

**Follow-up tasks (explicit)**
- Document the v1 triggerable event type set and the fields available for rule matching.
- Add a recursion guard: bus-emitted events are excluded unless a rule explicitly opts in (future; not v1).

---

### DR-0008 — FS triggers source (fs diffs vs external watchers/git feeds)

**Decision owner(s):** Shell + World-Agent maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- We want file-change triggers without expanding the threat surface via host filesystem watching or introducing git as a hard dependency.

**Option A — FS triggers derived only from Substrate-produced `fs_diff` (recommended)**
- **Pros:**
  - Deterministic and policy-aligned (changes are tied to Substrate executions).
  - Avoids background host watchers and associated permissions/DoS risk.
  - Aligns naturally with ADR-0018 matching semantics and redaction posture.
- **Cons:**
  - Does not observe out-of-band changes (manual edits, external tools) unless they run under Substrate.
- **Cascading implications:**
  - Rules must be explicit about what “change” means in terms of `fs_diff` events (create/modify/delete/rename).
- **Risks:**
  - Operators expect “watch my repo”; must be clearly documented as out-of-scope in v1.
- **Unlocks:**
  - Safe baseline that can later incorporate git or host watchers without reworking the core request model.
- **Quick wins / low-hanging fruit:**
  - Emit `fs_change` derived events from `command_complete.fs_diff` with bounded payload size.

**Option B — Add host filesystem watchers and/or git-backed feeds in v1**
- **Pros:**
  - Captures more changes without requiring Substrate-mediated execution.
- **Cons:**
  - Significantly expands threat surface and complexity (permissions, path traversal, event storms).
  - Adds cross-platform differences (inotify/FSEvents/ReadDirectoryChangesW).
- **Cascading implications:**
  - Requires a full monitoring and rate-limiting architecture and additional policy gates.
- **Risks:**
  - Creates a new always-on subsystem that is easy to DoS and hard to make deterministic.
- **Unlocks:**
  - “Watch my tree” experience (future track, not required for routing correctness).
- **Quick wins / low-hanging fruit:**
  - None that remain safe and cross-platform.

**Recommendation**
- **Selected:** Option A — Derive triggers from `fs_diff` only.
- **Rationale (crisp):** Keeps triggers deterministic and policy-aligned while minimizing new background privileges in v1.

**Follow-up tasks (explicit)**
- Define a bounded `fs_change` derived event schema (no content; path metadata only; capped/truncated).
- Implement matching using ADR-0018 semantics and add tests that confirm parity.

---

### DR-0009 — Remote ingress (none in v1 vs authenticated inbound requests)

**Decision owner(s):** Shell + Security  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Remote ingress turns the bus into an attack surface. v1’s goal is local routing from trace → policy-gated actions without network exposure.

**Option A — No remote ingress in v1 (recommended)**
- **Pros:**
  - Strongest safety posture; no new network listener.
  - Keeps scope focused on local correctness: durable cursors, dedupe, policy gating.
- **Cons:**
  - Remote systems cannot enqueue requests directly.
- **Cascading implications:**
  - Any “external trigger” support must be built later as an explicitly gated feature, likely writing only to `inbox`.
- **Risks:**
  - Operators expect webhooks; must be clearly out-of-scope.
- **Unlocks:**
  - Safe baseline with room to add authenticated ingress later.
- **Quick wins / low-hanging fruit:**
  - N/A (this is a scoping decision).

**Option B — Local-only authenticated ingress (mTLS/token) that enqueues into `inbox`**
- **Pros:**
  - Enables integrations without tailing trace.
  - Provides a stepping stone to future remote webhooks.
- **Cons:**
  - Requires auth, secret management, and policy gates immediately.
  - Still adds an ingress surface that can be abused or misconfigured.
- **Cascading implications:**
  - Must define secret storage rules and rotation (explicitly avoided in v1).
- **Risks:**
  - Misconfiguration creates execution-enabling backdoor semantics.
- **Unlocks:**
  - Integrations earlier (but v1 does not need this).
- **Quick wins / low-hanging fruit:**
  - None without full security design.

**Recommendation**
- **Selected:** Option A — No remote ingress in v1.
- **Rationale (crisp):** Avoids introducing a new attack surface and keeps v1 focused on local, auditable correctness.

**Follow-up tasks (explicit)**
- Ensure derived request schemas have a stable “future remote ingress” slot (e.g., `source.kind=local_trace|remote`) without enabling remote by default.

---

### DR-0010 — Idempotency key strategy (deterministic derived key vs random per request)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The bus is at-least-once by necessity (crashes, restarts, rotation). Without deterministic dedupe, rules can create infinite loops or repeated actions.

**Option A — Deterministic idempotency key derived from `(source_event, rule_id, target, action_kind, payload_digest)` (recommended)**
- **Pros:**
  - Enables bounded dedupe and safe reprocessing after crashes.
  - Allows cross-run correlation (same trigger produces same key) for diagnostics.
- **Cons:**
  - Requires stable “source event identity” and a deterministic payload digest.
- **Cascading implications:**
  - Must define a canonical source event identity for v1:
    - `source_event_key = sha256(cmd_id + event_type + ts)` (or a stronger schema once ADR-0028 provides a stable event id).
  - Payload digest must use a canonical JSON encoding of the request payload.
- **Risks:**
  - If the source event identity is not stable enough, dedupe becomes flaky; mitigated by a small rewind window plus per-line digest checks.
- **Unlocks:**
  - Safe “repeatable triggers” without runaway duplication.
- **Quick wins / low-hanging fruit:**
  - Start with `command_complete` where `cmd_id` and `ts` are present.

**Option B — Random id per request; rely on “best effort” suppression**
- **Pros:**
  - Easiest to implement.
- **Cons:**
  - Restarts and rotation lead to repeated actions with no safe suppression mechanism.
  - Makes debugging and replay correlation much harder.
- **Cascading implications:**
  - Requires a stronger ack/lease model or external state store to avoid duplicates.
- **Risks:**
  - Severe: runaway repeated actions and unintended side effects.
- **Unlocks:**
  - None; this is an anti-pattern for a durable routing system.
- **Quick wins / low-hanging fruit:**
  - None that remain correct.

**Recommendation**
- **Selected:** Option A — Deterministic derived key.
- **Rationale (crisp):** Makes at-least-once processing safe by bounding duplicates deterministically.

**Follow-up tasks (explicit)**
- Define a v1 canonical source event identity and upgrade path once stable event ids exist.
- Implement bounded dedupe storage and tests for duplicate suppression across restarts.

