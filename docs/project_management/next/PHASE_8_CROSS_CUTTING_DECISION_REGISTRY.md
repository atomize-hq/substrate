# Phase 8 — Cross-Cutting Decision Registry (LLM/Agents/Workflows/Router)

This document is the Phase 8 “circle-back” registry for cross-cutting contracts that span:

- Trace foundations: ADR-0028 (`docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`)
- Output/event routing: ADR-0017 (`docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`)
- Config/policy surface: ADR-0027 (`docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`)
- LLM gateway + engines: ADR-0023 / ADR-0024
- Agent hub + toolbox: ADR-0025 / ADR-0026
- Host router daemon: ADR-0029
- (Deferred, but must remain compatible): workflow-engine + forge (ADR-0021 / ADR-0022)

Non-negotiable (Phase 8 constraint): updates to foundation ADRs that are `Accepted` must be additive-only (no contract reshapes). This registry is intended to surface “what must be aligned before acceptance” and “what must be added additively during the circle-back pass”.

---

## Phase 8 checklist mapping (from `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md`)

This registry covers:

- ADR-0028 circle-back: new event families + correlation fields + derived router events + redaction/caps notes + `docs/TRACE.md` updates.
- Secrets delivery channel rubric: when to prefer inherited one-time FD/pipe vs env var injection.
- ADR-0017 circle-back: finalize correlation field set, buffering/backpressure compatibility, optional routing hints (channel/topic), and world session reuse attribution (`world_id`).
- ADR-0027 circle-back: ensure backend id formats + gating keys remain aligned and fail-closed; keep it limited to selection/allowlists and reference appendices.
- ADR-0025 circle-back: explicit control-plane vs event-plane separation; event-plane subscription/channel model.
- ADR-0029 circle-back: v1 trigger allowlist alignment + derived request/event schemas with stable join keys.

---

## Current alignment snapshot (what already composes cleanly)

- Backend id format is consistent across LLM + agents: `<kind>:<name>` (e.g., `cli:codex`) and is used for allowlists and routing selection.
- Default posture is fail-closed / deny-by-default via empty allowlists and `fail_closed.routing` (policy-owned boundary decision).
- Output classes separation (PTY bytes vs structured events) is established, and buffering is explicitly bounded (buffer + drop-with-summary), keeping TUI correctness non-negotiable.
- The workflow-router service is trace-first (tails `trace.jsonl`) and emits derived events append-only (no parallel “event plane” is introduced).

## Primary cross-cutting risks (what can drift without a Phase 8 lock)

- Correlation vocabulary/matrix must remain singular and authoritative (risk: heuristic joins and event-family drift if downstream docs diverge from ADR-0028).
- `agent_id` semantics must remain unified across trace spans vs structured agent events (risk: audit confusion if emitters drift and `backend_id` is inferred heuristically).
- Control plane vs event plane separation is now locked, but future mutating control-plane surfaces must preserve fail-closed gates and must not reintroduce “second execution plane” drift.
- Toolbox/tool-call trace family is now locked; future toolbox expansions must preserve deterministic `tool_call_id` joinability and safe-by-default payload posture (no raw secrets in trace by default).
- Secrets delivery mechanisms are now standardized, but new secret surfaces must reference the rubric and keep “no secret persistence” + redaction/caps invariants intact (risk: env var proliferation and inconsistent hardening).

---

## Phase-by-phase validation findings (Phase 8 circle-back; 2026-02-13)

This section records a quick validate/invalidated snapshot of the `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md` phases, strictly focusing on cross-cutting alignment risks.

