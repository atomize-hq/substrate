# Decision Register — host_event_bus_router_daemon

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Service packaging (standalone binary vs `substrate` subcommand)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The workflow router service must be “always-on” on the host across Linux/macOS/Windows.
- This repo already ships multiple binaries, but the *release* surface is intentionally small:
  - `cargo dist` currently publishes `substrate` and `substrate-shim` only (see `Cargo.toml` `[package.metadata.dist.binaries]`).
- We also care about crate/module boundaries: the router should be isolated as its own crate/module even if it is invoked via `substrate`.

**Option A — Standalone service binary (e.g., `substrate-workflow-routerd`)**
- **Pros:**
  - Clear process identity and separation from the CLI.
  - Service units can reference a dedicated executable.
  - Tighter dependency surface is possible (a small main + a dedicated router crate), reducing long-running resident complexity.
- **Cons:**
  - Additional build/release surface:
    - requires adding a new published binary to `cargo dist` and any downstream packaging (Homebrew/MSI/etc),
    - requires platform service unit templates to reference a second executable path.
  - Operator UX ambiguity unless we *also* keep `substrate workflow serve` as the canonical entrypoint (which then becomes a wrapper around the binary).
  - More “two places to keep consistent” pressure: config/policy resolution, trace output conventions, and schema versioning must be shared rather than duplicated.
- **Cascading implications:**
  - Requires explicit installer + upgrade logic for a second binary across platforms (and uninstall/cleanup rules).
  - Service managers must either:
    - run the standalone binary directly, or
    - run `substrate workflow serve` which `exec`s the standalone binary after resolving paths/env (adds one more indirection step).
  - CLI and service MUST share the same core crate(s) (recommended: a `crates/workflow-router` library used by both binaries) to avoid logic drift.
- **Risks:**
  - Version drift between `substrate` and the service breaks schema compatibility (rules/state formats; derived event types).
  - “Half upgrade” failure mode: service upgraded but CLI not (or vice versa) depending on platform packaging behavior.
- **Unlocks:**
  - Future split packaging (service-only install) if ever needed.
- **Quick wins / low-hanging fruit:**
  - The process boundary can be used as a safety valve: router crashes do not take down the CLI, and restarts are owned by the service manager.

**Option B — Service runs as a `substrate` subcommand (recommended)**
- **Pros:**
  - Single binary to ship/version; avoids drift.
  - Uniform operator mental model: “Substrate does X; service is a mode.”
  - Service managers can still run `substrate workflow serve` as the ExecStart.
  - Crate boundaries are still achievable: the router implementation can live in a dedicated crate (e.g., `crates/workflow-router`) that `substrate-shell` calls from the `workflow serve` subcommand.
- **Cons:**
  - Process name is `substrate` unless explicitly overridden in service definitions.
  - The `substrate` binary grows (more code linked in), so we must be deliberate about dependency creep in the long-running mode.
- **Cascading implications:**
  - CLI must implement a stable `substrate workflow serve [--foreground]` contract usable by service managers.
  - The `workflow serve` path MUST avoid pulling heavyweight features into the always-on process by default (keep dependencies narrow and lazy-init where possible).
- **Risks:**
  - If `substrate` gains heavy startup cost, service cold-start may regress (mitigate via careful init and/or future socket-activation).
  - A bug in shared CLI initialization can impact the service mode; mitigate by having `workflow serve` take a “minimal init” path and by adding smoke tests that run only the service mode.
- **Unlocks:**
  - Simpler initial delivery of “always-on” behavior via platform units that all invoke the same command.
- **Quick wins / low-hanging fruit:**
  - Reuse existing CLI + config plumbing for locating `SUBSTRATE_HOME`, effective config/policy, and trace paths.

**Recommendation**
- **Selected:** Option B — Service runs as a `substrate` subcommand.
- **Rationale (crisp):** Avoids expanding the published-binary surface area (dist + installers) while still allowing clean crate boundaries via a dedicated router crate called by `substrate workflow serve`.

**Follow-up tasks (explicit)**
- Define `substrate workflow serve` flags required for service operation (`--foreground`, log destination, `--json` health).
- Add platform unit templates (systemd/launchd/Windows service) that execute `substrate workflow serve` with an explicit `SUBSTRATE_HOME`.
- Keep router logic in a dedicated crate/module (even under Option B) so Option A remains a future packaging-only change, not a refactor.
- Do not add a new published binary to `cargo dist` in v1 (keep the distributed binary set unchanged while iterating on router correctness).

