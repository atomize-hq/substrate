# ADR-0029 — Host Event Bus + Router Daemon (Trace-Driven Triggers, Cross-Workspace Requests)

## Status
- Status: Draft
- Date (UTC): 2026-02-05
- Owner(s): Spenser McConnell (Substrate); Shell maintainers

## Scope
- Feature directory: `docs/project_management/next/host_event_bus_router_daemon/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Trace/event foundations:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- FS path semantics & allow/deny matching:
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Config/policy layering model:
  - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
- Future alignment (not required to land this ADR):
  - `docs/project_management/next/world-sync/` (internal git support; not yet landed)

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- Substrate gains an always-on host router that can trigger policy-gated actions from trace events (including cross-workspace)
  - Existing: Substrate records trace events (`~/.substrate/trace.jsonl`), but there is no always-on host service that can “listen” for specific events and route them into follow-on work.
  - New: A host daemon tails the canonical trace stream and (optionally) produces policy-gated requests/actions, including cross-workspace routing using an explicit workspace registry under `SUBSTRATE_HOME`.
  - Why: Enable reliable “when A completes, trigger B” workflows and selective file-change triggers without introducing an external broker or bypassing workspace policy boundaries.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md#L1`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md#L1`
    - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md#L1`

## Problem / Context
- Substrate is moving toward multi-agent, multi-workspace orchestration. We need a reliable host-side mechanism to:
  - listen for specific events (command completion, workflow completion, selected fs diffs),
  - route them into follow-on work (often in a different workspace),
  - ensure all follow-on execution is policy-gated under the target workspace’s effective policy/config.
- We want this without:
  - introducing an external broker (Kafka/RabbitMQ) as a hard dependency,
  - inventing a second “event schema” that diverges from trace,
  - letting remote inputs bypass world/policy constraints.

## Goals
- Use `trace.jsonl` as the canonical local event log and primary subscription source.
- Implement an always-on **host daemon** (“router”) that:
  - tails `trace.jsonl` using durable cursors,
  - matches events against global + workspace-scoped routing rules,
  - produces durable, policy-gated requests/actions.
- Ensure the router daemon is a host service (not world-agent) so it remains available when worlds are disabled or when a VM/WSL backend is down.
- Support cross-workspace routing by introducing an explicit workspace registry under `SUBSTRATE_HOME` updated by `substrate workspace init|enable|disable`.
- Support selective file-operation triggers based on Substrate-collected fs diffs, scoped to specific file paths/directories (workspace-relative matching).

## Non-Goals
- Replacing the trace system: trace remains canonical.
- Building a general-purpose external message broker (Kafka/RabbitMQ) into Substrate.
- Triggering on out-of-band filesystem changes not mediated by Substrate execution (v1 triggers rely on Substrate-produced fs diffs; future work may add git-backed feeds or watchers).
- Exposing an Internet-accessible webhook gateway in v1 (remote ingress is future; v1 focuses on host daemon + local queues).

## User Contract (Authoritative)

### Terminology
- **Event**: an immutable record appended to `trace.jsonl`.
- **Derived trigger event**: a bus-produced trace event indicating a rule match / routing decision.
- **Request**: a durable “intent to act” object produced by the bus, evaluated under target workspace policy before execution.

### Files and locations (Authoritative)
All bus state is stored under `SUBSTRATE_HOME` (default `~/.substrate`):

- Canonical event log (existing):
  - `SUBSTRATE_HOME/trace.jsonl`
- Router daemon state (new):
  - `SUBSTRATE_HOME/bus/state.json` (durable cursor + dedupe metadata)
  - `SUBSTRATE_HOME/bus/inbox.jsonl` (durable inbound requests; local-only in v1)
  - `SUBSTRATE_HOME/bus/work_queue.jsonl` (durable queued actions derived from requests)
- Workspace registry (new):
  - `SUBSTRATE_HOME/workspaces/registry.json` (authoritative list of known workspaces and IDs)

### Workspace identity (Authoritative)
- Each workspace MUST have a stable `workspace_id`.
- Default algorithm (v1): `workspace_id = sha256(canonical_workspace_root_path)` encoded as lowercase hex.
- The registry MUST store:
  - `workspace_id`
  - `workspace_root` (canonical absolute path)
  - `enabled` boolean (mirrors `workspace.disabled` behavior)
  - optional `label`