- Phase 0 (No rewrite rules): **Validated** (planning order states additive-only once Accepted).
- Phase 1 (Trace foundation / ADR-0028): **Validated** — Phase 8 additive correlation vocabulary + required/optional matrix exists in ADR-0028, including router-derived families/keys and reserved workflow/toolbox identifiers (see CC-0001/CC-0007/CC-0008/CC-0009).
- Phase 2 (Output/routing / ADR-0017): **Validated** — ADR-0017 now includes an explicit structured agent event envelope section aligned to DR-defined envelope extensions (`backend_id`, conditional `world_id`, optional `channel`) and Phase 8 operator-verifiable world lifecycle alerts (see CC-0003/CC-0004/CC-0010).
- Phase 3 (Config/policy surface / ADR-0027): **Validated** — router policy gating keys (`workflow.router.*`) required by ADR-0029 are represented in ADR-0027 schema/contract outputs (see CC-0011).
- Phase 4 (LLM gateway + engines / ADR-0023/ADR-0024): **Validated** — gateway/engine ADRs explicitly defer logging/attribution and correlation vocabulary to ADR-0028 + ADR-0017 (no heuristic joins; `backend_id` is explicit; `channel` remains a routing hint only).
- Phase 5 (Agent hub + toolbox / ADR-0025/ADR-0026): **Validated** — control-plane vs event-plane separation is explicitly locked (Agent Hub + toolbox), and toolbox tool-call trace families/required join keys are specified so control-plane activity is auditable and joinable without heuristics (see CC-0005/CC-0009).
- Phase 6 (Router daemon / ADR-0029): **Validated** — router DRs define derived event taxonomy and required correlation keys, and ADR-0028/`docs/TRACE.md` list the derived families and match the router’s explicit cause-reference naming (see CC-0007/CC-0012).
- Phase 7 (Workflow composition / ADR-0021/ADR-0022): **Validated as deferred** — remains Draft and must stay compatible with reserved workflow correlation keys in ADR-0028; do not lock additional workflow fields beyond accepted DR items unless explicitly called for.
- Phase 8 (Circle-back registry): **In progress** — cross-cutting contract alignment items are now locked in docs; remaining work is implementation + ongoing drift prevention (keep downstream specs/DRs deferring to ADR-0028/ADR-0017 rather than re-stating correlation rules).

---

## Registry (cross-cutting alignment items)

Each item below is written as: **Decision/contract to lock**, **current sources**, **gap**, and **alignment action**.

### CC-0001 — Canonical correlation field set (trace-wide vocabulary + required/optional classification)

**Decision/contract to lock**
- Define the canonical set of correlation identifiers and their required/optional classification, by event family, across:
  - command spans (`command_start` / `command_complete`)
  - in-world process tree events (`world_process_*`)
  - structured agent events
  - LLM request lifecycle events/spans
  - workflow root/node spans (future; must remain compatible)
  - workflow-router derived events (rule/request lifecycle)
  - MCP/toolbox tool-call events

**Current sources**
- Attribution envelope draft: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- Workflow trace intent: `docs/project_management/next/workflow-engine/decision_register.md` (DR-0005)
- Router derived events: `docs/project_management/next/host_event_bus_router_daemon/decision_register.md` (DR-0003, DR-0007, DR-0008)
- LLM gateway correlation intent: `docs/project_management/next/llm_gateway_in_world/contract.md`

**Gap**
- Historically, downstream ADRs/DRs asserted “must carry” fields without a single authoritative matrix (drift risk).

**Alignment action**
- Phase 8: addressed by the additive “Correlation vocabulary + required/optional matrix” section in ADR-0028:
  - field vocabulary (names + meanings),
  - required/optional matrix per event family,
  - and a strict “no heuristic joins” joinability rule.

---

### CC-0002 — `session_id` vs `orchestration_session_id` semantics (no heuristic joins)

**Decision/contract to lock**
- Define whether:
  - `session_id` remains the shell trace session identifier, and
  - `orchestration_session_id` is the multi-agent orchestration session identifier,
  - and when both must appear on a record (or whether one supersedes the other for certain families).