---

### DR-0002 — Event ingestion (tail `trace.jsonl` vs direct publish API)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The workflow router must subscribe to execution outcomes and fs diffs without introducing a second, divergent event plane or weakening auditability.
- Preference (v1): keep the system file-based (tail the canonical trace) rather than introducing new local RPC surfaces.

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

**Option B — Direct publish from core components to the workflow router over a local API (UDS/NamedPipe)**
- **Pros:**
  - Lower latency and avoids tailer edge cases.
  - Enables explicit backpressure and “router is down” diagnostics at publish time.
- **Cons:**
  - New privileged ingress surface that must be secured and policy-gated.
  - Creates coupling between producers and workflow-router schema/versioning.
- **Cascading implications:**
  - Requires an auth model for local publishing and a compat policy between producer versions and the workflow router.
- **Risks:**
  - Becomes a second “source of truth” and drifts from trace unless carefully enforced.
- **Unlocks:**
  - Future remote ingress could re-use this API surface (but that is explicitly out of scope for v1).
- **Quick wins / low-hanging fruit:**
  - None without new transport/auth design work.

**Recommendation**
- **Selected:** Option A — Tail the canonical `trace.jsonl`.
- **Rationale (crisp):** Preserves `trace.jsonl` as the single source of truth, keeps routing purely file-driven, and avoids introducing a new privileged ingestion API in v1.

**Follow-up tasks (explicit)**
- Specify and implement a rotation-safe cursor model for the follower (inode + offset + “rewind window” for safety).
- Add tests that rotate trace while the follower is running and assert no missed events for the monitored window.

---

### DR-0003 — Derived workflow-router events location (append to `trace.jsonl` vs separate log)

**Decision owner(s):** Trace + Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Rule matches, request enqueueing, allow/deny outcomes, and execution outcomes must be observable, replayable, and attributable.
- In this decision, “derived workflow-router events” means: **new** structured trace records emitted by the workflow router *because* it observed canonical trace events (e.g., `command_complete`) and applied routing logic.
  - They are **not** the router’s durable state/queues (`SUBSTRATE_HOME/workflow/state.json`, `inbox.jsonl`, `work_queue.jsonl`).
  - They are **not** “mutations” of existing trace lines; they are additional appended records with explicit correlation back to the source event.

**Option A — Append derived workflow-router events into the canonical `SUBSTRATE_HOME/trace.jsonl` (recommended)**
- **Pros:**
  - Single canonical audit log; derived actions are visible to the same tooling as executions.
  - Replay tooling can reason about triggers and their consequences.
  - Operators can `tail -f trace.jsonl | jq ...` to debug routing.
- **Cons:**
  - Requires explicit recursion guard (workflow-router events must not trigger themselves by default).
  - Adds additional event volume to trace.
- **Cascading implications:**
  - Derived workflow-router events MUST be clearly marked (`component=workflow_router`) and excluded from triggers by default.
- **Risks:**
  - Poorly designed derived events can leak payload details; must follow redaction rules and avoid including sensitive content.
- **Unlocks:**
  - Consistent observability across execution, routing, and policy outcomes.
- **Quick wins / low-hanging fruit:**
  - Emit a small, stable set of derived event types (rule_match, request_enqueued, request_denied, action_executed).

**Option B — Write derived workflow-router events to a separate `SUBSTRATE_HOME/workflow/derived.jsonl`**
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
  - Potentially simpler high-volume workflow-router diagnostics, at the cost of coherence.
- **Quick wins / low-hanging fruit:**
  - None; still requires correlation and tooling work.

**Recommendation**
- **Selected:** Option A — Append derived workflow-router events into `trace.jsonl`.
- **Rationale (crisp):** Keeps routing outcomes in the single canonical audit stream (`trace.jsonl`) while leaving `SUBSTRATE_HOME/workflow/*` as durable state/queues, not a second event log.

**Follow-up tasks (explicit)**
- Define derived workflow-router event types and required correlation keys, aligned to ADR-0028.
- Implement a recursion guard so workflow-router-derived events are non-triggerable by default.

---