### Rule scoping and precedence (Authoritative)
- Rules exist at:
  - global scope (loaded from `SUBSTRATE_HOME`), and
  - workspace scope (loaded from the target workspace root).
- Rule evaluation precedence for a given event:
  1. workspace-scoped rules (for the owning workspace, when applicable)
  2. global rules (fallback)

### Trigger taxonomy (Authoritative)
Only an explicit allowlist of event families is triggerable. v1 supports:
- Execution completion events (root span completion / command completion)
- Workflow/agent lifecycle completion events (once available in trace)
- Filesystem diff-derived events:
  - derived trigger inputs from fs diffs indicating create/modify/delete/rename of workspace-relative paths
  - path matching MUST reuse the same workspace-relative semantics and matcher behavior as ADR-0018

### File operation triggers (Authoritative)
- File triggers MUST be derived from Substrate-produced fs diffs (not OS filesystem watching in v1).
- Triggers MUST support include/exclude path matching using workspace-relative paths:
  - exact file match
  - subtree match
  - pattern/glob match (as defined by ADR-0018)

### Policy gating (Authoritative)
- A trigger match MUST NOT execute work directly.
- Instead, it creates a durable **request** that is evaluated under the **target workspace’s** effective config + effective policy.
- If policy denies the requested action, the request MUST be recorded as denied with an explainable reason and MUST NOT execute.
- If policy requires approval, the request MUST be recorded as pending approval (approval mechanism defined elsewhere).

### Cross-workspace routing (Authoritative)
- A rule may route a request from an event in workspace A to a target workspace B.
- The target MUST be resolved via `workspace_id` (path may be used only as a fallback / debug surface).
- The bus MUST re-resolve effective config/policy for workspace B at execution time (not reuse workspace A’s).

### Daemon behavior (Authoritative)
- The router daemon MUST be host-level and MUST run independently of world-agent availability.
- It MUST degrade gracefully:
  - if it cannot read `trace.jsonl`, it does not lose cursor state and retries,
  - if it cannot resolve a target workspace, it records a failed request and continues.

### Request queue semantics (Authoritative)
- The bus uses durable JSONL queues under `SUBSTRATE_HOME/bus/`:
  - `inbox.jsonl` stores **requests** (durable “intent to act”).
  - `work_queue.jsonl` stores **actions** derived from requests after routing/policy evaluation.
- Processing semantics:
  - Handling is at-least-once; duplicate processing MUST be bounded via dedupe keys.
  - Each request/action MUST have a stable idempotency key derived from:
    - the source event identity (e.g., `span_id` + `event_type` + rule_id), and
    - the target workspace identity (`workspace_id`).
  - The bus MUST persist per-subscriber cursors and dedupe state in `bus/state.json` so restarts do not replay unboundedly.

### CLI (minimal; may be extended)
- `substrate bus status [--json]`
  - Behavior: reports whether the daemon is running, the current trace cursor, and queue sizes.
  - Exit codes: `0` success; `4` daemon not available.
- `substrate bus workspaces list [--json]`
  - Behavior: prints registry entries (workspace_id, root, enabled).
  - Exit codes: `0` success; `3` registry read/parse failure.

## Architecture Shape
- Host daemon:
  - tails `SUBSTRATE_HOME/trace.jsonl` with durable cursor state
  - applies routing rules (workspace + global)
  - writes requests to inbox / work_queue
  - emits derived trace events for:
    - rule match
    - request enqueued
    - request denied / pending approval / executed
- Service boundary:
  - the daemon is a host service (not in-world) and must remain available in host-only mode.
  - it MAY reuse transport patterns and code organization from world-agent, but must not depend on world-agent being available.
- Event recursion guard:
  - bus-emitted events MUST be identifiable (e.g., `component=busd`) and MUST be excluded from re-trigger evaluation by default to avoid infinite loops.
- FS trigger derivation:
  - the bus derives file-change trigger inputs from fs diff events already persisted to trace, and applies ADR-0018 matching semantics for include/exclude.

## Sequencing / Dependencies
- Prerequisites:
  - ADR-0028 must land first (trace schema/correlation + redaction requirements; span parent correctness).
  - ADR-0017 must be stable (structured output vs PTY bytes; routing attribution).
  - ADR-0018 path semantics must be treated as authoritative for fs-trigger matching.
- Follow-on alignment:
  - World-sync/internal git support can provide higher-quality change classification later, but is not required for v1.

