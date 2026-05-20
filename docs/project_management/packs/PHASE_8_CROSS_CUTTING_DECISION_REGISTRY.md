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

- Attribution envelope draft: `docs/project_management/_archived/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- Workflow trace intent: `docs/project_management/_archived/next/workflow-engine/decision_register.md` (DR-0005)
- Router derived events: `docs/project_management/_archived/next/host_event_bus_router_daemon/decision_register.md` (DR-0003, DR-0007, DR-0008)
- LLM gateway correlation intent: `docs/project_management/_archived/next/llm_gateway_in_world/contract.md`

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
- Structured agent events require `orchestration_session_id`: `docs/project_management/_archived/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- LLM gateway wants `orchestration_session_id`/`run_id`/`thread_id`: `docs/project_management/_archived/next/llm_gateway_in_world/contract.md`

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
- Structured agent event envelope defines `agent_id` as the actor/principal identifier and relies on `backend_id` for backend selection identity: `docs/project_management/_archived/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- Agent hub derives `backend_id` (`<kind>:<agent_id>`): `docs/project_management/_archived/next/agent_hub_core/decision_register.md` (DR-0001)

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

- Initial envelope shape: `docs/project_management/_archived/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
- World session reuse decision requires surfacing `world_id`: `docs/project_management/_archived/next/agent_hub_core/decision_register.md` (DR-0004)
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

- Toolbox token explicitly chooses FD/pipe: `docs/project_management/_archived/next/orchestration_mcp_toolbox/decision_register.md` (DR-0009)
- LLM gateway chooses env injection (v1): `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`
- Cross-track rubric (authoritative): `docs/project_management/system/standards/shared/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`

**Gap**

- Historically, secret-handling decisions were scattered across tracks, creating ad-hoc env var expansion risk and inconsistent operator expectations.

**Alignment action**

- Phase 8: addressed by introducing the shared standard `docs/project_management/system/standards/shared/SECRETS_DELIVERY_CHANNEL_RUBRIC.md` and updating relevant ADRs/DRs/specs to reference it as the shared rationale.

**Inventory (Phase 8 circle-back; host→world secret channel surfaces)**

| Secret values (examples)                                                                                      | Source of truth (host)                                                     | Transport (host→world)                          | In-world consumer                                       | In-world delivery (current)                                                                                     | Current policy gates                                                                                       | Recommendation (Phase 8)                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Provider API keys for `api:*` backends (e.g., host `OPENAI_API_KEY`)                                          | Host process env (names declared in agent inventory `config.api.auth.env`) | World-agent spawn request (gateway ensure/sync) | In-world gateway/engine                                 | **v1:** secret-bearing env vars on the gateway/engine process (`SUBSTRATE_LLM_BACKEND_AUTH_API_<NAME>_<FIELD>`) | `llm.secrets.env_allowed` (deny-by-default) + `llm.allowed_backends` + `net_allowed`                       | Keep host env as the source, but upgrade host→world to a **secret-channel payload** and deliver to the in-world gateway/manager via **FD/pipe by default** (no secret values in in-world process env).                      |
| Codex subscription auth for `cli:codex` (e.g., account id + access token extracted from `~/.codex/auth.json`) | Host credential file read (with optional env override if specified)        | World-agent spawn request (gateway ensure/sync) | In-world gateway/manager and/or Substrate-owned wrapper | **v1:** secret-bearing env vars on the gateway/manager process (`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`)       | `agents.host_credentials.read.allowed_backends` (deny-by-default) + `llm.allowed_backends` + `net_allowed` | Keep host credential reads narrow + policy-gated, but upgrade host→world to a **secret-channel payload** and deliver to the in-world gateway/manager via **FD/pipe by default** (no secret values in in-world process env). |
| Gateway/manager → Substrate-spawned wrapper/engine auth propagation                                           | In-world gateway/manager memory                                            | In-world inherited FD/pipe                      | Substrate-owned wrapper/engine process                  | **v1:** FD/pipe by default (env var fallback only when required)                                                | N/A (internal propagation; still subject to redaction/caps invariants)                                     | Keep FD/pipe as the default for Substrate-spawned processes; ensure the same “no secret env by default” posture holds for host→world delivery.                                                                              |

**Inventory (Phase 8 circle-back; canonical `SUBSTRATE_LLM_BACKEND_AUTH_*` field-name set)**