### DR-0004 — Durable queue format (JSONL + state vs sqlite)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The workflow router requires durable “inbox” and “work queue” semantics, plus crash-safe cursors, bounded retries, and bounded dedupe state.

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
- **Rationale (crisp):** Keeps v1 file-based and inspectable (`trace.jsonl` + `SUBSTRATE_HOME/workflow/*`) without introducing sqlite schema/migration baggage, while keeping correctness manageable via bounded dedupe and append-only semantics.

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
  - Rules are parsed strictly (schema errors are surfaced in `substrate workflow doctor`).
  - Policy adds explicit allowlists for rule ids and cross-workspace target allow/deny.
  - v1 MUST NOT introduce a workflow-specific policy file family; workflow gating keys live under the existing global/workspace `policy.yaml` patch surfaces (ADR-0013).
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
- Define workflow rule file locations + precedence and add schema to ADR-0027 surfaces additively (`workflow.*` keys).
- Define policy gating keys for workflow rule allowlisting and cross-workspace routing allow/deny.

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
  - Workflow-router-derived events are non-triggerable by default to prevent recursion.
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
  - Turns the workflow router into a policy-bypass vector (“trigger on stdout containing …”).
- **Unlocks:**
  - Powerful automation at the cost of safety and auditability.
- **Quick wins / low-hanging fruit:**
  - None that remain safe.

**Recommendation**
- **Selected:** Option A — Strict allowlist.
- **Rationale (crisp):** Ensures the trigger surface is safe-by-default and expands only via explicit, reviewable schema changes.

**Follow-up tasks (explicit)**
- Document the v1 triggerable event type set and the fields available for rule matching.
- Add a recursion guard: workflow-router-emitted events are excluded unless a rule explicitly opts in (future; not v1).

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
- Remote ingress turns the workflow router into an attack surface. v1’s goal is local routing from trace → policy-gated actions without network exposure.

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
- The workflow router is at-least-once by necessity (crashes, restarts, rotation). Without deterministic dedupe, rules can create infinite loops or repeated actions.

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

---

### DR-0011 — Service liveness + single-instance detection (file heartbeat vs control socket)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Operators and service managers need a deterministic way to answer:
  - “Is the workflow router running?”
  - “Is there exactly one instance?”
- DR-0002 constrains v1 away from adding new privileged ingestion RPC; we still need liveness/status semantics.

**Option A — File-based lock + heartbeat (recommended)**
- **Pros:**
  - Stays aligned with the v1 file-based posture (trace tailing + JSONL queues).
  - Works even when no local sockets are available/allowed.
  - Simple cross-platform story: an OS-level exclusive file lock held for process lifetime + periodic heartbeat timestamp.
  - Reuses existing infrastructure: `crates/shell/src/execution/lock.rs` already provides a cross-platform exclusive `ProcessLock` via `fs2`.
- **Cons:**
  - Requires careful “unhealthy vs not-running” semantics:
    - a stale heartbeat does not mean “no instance exists” if the lock is still held.
  - PID reuse edge cases exist; PID is diagnostic only and must not be used as the sole authority.
- **Cascading implications:**
  - The workflow router MUST take an exclusive lock under `SUBSTRATE_HOME/workflow/` at startup.
  - The router MUST update a heartbeat timestamp in `SUBSTRATE_HOME/workflow/state.json` at a fixed cadence.
  - The lock MUST be the single-instance authority:
    - if `substrate workflow status` can acquire the lock non-blocking, the service is not running (regardless of leftover files).
    - if the lock is held, the service is running; heartbeat staleness indicates “unhealthy” not “absent”.
- **Risks:**
  - Clock skew can create false “unhealthy” signals; mitigate by conservative thresholds and by surfacing both lock-held and heartbeat age in diagnostics.
- **Unlocks:**
  - Zero new daemon-facing RPC surfaces in v1.
- **Quick wins / low-hanging fruit:**
  - Reuse the existing atomic-write pattern (temp + rename) for updating `state.json`.

**Option B — Local control socket for status/commands (UDS/NamedPipe)**
- **Pros:**
  - Strong liveness semantics (active handshake).
  - Enables richer commands (dump stats, trigger compaction) later.
- **Cons:**
  - Introduces a new always-on local RPC surface that must be permissioned and documented per platform.
  - Windows NamedPipe vs Unix UDS differences complicate the cross-platform contract.
