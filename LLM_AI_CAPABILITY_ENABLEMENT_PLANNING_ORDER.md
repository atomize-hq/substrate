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
  - `docs/project_management/next/world_process_exec_tracing_parity/spec_manifest.md`
  - Trace event schema spec (`world_process_*`)
  - Shared redaction spec (argv/env) + caps/truncation spec
  - World-agent API payload spec (`process_events`)
  - Span parent-linkage fix spec

## Phase 2 — Output/routing contract (accept)
- ADR-0017: `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- Outputs (required):
  - Output class contract (PTY bytes vs structured events) + buffering/render rules
  - Attribution requirements for concurrent agent output

## Phase 3 — Config/policy surface for LLM + agents (accept)
- ADR-0027: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Outputs (required):
  - `docs/project_management/next/llm_and_agent_config_policy_surface/contract.md`
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
- Circle back to ADR-0028:
  - Additive updates only:
    - new trace event families required by LLM/agents/workflows (if any)
    - additional correlation fields (e.g., `agent_id`, `tool_call_id`, `workflow_node_id`) with required/optional classification
    - any derived trigger/request lifecycle event families introduced by the router daemon (ADR-0029)
    - special redaction/caps notes for LLM/agent-specific subprocesses
    - documentation pointers/updates (`docs/TRACE.md` as needed)
  - Non-negotiable: do not reopen the core capture mechanism choice, base event types, or span-parent correctness—only extensions.