These identifiers are the canonical **auth field names** (even when values are delivered via FD/pipe bundles rather than env vars). Any field name in this family MUST be treated as secret-bearing for redaction/caps purposes.

| Auth field name (`SUBSTRATE_LLM_BACKEND_AUTH_*`)    | Scope        | Source of value                                                                       | Notes                                                                                                |
| --------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`   | `cli:codex`  | Host credential read (default) and/or explicit env override (if supported by adapter) | Fixed, closed set for `cli:codex` v1.                                                                |
| `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN` | `cli:codex`  | Host credential read (default) and/or explicit env override (if supported by adapter) | Fixed, closed set for `cli:codex` v1.                                                                |
| `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`     | `api:openai` | Host env read (`OPENAI_API_KEY`), gated by `llm.secrets.env_allowed`                  | Current v1 “quick win” example; `api:*` backends may add additional fields additively per-inventory. |

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

- Derived events appended to `trace.jsonl`: `docs/project_management/_archived/next/host_event_bus_router_daemon/decision_register.md` (DR-0003)
- Trigger allowlist + recursion guard: `docs/project_management/_archived/next/host_event_bus_router_daemon/decision_register.md` (DR-0007)

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

- Workflow spans decision: `docs/project_management/_archived/next/workflow-engine/decision_register.md` (DR-0005)

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

- Shared-world default + required `world_restarted` event: `docs/project_management/_archived/next/agent_hub_core/decision_register.md` (DR-0004)
- Drift handling + reason taxonomy: `docs/project_management/_archived/next/agent_hub_core/decision_register.md` (DR-0008)
- `world_restarted` alert event schema: `docs/project_management/_archived/next/agent_hub_core/decision_register.md` (DR-0010)
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

- Router DR requires adding `workflow.*` keys to ADR-0027 surfaces: `docs/project_management/_archived/next/host_event_bus_router_daemon/decision_register.md` (DR-0006 follow-up)

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

## Potential Crates/repos to use:

- https://crates.io/crates/mpatch
- https://crates.io/crates/pmat
- https://crates.io/crates/pmcp
- https://crates.io/crates/claude-codes
- https://github.com/meawoppl/rust-claude-codes
- https://crates.io/crates/rmcp-openapi
- https://crates.io/crates/codex-helper
- https://crates.io/crates/langchain-rust
- https://crates.io/crates/worktrunk
- https://crates.io/crates/fluxencrypt
- https://crates.io/crates/workmux
- https://crates.io/crates/vtcode-file-search
- https://crates.io/crates/aichat-search
- https://github.com/9j/claude-code-mux
- https://crates.io/crates/claude-hook-advisor
- https://crates.io/crates/agents-core#quick-start
- https://github.com/Recusive/agentsdk
- https://github.com/erans/lunaroute
- https://github.com/sopaco/cortex-mem
- https://github.com/sopaco/deepwiki-rs
- https://github.com/0xPlaygrounds/rig?tab=readme-ov-file
- https://github.com/aipack-ai/aipack
- https://github.com/CSCSoftware/AiDex
- https://chatgpt.com/c/6925064b-cd44-8331-b06c-fab4a5b622a2

Detailed table

Star/commit counts are from the current GitHub pages; release info is from docs.rs where I could see it. Some “last release” cells are “n/a” where I didn’t have enough budget to pull exact versions/dates.

Legend for “Candidate for substrate?”

Crate = realistically usable as a direct Cargo dependency

Clone bits = probably read & selectively copy internal code

Reference = mostly architectural / API inspiration

No = not really in scope for Substrate

1. Core agent / runtime / LLM crates
   Library / Tool Description / Purpose Stars (GH) Commits (GH) Latest release / activity (approx) docs.rs URL GitHub URL Candidate for substrate?
   agents-runtime Tokio-powered runtime that glues together planners, tools, prompts, state stores, tracing; part of Rust Deep Agents SDK. 13 100 Crate agents-runtime 0.0.25 (0.x, active). https://docs.rs/agents-runtime/latest/agents_runtime/
   https://github.com/yafatek/rust-deep-agents-sdk
   Crate (for an orchestration/“deep agent” layer), but I’d treat it carefully and probably wrap it behind your own traits.
   agents-core Core traits, events, state, tool system and persistence primitives for the Deep Agents SDK. 13 (same repo) 100 Crate agents-core 0.0.25 (same workspace). https://docs.rs/agents-core/latest/agents_core/
   https://github.com/yafatek/rust-deep-agents-sdk
   Crate – useful for your internal event/state/tool types even if you don’t adopt their runtime wholesale.
   genai Multi-provider generative AI client for Rust (OpenAI, Anthropic, Gemini, Groq, DeepSeek, Cohere, Ollama, etc.); unified Client API. 566 554 Crate genai 0.4.4 (actively maintained). https://docs.rs/genai/latest/genai/
   https://github.com/jeremychone/rust-genai
   Crate – my top pick for Substrate’s LLM gateway layer. Wrap this in a substrate-llm crate with token/cost tracking + policy.
   open-agent-sdk (open_agent) Production-ready, streaming-first Rust SDK for building agents against local OpenAI-compatible servers (LM Studio, Ollama, llama.cpp, vLLM) with tools, hooks, interrupts, etc. 6 70 Crate open_agent 0.6.0 (well-documented). https://docs.rs/open-agent-sdk/latest/open_agent/
   https://github.com/slb350/open-agent-sdk-rust
   Crate (optional) – nice drop-in backend for “local models” support if you don’t want to hand-roll it on top of genai.
   llm-toolkit “Basic llm tools for rust” – small toolbox / helper crates for LLM work. 2 368 n/a (didn’t inspect crate metadata). https://docs.rs/llm-toolkit/latest/llm_toolkit/
   https://github.com/ynishi/llm-toolkit
   Reference – lots of small utilities; but given its low adoption and the overlap with genai/rig/etc., I’d treat it as a code/idea reference, not a core dep.
   radkit Rust Agent Development Kit; provides agent primitives, macros, examples, docs; seems aimed at their own “agents-sh” ecosystem. 41 110 n/a https://docs.rs/radkit/latest/radkit/
   https://github.com/agents-sh/radkit
   Reference – useful to study how they do agent composition & tooling, but you already have your own substrate architecture emerging.
   cloudllm Batteries‑included Rust toolkit for building intelligent agents with LLM integration, multi‑protocol tools (incl. MCP), LLMSession for token‑aware context, and a council multi-agent orchestration engine; supports OpenAI, Claude, Gemini, Grok, and OpenAI-compatible endpoints. 7 193 Crate exists (not inspected), repo quite active. https://docs.rs/cloudllm/latest/cloudllm/
   https://github.com/CloudLLM-ai/cloudllm
   Clone bits / Reference – extremely aligned with your goals, but almost overlaps the entire Substrate vision. Great source for ideas (esp. LLMSession, MCPServerBuilder, council patterns); I’d avoid taking it wholesale as a dependency.
   ai-lib Unified multi-provider AI SDK for Rust; production-grade, provider-agnostic API for 20+ platforms (OpenAI, Groq, Anthropic, Gemini, Mistral, Azure, Ollama, DeepSeek, etc.). 3 153 Has published crate(s), actively maintained. https://docs.rs/ai-lib/latest/ai_lib/
   https://github.com/hiddenpath/ai-lib
   Reference – another multi-provider option, but with far less adoption than genai. Good design reference, but I’d standardize on genai rather than juggling multiple gateways.
   rig (rig-core) Large framework for modular and scalable LLM apps (tools, vector stores, connectors, etc.), with many sub-crates. 5k 751 Actively releasing multiple rig-\* crates. https://docs.rs/rig-core/latest/rig/
   https://github.com/0xPlaygrounds/rig
   Reference – great to steal patterns (tools, resource abstractions, multi-backend support), but it’s its own world. Using it directly would make substrate “Rig with extra steps.”
2. MCP / tool ecosystem
   Library / Tool Description / Purpose Stars Commits Latest release / activity docs.rs URL GitHub URL Candidate for substrate?
   pmat Pragmatic AI Labs MCP Agent Toolkit – an MCP server & toolkit designed to make agent code more deterministic; a very large, batteries‑included monorepo with a whole CLI and ecosystem. 104 2,396 Crate pmat 2.205.0 (extremely active). https://docs.rs/crate/pmat/latest
   https://github.com/paiml/paiml-mcp-agent-toolkit
   Reference – this is essentially its own agent platform. Great to mine for patterns and maybe some code (diagnostics, CLI ergonomics), but using it as a dependency would blur the line between Substrate and PMAT.
   pmcp (rust-mcp-sdk) Pragmatic AI Labs MCP SDK – focused MCP client/server library (with cargo pmcp tooling) that pmat builds on; full-featured, includes streaming, file watching, etc. 18 769 Crate pmcp (well-documented, 100% docs coverage). https://docs.rs/pmcp/latest/pmcp/
   https://github.com/paiml/rust-mcp-sdk
   Crate – strong candidate for your MCP client + registry layer (connecting to arbitrary MCP servers), especially early on.
   prism-mcp-rs “Enterprise-grade Rust implementation of Anthropic’s MCP protocol” – heavy focus on correctness, testing, and enterprise concerns (supply chain, deny.toml, etc.). 39 132 Crate prism-mcp-rs exists and is actively updated. https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/
   https://github.com/prismworks-ai/prism-mcp-rs
   Crate – top contender for the substrate-level MCP server engine (especially if you want strong testing & security posture). You can still use pmcp for client-side integration with the broader MCP ecosystem.
   AIPack “Run, Build, Share your AI Packs” – opinionated agent packaging/runtime with its own CLI and concepts (packs, etc.). 165 1,328 Very active repository & releases. https://docs.rs/crate/aipack/latest
   https://github.com/aipack-ai/aipack
   Reference – great to look at for UX, config, and packaging ideas, but not something you embed; Substrate should define its own mental model.
   aichat Very popular all‑in‑one LLM CLI (shell assistant, REPL, RAG, tools/agents; supports OpenAI, Claude, Gemini, Ollama, Groq, etc.). 8.7k 985 Very active with frequent tags/releases. https://docs.rs/crate/aichat/latest
   https://github.com/sigoden/aichat
   Reference – this is “what substrate wants to orchestrate rather than replace.” Treat as competitor + design reference (commands, config, tool UX).
3. Claude / Codex / routing ecosystem
   Library / Tool Description / Purpose Stars Commits Latest release / activity docs.rs URL GitHub URL Candidate for substrate?
   claude-code-mux High-performance AI routing proxy in Rust (automatic failover, priority routing, supports 15+ providers including Anthropic, OpenAI, Cerebras, Minimax, Kimi, etc.). 374 38 Very active; used as a stand‑alone proxy. https://docs.rs/crate/claude-code-mux/latest
   https://github.com/9j/claude-code-mux
   Clone bits – extremely relevant to your LLM gateway; worth studying and possibly copying routing/failover strategies, but I’d keep your gateway integrated directly into Substrate instead of relying on an external proxy.
   cc-sdk / claude-code-api-rs High-performance Rust implementation of an OpenAI-compatible API gateway for Claude Code CLI; includes claude-code-sdk-rs library used by others (e.g., url-preview). 53 28 Actively updated (release notes, v0.3.0, etc.). https://docs.rs/cc-sdk/latest/cc_sdk/
   https://github.com/ZhangHanDong/claude-code-api-rs
   Clone bits – a good blueprint if you ever want Substrate to expose an OpenAI-compatible HTTP API. For core substrate, I’d keep Claude Code integration at the CLI level and only reuse ideas/wire types.
   kodegen_claude_agent “Claude Agent SDK for Rust” – Rust bindings around Claude Code; async, strong typing; small crate (0.3.3). ~0 ~7 Crate 0.3.3 present but early-stage. https://docs.rs/kodegen_claude_agent/latest/kodegen_claude_agent/
   https://github.com/cyrup-ai/kodegen-claude-agent
   Reference – good for protocol examples, but too early-stage to depend on. Your own Unified Agent Wrapper is already further along conceptually.
   turboclaudeagent Interactive Agent SDK for TurboClaude; designed for in‑IDE agents with hooks, permission callbacks, interactive sessions, etc. 5 15 Crate turboclaudeagent 0.1.0. https://docs.rs/turboclaudeagent/latest/turboclaudeagent/
   https://github.com/Epistates/turboclaude
   Reference – great for ideas about IDE‑style permission gating and hooks; not something you want as a hard dependency.
   claude-agent-sdk Claude Agent SDK for Rust, also targeting Claude Code CLI; similar story to kodegen_claude_agent. 7 8 Crate 0.1.1. https://docs.rs/claude-agent-sdk/latest/claude_agent_sdk/
   https://github.com/Wally869/claude_agent_sdk_rust
   Reference – more protocol examples; but you’d end up with a lot of overlapping abstractions if you pulled it in.
   claude-agent-sdk-rs Another Claude Code SDK (“Rust SDK for Claude Code CLI – build production-ready AI agents with type safety”). 12 18 Has crate claude-agent-sdk-rs on docs.rs (not inspected). https://docs.rs/claude-agent-sdk-rs/latest/claude_agent_sdk_rs/
   https://github.com/tyrchen/claude-agent-sdk-rs
   Reference / maybe Clone bits – if you decide to reuse one Claude SDK’s types, this is the one I’d inspect first, but I’d still lean toward your own thin wrapper over the CLI.
   codex-helper Small helper library around Codex/Claude Code (providing convenience wrappers and example configs). 6 13 Crate exists (codex-helper), small and simple. https://docs.rs/crate/codex-helper/latest
   https://github.com/Latias94/codex-helper
   Reference – useful to see how others manage Codex configs and invocation, but Substrate’s Unified Agent Wrapper is more central and should stay in your control.
   url-preview High‑performance library for generating rich URL previews; already uses claude-code-api as a backend for LLM-based extraction. 10 33 Crate url-preview with v0.6.0 using claude-code-api. https://docs.rs/url-preview/latest/url_preview/
   https://github.com/ZhangHanDong/url-preview
   Crate – very nice candidate for a substrate-level MCP/tool that “previews URLs” for agents. Clean, self-contained.
4. Security, structure, and “infrastructure helpers”
   Library / Tool Description / Purpose Stars Commits Latest release / activity docs.rs URL GitHub URL Candidate for substrate?
   llm-security “Comprehensive LLM security layer to prevent prompt injection and manipulation attacks.” Includes examples and docs; relatively small but focused. 8 8 Has crate llm-security, active but young. https://docs.rs/llm-security/latest/llm_security/
   https://github.com/redasgard/llm-security
   Crate – this is almost exactly the kind of thing you want at the orchestration layer for request/response sanitization and policy enforcement.
   rstructor “Pydantic + Instructor for Rust” – structured output / schema enforcement around LLM calls; integrates nicely into Rust type system. 14 68 Crate rstructor published and maintained. https://docs.rs/rstructor/latest/rstructor/
   https://github.com/clifton/rstructor
   Crate – strong candidate for typed tool results, config schemas, and JSON output across your --json modes.
   sublinear-time-solve Repo name suggests algorithmic / problem-solving utilities; I couldn’t successfully fetch the GitHub page via the tools (internal error). n/a n/a n/a (no docs.rs crate) https://github.com/ruvnet/sublinear-time-solve
   No – appears unrelated to agent orchestration / LLM infrastructure.
5. Big “agent OS / CLI” systems
   Library / Tool Description / Purpose Stars Commits Latest release / activity docs.rs URL GitHub URL Candidate for substrate?
   pmat (see above) – full MCP agent toolkit + CLI. 104 2,396 Very active. https://docs.rs/crate/pmat/latest
   https://github.com/paiml/paiml-mcp-agent-toolkit
   Reference – too opinionated to embed.
   AIPack Agent/pack-based environment; you run/build/share “AI Packs” with their CLI. 165 1,328 Rapid development, multiple issues & discussions. https://docs.rs/crate/aipack/latest
   https://github.com/aipack-ai/aipack
   Reference – learn from their UX and packaging, but Substrate should keep its own UX.
   aichat All‑in‑one LLM CLI with shell assistant, chat REPL, RAG, tools & agent support. 8.7k 985 Very active. https://docs.rs/crate/aichat/latest
   https://github.com/sigoden/aichat
   Reference – treat as “competition and inspiration” rather than dependency.
6. Miscellaneous / automation
   Library / Tool Description / Purpose Stars Commits Latest release / activity docs.rs URL GitHub URL Candidate for substrate?
   terminator-rs Playwright-style SDK for automating desktop GUI apps (cross-platform, lots of crates under the hood). ~1.2k 2,140 Crate terminator-rs 0.23.22 (releasing very frequently lately). https://docs.rs/crate/terminator-rs/latest
   https://github.com/mediar-ai/terminator
   Reference (future) – not core to “CLI-first substrate,” but very interesting if you later add “GUI execution agents.”