- **Cascading implications:**
  - Requires an auth/ACL story and a stable API surface (even if local-only).
- **Risks:**
  - Drifts toward “direct publish API” patterns that DR-0002 explicitly avoids for v1.
- **Unlocks:**
  - Future operator tooling, at the cost of additional surface area.
- **Quick wins / low-hanging fruit:**
  - None without designing the API + security posture.

**Recommendation**
- **Selected:** Option A — File-based lock + heartbeat.
- **Rationale (crisp):** Preserves the v1 file-based posture and avoids introducing a new always-on local RPC surface just to answer liveness questions.

**Follow-up tasks (explicit)**
- Define the lock/heartbeat fields and thresholds (including stale-lock handling rules) as part of the `state.json` schema.
- Ensure `substrate workflow status --json` reports: `running`, `pid` (if known), `last_heartbeat_ts`, and `stale_lock_detected`.
  - Additionally report `lock_held` (authoritative) and `health` (`healthy|unhealthy`) derived from heartbeat age.

---

### DR-0012 — Trace follower cursor model (inode+offset vs event-key scan)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router must tail `trace.jsonl` across rotation and restarts without missing or replaying unboundedly.

**Option A — Persist `(inode, byte_offset, rewind_bytes, last_line_hash)` (recommended)**
- **Pros:**
  - Deterministic and fast: continue from a byte offset without rescanning large files.
  - Rotation-safe: inode change signals rename/rotation; size checks detect truncation.
  - `last_line_hash` enables safe “rewind window” replay while suppressing duplicates via DR-0010.
- **Cons:**
  - Requires platform-specific inode equivalents on Windows (file id) or an alternative rotation signal.
- **Cascading implications:**
  - Cursor persistence MUST include a small rewind window (bytes) to handle partial writes and rotation races.
  - On resume, the follower MUST re-read the rewind window and rely on DR-0010 dedupe to bound duplicates.
- **Risks:**
  - If rotation occurs faster than retention keeps `.jsonl.1`, some events can be missed; mitigate via TRACE_LOG size/keep guidance and by logging a “cursor_gap_detected” derived event.
- **Unlocks:**
  - Efficient long-running behavior with bounded overhead.
- **Quick wins / low-hanging fruit:**
  - Use `metadata.len()` + inode/file-id checks to detect rotation/truncation.

**Option B — Event-key-based cursor (store last `cmd_id`/`ts` and rescan)**
- **Pros:**
  - Avoids platform-specific inode/file-id handling.
- **Cons:**
  - Requires rescanning and parsing potentially large portions of trace at every restart.
  - Fragile without a canonical event id: collisions and ordering issues are likely (`cmd_id` is per-span, not per-line event id).
- **Cascading implications:**
  - Would effectively require a stable event id field in ADR-0028 to be robust.
- **Risks:**
  - Performance regressions and missed/doubled events under load.
- **Unlocks:**
  - Simpler implementation only in small traces; not acceptable for v1 reliability targets.
- **Quick wins / low-hanging fruit:**
  - None that remain robust.

**Recommendation**
- **Selected:** Option A — Persist `(inode, byte_offset, rewind_bytes, last_line_hash)`.
- **Rationale (crisp):** Provides efficient, rotation-safe tailing with bounded duplicates via rewind+dedupe, without requiring a new canonical event id in v1.

**Follow-up tasks (explicit)**
- Define cursor fields in `state.json` and add tests for: rotation, truncation, and partial-line writes.
- Emit a derived trace event when a cursor gap is detected (miss risk) so operators can diagnose retention misconfiguration.

---

### DR-0013 — `state.json` schema + versioning (single versioned doc vs multiple files)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router needs durable state for cursors, dedupe, retry counters, and liveness heartbeat.
- The state must be atomic to update and must not grow unboundedly.

**Option A — Single versioned `state.json` (recommended)**
- **Pros:**
  - One atomic write per update cycle (temp + rename).
  - Easy to reason about and inspect (`jq`).
  - Schema versioning can be explicit (`schema_version`), enabling fail-closed parsing.
- **Cons:**
  - Care needed to keep the file small and bounded.