## Security / Safety Posture
- Fail-closed rules:
  - No bus-triggered execution may occur without policy evaluation under the target workspace.
  - File triggers must not become an exfil channel: only path metadata required for routing is used by default; content is not used.
- Protected paths/invariants:
  - All bus state lives under `SUBSTRATE_HOME` with user-only permissions.
  - Requests are durable and auditable; every request produces an observable trace record of allow/deny/approval outcomes.

## Open Questions (Draft-only; resolve before `Accepted`)

This section intentionally captures unresolved decisions and implementation-level constraints discovered during discovery. It must be emptied (migrate A/B items into `decision_register.md`) before the ADR status flips to `Accepted`.

### 1) Daemon packaging + lifecycle (host service)
- Service manager targets:
  - Linux: systemd service (optionally socket-activated for the control plane)
  - macOS: launchd agent/daemon
  - Windows: service (or scheduled task) with equivalent “always-on” semantics
- Open questions:
  - Should the router daemon be a standalone binary (e.g., `substrate-busd`) or a subcommand mode of `substrate` (e.g., `substrate busd --foreground`)?
  - Should it be socket-activated (wake on CLI/control requests) while still tailing trace, or strictly long-running?
  - What is the “single-instance” lock strategy under `SUBSTRATE_HOME` (lock file vs OS-level mutex), and what is the operator UX when a stale lock is detected?

### 2) Event source: trace tailing vs direct RPC emits
- Current decision: `trace.jsonl` is canonical.
- Open questions:
  - Does the bus exclusively consume trace by tailing the file, or do core components also publish “bus requests” directly over a local API (UDS), with trace as the audit sink?
    - Tail-only is simplest and keeps coupling low.
    - Direct publish reduces latency and avoids “tailer correctness” edge cases, but adds another API surface that must be secured.
  - If tail-only: what is the required cursor correctness model (byte offset vs line count vs event_id-based cursor)?

### 3) Request queue and “intent” semantics (durability + retries)
- Current decision: durable queues live under `SUBSTRATE_HOME/bus/`:
  - `inbox.jsonl` = durable **requests** (intent to act)
  - `work_queue.jsonl` = durable **actions** derived after routing/policy evaluation
  - `state.json` = cursors + dedupe + subscriber state
- Open questions:
  - Is JSONL sufficient for v1, or do we want sqlite from day 1 to simplify:
    - dedupe windows,
    - “claim/lease” semantics,
    - retries with backoff,
    - and crash-safe acknowledgements?
  - Exactly what is the “ack model”?
    - append-only queue with cursor advancement after success,
    - vs per-item ack record (more robust for reordering and partial failure).
  - Should requests/actions be immutable (append-only) with status updates emitted as trace events, or should the queue entries also include status transitions?

### 4) Workspace registry authority + update semantics
- Current decision: registry is explicit and owned by `substrate workspace init|enable|disable`.
- Open questions:
  - Should the router daemon accept “discovered workspaces” (scan for `.substrate/workspace.yaml`) as a fallback, or only trust the explicit registry?
  - What is the expected behavior if `workspace_root` moves?
    - If `workspace_id` is hash-of-path, move implies ID changes and requires migration.
    - If `workspace_id` can be stored inside `workspace.yaml`, move can preserve identity.
  - How do we handle:
    - duplicate entries,
    - stale paths,
    - and disable/enable propagation (workspace.disabled marker vs registry `enabled=false`)?

### 5) Workspace ID strategy (stability vs simplicity)
- Current draft default: `workspace_id = sha256(canonical_workspace_root_path)`.
- Open questions:
  - Should we add an explicit ID field inside `workspace.yaml` now to avoid later breaking changes?
  - If explicit ID exists, which source wins when registry and workspace.yaml disagree?
  - Do we want a “label” or “alias” system for operators (human-friendly names) and how does that intersect with security (spoofing/misdirection risk)?

### 6) Cross-workspace routing semantics (the “A triggers B” model)
- Current decision: cross-workspace triggers create **requests** that are re-evaluated under the target workspace’s effective config/policy.
- Open questions:
  - What is the minimum request schema for a cross-workspace trigger?
    - required: `source_event_ref` (span_id/event id), `rule_id`, `target.workspace_id`, `action_kind`, `payload`
  - How do we correlate the resulting execution back to the originating event?
    - required: a stable `cause_id` or `trigger_event_ref` field appended into new trace records
  - Do we allow fan-out (one event triggers many targets), and what rate limits apply?
  - Should cross-workspace triggers be allowed by default, or require explicit policy allowlisting (recommended)?