**Current sources**
- Existing trace docs use `session_id`: `docs/TRACE.md`
- Structured agent events require `orchestration_session_id`: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- LLM gateway wants `orchestration_session_id`/`run_id`/`thread_id`: `docs/project_management/next/llm_gateway_in_world/contract.md`

**Gap**
- Without an explicit mapping story, consumers (router, session loggers, UIs) risk heuristic “best-effort” joins between trace sessions and orchestration sessions.

**Alignment action**
- Phase 8: addressed by ADR-0028 Phase 8 additive vocabulary + required/optional matrix:
  - `session_id` remains mandatory on all canonical trace records,
  - `orchestration_session_id` is required on any record family that participates in multi-agent orchestration joins,
  - and consumers MUST NOT assume a 1:1 mapping between `session_id` and `orchestration_session_id` (no heuristic joins).

---

### CC-0003 — `agent_id` meaning (principal identity vs backend identity)

**Decision/contract to lock**
- Decide a single semantic meaning for `agent_id` across:
  - trace command spans,
  - structured agent events,
  - LLM spans/events,
  - router derived events,
  - toolbox tool-call events.

**Current sources**
- Trace example uses `agent_id` as a generic “who ran this” label: `docs/TRACE.md`
- Structured agent event envelope defines `agent_id` as the actor/principal identifier and relies on `backend_id` for backend selection identity: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- Agent hub derives `backend_id` (`<kind>:<agent_id>`): `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0001)

**Gap**
- Historically, ambiguity between “human/actor principal” and “agent backend identity” created downstream join and audit confusion.

**Alignment action**
- Phase 8 additive alignment:
  - Define `agent_id` as the **principal/actor identifier** (`human` for operator actions; agent inventory id for agent-driven actions/events).
  - Define `backend_id` as the **backend identifier** in `<kind>:<name>` form when a specific backend is involved.
  - Require `backend_id` when the backend kind/name is known so allowlist/routing joins are explicit and non-heuristic.
  - Implemented via additive clarifications in:
    - ADR-0028 Phase 8 correlation vocabulary (`agent_id` vs `backend_id`)
    - ADR-0017 structured agent event envelope section
    - Agent Hub DR-0003 envelope field descriptions

---

### CC-0004 — Structured agent event envelope extensions (world attribution + routing hints)

**Decision/contract to lock**
- Extend the structured agent event envelope additively to support:
  - `world_id` attribution (when `execution.scope=world`)
  - an event-plane routing hint (`channel`/`topic`) suitable for subscribe/filter semantics

**Current sources**
- Initial envelope shape: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- World session reuse decision requires surfacing `world_id`: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0004)
- Phase 8 explicit discussion points: `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md` (Phase 8 section)

**Gap**
- Historically, ADR-0017 referenced envelope fields only via decision registers, creating drift risk between ADR text and the DR-defined envelope extensions.

**Alignment action**
- Phase 8: addressed additively by:
  - adding an explicit “Structured agent event envelope (v1; Phase 8 additive clarifications)” section to ADR-0017 that lists the envelope fields and their required/conditional/optional semantics (`backend_id`, conditional `world_id`, optional `channel`),
  - locking `channel` constraints in ADR-0017 as non-negotiable (producer-declared, capped, no secrets, not a join key, not used for policy gating),
  - and aligning the envelope field names to ADR-0028’s canonical correlation vocabulary.

---

### CC-0005 — Agent Hub control plane vs event plane (explicit separation + policy gates)

**Decision/contract to lock**
- Explicitly define:
  - the **control plane** (task assignment, cancel, steering RPCs) and its policy gates
  - the **event plane** (structured events/telemetry) and its routing/attribution contract

**Current sources**
- Phase 8 discussion point: `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md`
- ADR-0026 already insists tools must not create a second execution plane: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Router indirect execution is separately gated by `workflow.router.*`: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` and router DR-0017

**Gap**
- Historically, “control plane” vs “event plane” was referenced as a risk, but not locked as an explicit contract with surfaces, gates, and audit records.