- **Cascading implications:**
  - `state.json` MUST include:
    - `schema_version`
    - `trace_cursor` (DR-0012)
    - `inbox_cursor`, `work_queue_cursor`
    - `dedupe` (bounded window; keys + timestamps)
    - `heartbeat` (DR-0011)
    - bounded retry metadata
  - Compaction MUST prune dedupe keys and retry metadata deterministically.
- **Risks:**
  - If boundedness is not enforced, state growth becomes an operator incident.
- **Unlocks:**
  - Minimal moving parts in v1.
- **Quick wins / low-hanging fruit:**
  - Enforce maximum dedupe key count and maximum age.

**Option B — Split state into multiple files (`cursor.json`, `dedupe.json`, etc.)**
- **Pros:**
  - Potentially smaller individual updates; isolates corruption impact.
- **Cons:**
  - Cross-file consistency becomes a correctness problem (must coordinate updates).
  - More files to manage, document, and permission correctly.
- **Cascading implications:**
  - Requires a transactional story across multiple files or acceptance of partial inconsistency.
- **Risks:**
  - Harder to reason about recovery; more edge cases.
- **Unlocks:**
  - Future specialization, at the cost of v1 complexity.
- **Quick wins / low-hanging fruit:**
  - None without a transaction/locking design.

**Recommendation**
- **Selected:** Option A — Single versioned `state.json`.
- **Rationale (crisp):** Keeps correctness and atomicity straightforward in v1 while remaining inspectable and fail-closed via explicit schema versioning.

**Follow-up tasks (explicit)**
- Define the exact `state.json` schema (fields + bounds) and add strict parsing tests.
- Implement deterministic compaction/pruning rules and tests that assert bounded file size over time.

---

### DR-0014 — Queue item immutability + ack model (cursor-based vs in-queue status mutation)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Requests/actions must be durable and crash-safe while preserving auditability.

**Option A — Append-only queues + cursor-based acknowledgement (recommended)**
- **Pros:**
  - Preserves append-only durability guarantees (simple to implement safely).
  - Makes queue inspection easy (`tail`, `jq`).
  - Status transitions are represented as derived trace events (DR-0003), not as mutable queue records.
- **Cons:**
  - Requires careful cursor persistence and bounded rewind logic.
- **Cascading implications:**
  - `inbox.jsonl` and `work_queue.jsonl` records are immutable.
  - A “processed” item is acknowledged by advancing a durable cursor in `state.json`.
  - Failures are represented by derived trace events plus bounded retry metadata (DR-0015).
- **Risks:**
  - Without compaction, JSONL can grow; mitigate via periodic compaction rules tied to cursors.
- **Unlocks:**
  - Strong audit posture: the queues are durable intent streams and trace is the outcome stream.
- **Quick wins / low-hanging fruit:**
  - Implement “compaction = rewrite remaining tail after cursor” guarded by a lock.

**Option B — Mutable queue items (in-place status updates / tombstones)**
- **Pros:**
  - Easier to query “what is pending vs done” without joining against trace.
- **Cons:**
  - Harder to make crash-safe across platforms; introduces partial-write corruption risk.
  - Complicates audit posture: the queue becomes a state machine rather than an intent log.
- **Cascading implications:**
  - Requires transactional writes or sqlite (conflicts with DR-0004).
- **Risks:**
  - Silent corruption of queue state is more likely and harder to repair.
- **Unlocks:**
  - Richer local dashboards, at the cost of v1 correctness.
- **Quick wins / low-hanging fruit:**
  - None without transactional storage.

**Recommendation**
- **Selected:** Option A — Append-only queues + cursor-based acknowledgement.
- **Rationale (crisp):** Keeps durability simple and auditable in v1 while avoiding mutable state machines in JSONL.

**Follow-up tasks (explicit)**
- Define queue record schemas (request/action) and cursor semantics in the feature specs.
- Implement compaction rules and add tests that confirm no data loss when compacting after cursor advancement.

---

### DR-0015 — Retry/backoff + rate limiting (bounded retries vs no retries)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router will encounter transient failures (world backend down, workspace missing, policy unavailable).
- Without explicit retry and rate limits, it can either silently drop work or spam repeated attempts.

**Option A — Bounded retries with backoff + per-rule throttles (recommended)**
- **Pros:**
  - Improves reliability for transient failures while bounding repeated side effects.
  - Rate limits prevent event storms from turning into request storms.
- **Cons:**
  - Adds implementation complexity: retry counters, backoff schedule, and throttle windows.
