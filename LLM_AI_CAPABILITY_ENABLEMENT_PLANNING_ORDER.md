# LLM/AI Capability Enablement — ADR Finalization Order (with ADR-0028 Circle-Back)

This document is a lightweight tracking plan for finalizing ADRs and their corresponding schema/contract/spec files in an order that minimizes rewrites.

## Phase 0 — Freeze “No Rewrite” rules (one-time)
- Rule A: Once an ADR is `Accepted`, subsequent edits are additive-only (no contract reshapes).
- Rule B: Any later ADR that needs new trace fields/config keys must propose them as extensions to:
  - ADR-0028 (trace/event families + correlation fields + redaction/caps), or
  - ADR-0027 (config/policy key paths + precedence + defaults).

## Phase 1 — Trace foundation (accept early)
- ADR-0028: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Outputs (required):
  - `docs/project_management/_archived/next/world_process_exec_tracing_parity/spec_manifest.md`
  - Trace event schema spec (`world_process_*`)
  - Shared redaction spec (argv/env) + caps/truncation spec
  - World-agent API payload spec (`process_events`)
  - Span parent-linkage fix spec

## Phase 2 — Output/routing contract (accept)
- ADR-0017: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- Outputs (required):
  - Output class contract (PTY bytes vs structured events) + buffering/render rules
  - Attribution requirements for concurrent agent output

## Phase 3 — Config/policy surface for LLM + agents (accept)
- ADR-0027: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Outputs (required):
  - `docs/project_management/_archived/next/llm_and_agent_config_policy_surface/contract.md`
  - Schema/specs for new `llm.*` and `agents.*` keys (config + policy)
  - Fail-closed defaults + precedence rules

## Phase 4 — LLM front door then engines (accept in order)
- ADR-0023: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
- ADR-0024: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
- Outputs (required):
  - Gateway HTTP contract + world-boundary requirements
  - Backend capability/routing contract (CLI engines)
  - All logging/attribution requirements must reference ADR-0028 + ADR-0017

## Phase 5 — Agent hub then toolbox (accept in order)
- ADR-0025: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
- ADR-0026: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Outputs (required):
  - Agent backend interface + role assignment contract + attribution
  - MCP tool namespace/schemas + role gating rules

## Phase 6 — Host event bus/router daemon (accept before workflow composition)
- ADR-0029: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- Outputs (required):
  - Router daemon contract (trace-driven triggers → policy-gated requests/actions)
  - Durable request queue semantics (`inbox`/`work_queue`/cursor+dedupe state)
  - Workspace registry + `workspace_id` contract
  - FS-change trigger semantics aligned to ADR-0018 path matching

## Phase 7 — Workflow composition (accept last)
- ADR-0021: `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`
- ADR-0022: `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`

## Phase 8 — Circle-back pass (additive-only): trace classifications + landing items
- Phase 8 working registry (cross-cutting alignment): `docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md`
- Note (workflow composition):
  - ADR-0021 and ADR-0022 MUST remain `Draft` until we are closer to implementation.
  - We will defer solidifying remaining `workflow-engine` / `forge` contract details (beyond already-accepted DR items) until enough upstream foundations have landed (Phases 1–6) and we are ready to produce the Phase 7 Planning Packs.
- Circle back to ADR-0028:
  - Additive updates only:
    - new trace event families required by LLM/agents/workflows (if any)
    - additional correlation fields (e.g., `agent_id`, `tool_call_id`, `workflow_node_id`) with required/optional classification
    - any derived trigger/request lifecycle event families introduced by the router daemon (ADR-0029)
    - special redaction/caps notes for LLM/agent-specific subprocesses
    - documentation pointers/updates (`docs/TRACE.md` as needed)
  - Non-negotiable: do not reopen the core capture mechanism choice, base event types, or span-parent correctness—only extensions.

- Circle back (cross-track standard): secrets delivery channel rubric (FD/pipe vs env vars)
  - Goal: establish a concrete, reusable decision rubric for when Substrate should use:
    - inherited one-time FD/pipe secret channels (preferred for Substrate-spawned, Substrate-owned components), vs
    - environment-variable injection (interop-required cases; third-party tools/SDKs; world-boundary transport constraints).
  - Standard: `docs/project_management/standards/SECRETS_DELIVERY_CHANNEL_RUBRIC.md`
  - Output: update the relevant Decision Registers/ADRs to reference the rubric so current and future secret-handling decisions stay consistent (and avoid ad-hoc env var proliferation).

- Circle back to other foundation ADRs / decision registers (additive-only):
  - ADR-0017 (output routing contract):
    - align “structured agent events” attribution requirements to the final correlation set (`orchestration_session_id`, `run_id`, `thread_id`, `agent_id`, `role`, and join keys like `cmd_id`/`span_id` when applicable),
    - confirm buffering/backpressure rules remain compatible with any later session-log persistence strategy (do not conflate rendering with persistence).
    - discussion point (agent hub circle-back): confirm the structured-event envelope can optionally carry an event-plane routing hint (e.g., `channel` / `topic`), so future “subscribe/filter” behavior can be expressed without PTY injection or attribution ambiguity; ensure any “dropped buffered lines” summaries preserve the same routing metadata so suppressed output remains explainable.
    - discussion point (agent hub circle-back): decide and specify world session reuse + attribution—when multiple agents run “in world” under one `orchestration_session_id`, do they share a single `world_id` for the entire session by default, and should structured events carry `world_id` (and any “world restart” reason) so operators can verify that agents did or did not share the same filesystem/isolation boundary.
  - ADR-0027 (LLM + agent config/policy surface):
    - align backend id formats and role/tool gating keys with the final agent hub + MCP toolbox specs,
    - ensure any newly-discovered policy gates remain fail-closed by default and do not introduce secret storage.
    - discussion point: keep ADR-0027 limited to backend **id format + allowlist/selection surfaces** (no canonical “backend registry” list here); once ADR-0023/ADR-0024 (LLM gateway + engines) and ADR-0025 (agent backends) are accepted, circle back to add references (and, if helpful, a non-normative appendix mapping ids → their authoritative backend contracts).
  - ADR-0025 (agent hub core):
    - discussion point: explicitly separate **control plane** (orchestrator → executor steering RPCs; cancel; task assignment) from the **event plane** (executor → hub structured events), so “who can steer whom” is policy-gated and auditable while output streaming/rendering remains a pure event-plane concern (aligns with ADR-0017 and avoids conflating rendering with routing).
    - discussion point: define an event-plane **subscription/channel** model (pub/sub-style) for concurrent multi-agent operation, where agent configuration can declare which channels it emits to and which channels it may receive steering from; this is required to support a host-scoped orchestration agent (control-plane only) while keeping all LLM egress and world-bound capabilities in-world and subject to effective policy.
  - ADR-0029 (host event bus/router daemon):
    - align the v1 trigger allowlist to the final event families and correlation fields emitted by LLM gateway, agent hub, and workflow engine,
    - ensure request/derived-event schemas reference stable join keys (cause/trigger refs) consistent with the final trace/span contract.
  - Decision registers that define correlation/attribution surfaces (verify naming + required/optional classification, additive-only):
    - `docs/project_management/_archived/next/agent-hub-concurrent-execution-output-routing/decision_register.md`
    - `docs/project_management/_archived/next/agent_hub_core/decision_register.md`
    - `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md`
    - `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md`
    - `docs/project_management/_archived/next/orchestration_mcp_toolbox/decision_register.md`
    - `docs/project_management/_archived/next/workflow-engine/decision_register.md`
    - `docs/project_management/_archived/next/forge/decision_register.md`