**Alignment action**
- Phase 8: addressed by explicitly locking the two-plane model and its gates:
  - Agent Hub core defines control-plane vs event-plane separation and forbids treating event-plane records as a general-purpose execution trigger (ADR-0025).
  - The internal toolbox is the v1 control-plane surface and is introspection-only (no mutating tools), preventing “second execution plane” drift (ADR-0026; DR-0010).
  - Control-plane enablement is fail-closed and explicit:
    - `agents.enabled=true` and `agents.toolbox.enabled=true` (config; ADR-0027),
    - orchestrator backend allowlisted by `agents.allowed_backends[*]` (policy; ADR-0027),
    - valid per-session auth token (ADR-0026).
  - Control-plane auditability is provided by a dedicated tool-call trace family keyed by `tool_call_id` (ADR-0026; ADR-0028).

---

### CC-0006 — Secrets delivery channel rubric (FD/pipe vs env vars; cross-track standard)

**Decision/contract to lock**
- Establish a reusable rubric for secrets delivery between Substrate-managed components:
  - prefer inherited one-time FD/pipe channels where Substrate spawns and controls both endpoints,
  - allow env var injection where interop requires it (3rd-party tools/SDKs) or where transport constraints make FD/pipe infeasible.

**Current sources**
- Toolbox token explicitly chooses FD/pipe: `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md` (DR-0009)
- LLM gateway chooses env injection (v1): `docs/project_management/next/llm_gateway_in_world/specs/env_injection.md`
- Cross-track rubric (authoritative): `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`

**Gap**
- Historically, secret-handling decisions were scattered across tracks, creating ad-hoc env var expansion risk and inconsistent operator expectations.

**Alignment action**
- Phase 8: addressed by introducing the shared standard `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md` and updating relevant ADRs/DRs/specs to reference it as the shared rationale.

**Inventory (Phase 8 circle-back; host→world secret channel surfaces)**

| Secret values (examples) | Source of truth (host) | Transport (host→world) | In-world consumer | In-world delivery (current) | Current policy gates | Recommendation (Phase 8) |
|---|---|---|---|---|---|---|
| Provider API keys for `api:*` backends (e.g., host `OPENAI_API_KEY`) | Host process env (names declared in agent inventory `config.api.auth.env`) | World-agent spawn request (gateway ensure/sync) | In-world gateway/engine | **v1:** secret-bearing env vars on the gateway/engine process (`SUBSTRATE_LLM_BACKEND_AUTH_API_<NAME>_<FIELD>`) | `llm.secrets.env_allowed` (deny-by-default) + `llm.allowed_backends` + `net_allowed` | Keep host env as the source, but upgrade host→world to a **secret-channel payload** and deliver to the in-world gateway/manager via **FD/pipe by default** (no secret values in in-world process env). |
| Codex subscription auth for `cli:codex` (e.g., account id + access token extracted from `~/.codex/auth.json`) | Host credential file read (with optional env override if specified) | World-agent spawn request (gateway ensure/sync) | In-world gateway/manager and/or Substrate-owned wrapper | **v1:** secret-bearing env vars on the gateway/manager process (`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`) | `agents.host_credentials.read.allowed_backends` (deny-by-default) + `llm.allowed_backends` + `net_allowed` | Keep host credential reads narrow + policy-gated, but upgrade host→world to a **secret-channel payload** and deliver to the in-world gateway/manager via **FD/pipe by default** (no secret values in in-world process env). |
| Gateway/manager → Substrate-spawned wrapper/engine auth propagation | In-world gateway/manager memory | In-world inherited FD/pipe | Substrate-owned wrapper/engine process | **v1:** FD/pipe by default (env var fallback only when required) | N/A (internal propagation; still subject to redaction/caps invariants) | Keep FD/pipe as the default for Substrate-spawned processes; ensure the same “no secret env by default” posture holds for host→world delivery. |