- **Cascading implications:**
  - `state.json` MUST track per-item retry counts and next-eligible-at timestamps (bounded).
  - The router MUST implement per-rule and per-workspace rate limits (default conservative).
  - Exhausted retries MUST emit a derived trace event and stop retrying.
- **Risks:**
  - Misconfigured backoff can delay work too much; mitigate via strict defaults and diagnostics in `workflow status`.
- **Unlocks:**
  - “Always-on” behavior that is resilient, not noisy.
- **Quick wins / low-hanging fruit:**
  - Fixed max attempts + exponential backoff with cap.

**Option B — No retries (fail once; require manual replay)**
- **Pros:**
  - Simpler implementation.
  - Avoids any risk of repeated side effects.
- **Cons:**
  - Poor operator UX for transient failures; encourages ad-hoc manual intervention.
  - The “always-on” service becomes unreliable for common outage modes (world socket restart, temporary policy parse failure).
- **Cascading implications:**
  - Requires separate tooling for manual replay/reenqueue workflows.
- **Risks:**
  - Operators treat it as broken under transient failures.
- **Unlocks:**
  - None; correctness does not require “no retries”, bounded retries achieve the same safety posture.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Bounded retries with backoff + per-rule throttles.
- **Rationale (crisp):** Makes the always-on router resilient to transient failures while bounding repetition via strict retry caps and throttles.

**Follow-up tasks (explicit)**
- Define default retry/backoff parameters and rate limits, and surface them in `workflow status --json`.
- Add tests for: retry exhaustion, throttle enforcement, and “no busy-loop” guarantees.

---

### DR-0016 — Derived event taxonomy (explicit event types vs single generic wrapper)

**Decision owner(s):** Trace + Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Derived workflow-router events must be stable and easy to filter/debug.

**Option A — Small explicit set of `event_type` values (recommended)**
- **Pros:**
  - Easy to filter with `jq` and reason about semantically.
  - Encourages stable required fields per event type.
- **Cons:**
  - Requires additive updates when new lifecycle stages are added.
- **Cascading implications:**
  - v1 event types MUST include:
    - `workflow_router_rule_match`
    - `workflow_router_request_enqueued`
    - `workflow_router_request_denied`
    - `workflow_router_request_pending_approval`
    - `workflow_router_action_enqueued`
    - `workflow_router_action_executed`
    - `workflow_router_cursor_gap_detected`
  - Each derived event MUST include correlation keys:
    - `session_id` (required on all canonical trace records; see ADR-0028 Phase 8 additive correlation vocabulary)
    - a cause reference:
      - `source_span_id` when available (preferred), and/or
      - `source_cmd_id`
    - `rule_id`
    - `workspace_id`
    - `idempotency_key`
- **Risks:**
  - Poorly scoped event types could explode; mitigate by limiting v1 to the set above.
- **Unlocks:**
  - Stable operational tooling and replay correlation.
- **Quick wins / low-hanging fruit:**
  - Document the required fields per event type alongside ADR-0028’s schema conventions.

**Option B — Single `workflow_router_event` with `subtype` field**
- **Pros:**
  - Fewer top-level event types.
- **Cons:**
  - Makes filtering and alerting harder (must inspect payload).
  - Risks “schema drift” inside a generic envelope.
- **Cascading implications:**
  - Requires a robust subtype registry and schema validation anyway.
- **Risks:**
  - Becomes a dumping ground; weakens the “singular and testable” contract.
- **Unlocks:**
  - None in v1.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Small explicit set of `event_type` values.
- **Rationale (crisp):** Keeps derived events filterable and semantically stable with clear required fields per lifecycle stage.

**Follow-up tasks (explicit)**
- Add the derived event types to the trace schema docs (ADR-0028 circle-back) as additive families.
- Add tests that assert derived events never include sensitive payloads and always include required correlation keys.

---

### DR-0017 — Policy gating key paths for workflow router (dedicated workflow keys vs reuse generic cmd gating)

**Decision owner(s):** Broker + Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router introduces a new “indirect execution” pathway (event → request → action). This must be explicitly policy-gated, fail-closed, and explainable.

**Option A — Dedicated `workflow.router.*` policy keys (recommended)**
- **Pros:**
  - Explicit and auditable authorization surface for indirect execution.
  - Enables narrow allowlisting (rule ids, target workspaces, action kinds).