### 7) Trigger taxonomy: “what is triggerable?”
- Current direction: explicit allowlist of triggerable event families.
- Open questions:
  - What is the v1 minimal trigger set?
    - `command_complete` / root span completion
    - `workflow_node_complete` / `agent_run_complete` (once present)
    - fs diff-derived `fs_change` events (create/modify/delete/rename)
  - How do we ensure we never accidentally make “sensitive” event families triggerable (e.g., raw stdout chunks, PTY bytes, env dumps)?
  - How do we handle event recursion:
    - should bus-emitted events be non-triggerable by default, or triggerable only under an explicit “allow recursion” rule?

### 8) File operation triggers (path-scoped, diff-derived)
- Current direction: file triggers are derived from Substrate-produced fs diffs and use ADR-0018 path semantics (workspace-relative allow/deny matching).
- Open questions:
  - Which diff source is authoritative for triggers:
    - world-agent returned diffs only,
    - host diffs (if any),
    - or both?
  - How do we represent big diffs (dependency installers) safely?
    - per-path events vs batched summary event with counts and a truncated path sample
  - How do we define “modified” in a stable way for triggers?
    - metadata-only (mtime/size/hash) vs content hash (expensive)
  - Should triggers support “only when file lands in world overlay” vs “host sync landed” (ties into `world-sync` future work)?

### 9) Where do rules live: config vs policy (and what is the gating model)?
- Open questions:
  - Should routing rules be declared in config (behavior selection) but require policy permission to execute?
  - Or should routing rules themselves live in policy (because they are execution-enabling)?
  - How do profiles (ADR-0020) interact with bus rules:
    - can a profile fully pin bus behavior by supplying a complete “rules snapshot”?

### 10) Remote ingress (future) and threat model
- Current direction: v1 is local-only; remote ingress is future.
- Open questions:
  - Should remote ingress write only to `inbox.jsonl` (requests), never directly to `work_queue.jsonl`?
  - What auth model is acceptable (mTLS vs token-based), and where are secrets stored (do not store in config.yaml)?
  - What is the policy surface for remote ingress allow/deny and rate limiting?

### 11) Trace “landing items” and classification (circle-back alignment with ADR-0028)
- We expect the router daemon, LLM gateway, agent hub, and workflow engine to add new event families and correlation fields over time.
- Open questions:
  - Should derived trigger events be appended into `trace.jsonl` (single canonical log) or stored in `bus/derived.jsonl` and summarized into trace?
  - Which correlation fields should be standardized early (likely additive-only):
    - `cause_id` / `trigger_event_ref`
    - `workflow_node_id`
    - `agent_id` / `role`
    - `tool_call_id`
  - Which of these belong in shared `log_schema.rs` constants vs feature-local payloads?

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - cursor persistence and idempotency/dedupe behavior (at-least-once handling without runaway repeats)
  - workspace registry read/write behavior and `workspace_id` stability
  - path matching semantics are identical to ADR-0018 matcher behavior
- Integration tests:
  - cross-workspace routing: workspace A event produces request targeting workspace B and evaluates under B policy
  - file triggers: only configured include paths trigger (deny all other changes)

### Manual validation
- Manual playbook: `docs/project_management/next/host_event_bus_router_daemon/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/host_event_bus_router_daemon/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/host_event_bus_router_daemon/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/host_event_bus_router_daemon/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/host_event_bus_router_daemon/decision_register.md`:
    - DR-0001 (Derived bus events location: append to `trace.jsonl` vs separate `bus/derived.jsonl`)
    - DR-0002 (Workspace ID source: hash of canonical path vs explicit ID stored in `workspace.yaml`)
    - DR-0003 (Rule storage: config vs policy; and global vs workspace file locations)
    - DR-0004 (Remote ingress: whether/when to accept authenticated inbound requests and how to gate them)
    - DR-0005 (FS trigger source: fs diffs only vs adding watchers/git-backed feeds and the rollout order)
    - DR-0006 (Queue format: JSONL-only vs sqlite; and how cursors/dedupe are persisted)
    - DR-0007 (Idempotency key strategy: derived key recipe and which event families are eligible for deterministic dedupe)