**Inventory (Phase 8 circle-back; canonical `SUBSTRATE_LLM_BACKEND_AUTH_*` field-name set)**

These identifiers are the canonical **auth field names** (even when values are delivered via FD/pipe bundles rather than env vars). Any field name in this family MUST be treated as secret-bearing for redaction/caps purposes.

| Auth field name (`SUBSTRATE_LLM_BACKEND_AUTH_*`) | Scope | Source of value | Notes |
|---|---|---|---|
| `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID` | `cli:codex` | Host credential read (default) and/or explicit env override (if supported by adapter) | Fixed, closed set for `cli:codex` v1. |
| `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN` | `cli:codex` | Host credential read (default) and/or explicit env override (if supported by adapter) | Fixed, closed set for `cli:codex` v1. |
| `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY` | `api:openai` | Host env read (`OPENAI_API_KEY`), gated by `llm.secrets.env_allowed` | Current v1 “quick win” example; `api:*` backends may add additional fields additively per-inventory. |

For `api:*` backends, the canonical field-name family is defined by:
- `SUBSTRATE_LLM_BACKEND_AUTH_API_<BACKEND_NAME>_<FIELD>`
- `BACKEND_NAME` is the backend id name component (e.g., `api:openai` → `OPENAI`).
- `FIELD` is the backend’s declared auth field name (v1: derived from the declared host env var name(s) in `config.api.auth.env` per the gateway spec/DR; must be deterministic and documented per backend).

---

### CC-0007 — Workflow-router derived event families + correlation keys (trace-aligned)

**Decision/contract to lock**
- Define the derived router event families (at minimum):
  - `rule_match`
  - `request_enqueued`
  - `request_denied` / `request_allowed` (or a single status event with outcome)
  - `action_executed` (and failure variants)
- Define required correlation keys:
  - stable cause references to the source trace record (`span_id`/`cmd_id` and/or a future `event_id`)
  - `workspace_id` (source and target)
  - `rule_id`, `request_id`, `idempotency_key`

**Current sources**
- Derived events appended to `trace.jsonl`: `docs/project_management/next/host_event_bus_router_daemon/decision_register.md` (DR-0003)
- Trigger allowlist + recursion guard: `docs/project_management/next/host_event_bus_router_daemon/decision_register.md` (DR-0007)

**Gap**
- Historically, router DRs chose “append derived events to trace” before the canonical trace vocabulary explicitly enumerated router-derived families/keys.

**Alignment action**
- Phase 8: addressed additively by:
  - router DR-0016 enumerating the v1 `workflow_router_*` derived event types and required correlation keys,
  - ADR-0028 listing router-derived families and requiring explicit cause references (`source_span_id`/`source_cmd_id`) and stable join keys (`workspace_id`, `request_id`, `idempotency_key`, `rule_id`),
  - and `docs/TRACE.md` documenting the operator-facing derived families and join keys.

---

### CC-0008 — Workflow trace classification additions (future, but must be reserved additively)

**Decision/contract to lock**
- Reserve and define the workflow trace families/fields needed by:
  - a workflow root span (`workflow_run`)
  - workflow node spans (`workflow_node`)
  - linkage between node spans and underlying command spans

**Current sources**
- Workflow spans decision: `docs/project_management/next/workflow-engine/decision_register.md` (DR-0005)

**Gap**
- Historically, workflow fields were assumed downstream without explicit reservation in the canonical vocabulary.

**Alignment action**
- Phase 8: addressed additively by reserving `workflow_run_id` and `workflow_node_id` in ADR-0028’s correlation vocabulary (no reshapes; future workflow composition remains Draft).

---

### CC-0009 — MCP/toolbox tool-call correlation (`tool_call_id` and trace visibility)

**Decision/contract to lock**
- Define `tool_call_id` (and related fields) and ensure tool invocations are:
  - attributable to `(orchestration_session_id, agent_id, role, world_id?)`
  - persisted in trace in a stable family suitable for router/analytics (even if router v1 cannot trigger on it)