- **Cons:**
  - Requires additive policy schema work and docs updates.
- **Cascading implications:**
  - v1 policy keys MUST live under existing `policy.yaml` patch surfaces (no new workflow policy files; see DR-0006).
  - Policy must be able to deny:
    - cross-workspace routing
    - specific rule ids
    - specific action kinds
  - v1 policy keys (exact; patch-only under global/workspace `policy.yaml` per ADR-0013):
    - `workflow.router.enabled` (bool; default `false`)
    - `workflow.router.allow_cross_workspace` (bool; default `false`)
    - `workflow.router.allowed_rule_ids` (list of rule ids; default `[]`)
    - `workflow.router.allowed_workflow_ids` (list of workflow ids; default `[]`)
    - `workflow.router.allowed_target_workspace_ids` (list of workspace UUIDs; default `[]`)
- **Risks:**
  - Poorly designed keys become too permissive; mitigate by fail-closed defaults (router disabled unless explicitly enabled).
- **Unlocks:**
  - Safe automation without conflating “direct commands” with “indirect triggers”.
- **Quick wins / low-hanging fruit:**
  - Start with `workflow.router.enabled=false` default and explicit allowlists.

**Option B — Reuse generic `cmd_allowed/cmd_denied/cmd_isolated` only**
- **Pros:**
  - No new policy schema.
- **Cons:**
  - Indirect execution is not representable cleanly as a command string allow/deny.
  - Loses explainability (“why was this workflow triggered allowed?”) and makes cross-workspace gating awkward.
- **Cascading implications:**
  - Encourages brittle encodings (stuffing rule ids into fake command strings).
- **Risks:**
  - Operators accidentally permit indirect execution broadly while trying to allow a single workflow.
- **Unlocks:**
  - None; this is an ergonomics trap.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Dedicated `workflow.router.*` policy keys.
- **Rationale (crisp):** Gives a crisp, auditable allowlist surface for indirect execution and cross-workspace routing with fail-closed defaults.

**Follow-up tasks (explicit)**
- Define `--explain` provenance output for router decisions (including which policy keys contributed to allow/deny).
- Ensure deny reasons are emitted as derived trace events with a stable machine-readable code.

---

### DR-0018 — Workspace registry drift rules (fail-closed vs auto-repair)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The router relies on both:
  - `SUBSTRATE_HOME/workspaces/registry.json` (known workspaces), and
  - `<workspace_root>/.substrate/workspace_id` (workspace identity).
- These can drift (path moves, registry edits, stale entries).

**Option A — Fail-closed on drift (recommended)**
- **Pros:**
  - Preserves security posture: no accidental cross-workspace execution due to silent “best effort” resolution.
  - Makes drift visible and actionable (explicit errors + derived trace events).
- **Cons:**
  - Requires operators to run repair commands when drift occurs.
- **Cascading implications:**
  - If registry says workspace_id=X at path P but `P/.substrate/workspace_id != X`, the router MUST refuse to route to P.
  - The router MUST emit a derived trace event describing the drift and suggesting `substrate workspace init|enable|disable` repair flows.
- **Risks:**
  - More “hard errors” in early adoption; acceptable for safety.
- **Unlocks:**
  - Deterministic and safe cross-workspace routing.
- **Quick wins / low-hanging fruit:**
  - Implement a `workflow doctor` check that scans registry roots and validates `workspace_id` files.

**Option B — Auto-repair drift (prefer workspace file or registry)**
- **Pros:**
  - Smoother operator UX in some cases (path move “just works”).
- **Cons:**
  - Hidden mutations to registry or workspace metadata are risky and hard to audit.
  - Auto-repair can be exploited if an attacker can manipulate workspace paths.
- **Cascading implications:**
  - Requires a complete policy story for when auto-repair is permitted.
- **Risks:**
  - Silent cross-workspace misrouting.
- **Unlocks:**
  - Convenience, at the cost of safety.
- **Quick wins / low-hanging fruit:**
  - None without a security review.

**Recommendation**
- **Selected:** Option A — Fail-closed on drift.
- **Rationale (crisp):** Keeps cross-workspace routing deterministic and safe by making identity drift an explicit error rather than a silent “fixup”.

**Follow-up tasks (explicit)**
- Define drift diagnostics and stable error codes for: stale path, missing workspace_id file, mismatched workspace_id.
- Add tests that confirm drift causes deny/failure and produces derived trace events.

---

### DR-0019 — Approval handling for router-triggered requests (queue + wait vs treat-as-deny)

**Decision owner(s):** Broker + Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- The ADR states: “If policy requires approval, record pending approval (approval mechanism defined elsewhere).”
- v1 must define what the router does operationally when an approval gate is hit.

**Option A — Record `pending_approval` and hold the request (recommended)**
- **Pros:**
  - Preserves fail-closed posture: no execution until approval is granted.
  - Maintains a durable audit trail: the request remains visible and can be approved later.
- **Cons:**
  - Requires an approval “resume” mechanism (even if minimal in v1).
- **Cascading implications:**
  - The router MUST emit `workflow_router_request_pending_approval` and MUST NOT enqueue an action until approval is granted.
  - The request remains in `inbox.jsonl` with retry disabled while pending approval.
- **Risks:**
  - If approvals are never processed, inbox can accumulate; mitigate via status/doctor warnings and operator tooling.
- **Unlocks:**
  - Safe “human-in-the-loop” workflows.
- **Quick wins / low-hanging fruit:**
  - Surface “pending approvals count” in `workflow status --json`.

**Option B — Treat “requires approval” as deny for automation**
- **Pros:**
  - Simple: no pending state.
- **Cons:**
  - Violates operator intent (“I want approval, not deny”) and makes approvals unusable for router flows.
  - Encourages operators to disable approvals to make workflows work (bad safety tradeoff).
- **Cascading implications:**
  - Must emit deny events for approvals, conflating two distinct outcomes.
- **Risks:**
  - Pushes unsafe defaults.
- **Unlocks:**
  - None; this is a safety regression.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Record pending approval and hold the request.
- **Rationale (crisp):** Preserves fail-closed execution while enabling durable, auditable human approvals for router-triggered workflows.

**Follow-up tasks (explicit)**
- Define the minimal v1 approval resume path (CLI or broker API) that can transition a pending request to actionable.
- Add tests that confirm pending-approval requests do not execute and survive restarts.

---

### DR-0020 — v1 action kinds (workflow-only vs general-purpose “run anything”)

**Decision owner(s):** Shell + Workflow maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`, `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- The router creates “actions” from “requests”. If action kinds are too generic, the router becomes an indirect command-execution engine.

**Option A — v1 supports only `workflow.run` actions (recommended)**
- **Pros:**
  - Keeps the router narrowly scoped to the workflow system.
  - Avoids turning routing rules into an indirect arbitrary command runner.
  - Simplifies policy gating (`workflow.router.allowed_workflows[*]`, etc.).
- **Cons:**
  - Less flexible for non-workflow automation (intentionally out of scope).
- **Cascading implications:**
  - Action schema MUST include:
    - `action_kind = "workflow.run"`
    - `workflow_id` (positional arg) + optional args
  - Any future non-workflow action kinds require explicit additive DRs and policy keys.
- **Risks:**
  - Operators ask for “run a command on trigger”; must be declined in v1.
- **Unlocks:**
  - Safe baseline aligned to Phase 7 workflow execution model.
- **Quick wins / low-hanging fruit:**
  - Implement a single action executor path.

**Option B — General-purpose actions (arbitrary command execution)**
- **Pros:**
  - Maximum flexibility.
- **Cons:**
  - Massive security/policy surface expansion.
  - Hard to make safe, deterministic, and explainable.
- **Cascading implications:**
  - Would require extensive policy language and redaction rules for payloads.
- **Risks:**
  - Becomes an audit-friendly “remote code execution” primitive on the host.
- **Unlocks:**
  - Convenience, at a cost that is not acceptable for v1.
- **Quick wins / low-hanging fruit:**
  - None that remain safe.

**Recommendation**
- **Selected:** Option A — v1 supports only `workflow.run` actions.
- **Rationale (crisp):** Keeps the router bounded to workflow orchestration and avoids an indirect arbitrary execution surface in v1.

**Follow-up tasks (explicit)**
- Define the v1 `workflow.run` action schema (required fields + caps) and the matching policy allowlist keys.
- Add tests that reject unknown `action_kind` values fail-closed.