**Current sources**
- Toolbox exists and requires attribution: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Phase 8 explicitly calls out `tool_call_id`: `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md`

**Gap**
- `tool_call_id` was reserved in ADR-0028’s correlation vocabulary, but the end-to-end tool-call trace family and required join keys were not previously locked as a persisted record family.

**Alignment action**
- Phase 8: addressed by introducing a dedicated toolbox tool-call trace family and wiring it into the correlation vocabulary:
  - `toolbox_tool_call_start` / `toolbox_tool_call_complete` (ADR-0026; v1 additive list),
  - `tool_call_id` is required on these records (ADR-0028 matrix),
  - operator-facing trace documentation includes the tool-call families and join keys (see `docs/TRACE.md`).

---

### CC-0010 — World session reuse + restart attribution (operator-verifiable)

**Decision/contract to lock**
- Ensure operators can verify:
  - whether multiple agents shared the same world boundary (same `world_id`), and
  - when/why a world was restarted.

**Current sources**
- Shared-world default + required `world_restarted` event: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0004)
- Drift handling + reason taxonomy: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0008)
- `world_restarted` alert event schema: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0010)
- `docs/TRACE.md` (operator-facing trace family overview; Phase 8 alignment)

**Gap**
- Historically, `world_id` and restart attribution were asserted in Agent Hub DRs but not clearly surfaced as an operator-facing, persisted contract across the Phase 2 envelope and trace docs.

**Alignment action**
- Phase 8: addressed additively by:
  - requiring `world_id` on structured events for world-scoped agents (ADR-0017 envelope + DR-0003),
  - locking the `world_restarted` alert schema (`kind=alert` + `data.code="world_restarted"`; required world id/generation fields) and reason taxonomy (DR-0008/DR-0010),
  - and documenting the operator-facing alert codes and reason taxonomy in `docs/TRACE.md`.

---

### CC-0011 — Workflow-router rules surface in ADR-0027 (`workflow.*` keys + gating)

**Decision/contract to lock**
- Define the minimal `workflow.*` config and policy surfaces needed by ADR-0029 without introducing new file families.

**Current sources**
- Router DR requires adding `workflow.*` keys to ADR-0027 surfaces: `docs/project_management/next/host_event_bus_router_daemon/decision_register.md` (DR-0006 follow-up)

**Gap**
- Historically, ADR-0027 scoped to `llm.*` and `agents.*`; ADR-0029 required `workflow.router.*` gating keys.

**Alignment action**
- Phase 8: addressed by adding `workflow.router.*` policy keys to ADR-0027 and syncing them into the schema/contract planning outputs (fail-closed defaults; deny-by-default allowlists).

---

### CC-0012 — `docs/TRACE.md` alignment to Phase 1–6 contracts (Phase 8 documentation pointers)

**Decision/contract to lock**
- Update `docs/TRACE.md` to reflect:
  - the new correlation vocabulary (once CC-0001..CC-0003 are resolved),
  - the router derived families (CC-0007),
  - and reserved workflow fields (CC-0008),
  - plus any explicit redaction/caps rules for LLM/agent/toolbox event families.

**Current sources**
- Phase 8 requires documentation pointers/updates: `LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md`

**Gap**
- `docs/TRACE.md` historically documented a command-span-centric schema and did not describe Phase 1–6 planned event-family expansions (LLM/agents/router/workflows).

**Alignment action**
- Phase 8: addressed by updating `docs/TRACE.md` to:
  - link to ADR-0028 as canonical for correlation vocabulary/matrix,
  - document router-derived `workflow_router_*` event types and required join keys,
  - list reserved workflow/toolbox correlation ids (`workflow_run_id`, `workflow_node_id`, `tool_call_id`),
  - and state safe-by-default redaction/caps posture (raw wrapper logs remain per-session artifacts; no heuristic joins).
