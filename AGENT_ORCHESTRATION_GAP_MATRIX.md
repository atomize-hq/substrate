# Agent Orchestration Gap Matrix

## Canonical Product Intent

This section is the canonical description of the intended product shape for the current Substrate agent-orchestration effort.

### v1 intent

Substrate v1 orchestration is intended to use real external CLI agents through the Unified Agent API (`unified-agent-api` on crates.io; Rust crate name `agent_api`), not a bespoke Substrate-native agent runtime.

In v1:

- Substrate selects, policy-gates, and launches supported CLI agents such as Codex, Claude Code, and future additions that are onboarded into Unified Agent API.
- Substrate assigns those agents roles:
  - a host-scoped orchestrator, and
  - one or more member agents that execute inside a Substrate-managed world boundary.
- Substrate provides the execution boundary, policy, inventory, world lifecycle, trace correlation, and orchestration control plane.
- Unified Agent API provides the backend abstraction for the CLI agents themselves:
  - capability discovery,
  - run semantics,
  - explicit cancellation,
  - session resume/fork extensions,
  - and session-handle surfacing.
- The first meaningful shipped path is:
  - host orchestrator,
  - in-world member execution,
  - policy-gated orchestration,
  - and shared-world reuse where needed for member sessions.

### later intent

A later Substrate-native harness may still sit on top of Codex, Claude Code, or other Unified Agent API backends, but that is a follow-on architecture layer, not a prerequisite for v1.

That later layer may own:

- Substrate-specific session/context models,
- custom orchestration UX,
- and deeper runtime integration.

For the current phase, the priority is to get the existing CLI agents working cleanly through Unified Agent API inside Substrate’s orchestration and world model.

## Canonical Decisions

These items should be treated as decided unless explicitly revisited.

1. `unified-agent-api` is the canonical source of truth for CLI-agent runtime semantics.
2. Substrate intends to consume `unified-agent-api` as a normal crates.io dependency, not as a path dependency and not as an indefinitely separate local protocol.
3. The crates.io package is `unified-agent-api`; the Rust crate name is `agent_api`.
4. Substrate v1 should use external CLI agents through Unified Agent API rather than waiting for a custom Substrate-native harness.
5. Substrate owns orchestration semantics, world placement, policy, and trace/audit semantics.
6. Unified Agent API owns backend registration, capability discovery, run control, and session extension semantics for the CLI agents.

## Important Boundary Clarification

There are two different API layers in this repository and they should not be conflated.

- External CLI-agent abstraction:
  - `unified-agent-api` from crates.io
  - Rust crate name: `agent_api`
  - Purpose: run and control Codex / Claude Code / other CLI agents through one capability-gated contract
- Substrate-local host/world transport:
  - `crates/agent-api-types`
  - `crates/agent-api-core`
  - `crates/agent-api-client`
  - Purpose: Substrate host components talking to `world-agent`

The local `agent-api-*` crates are not the same thing as Unified Agent API. They currently represent Substrate’s transport layer for host-to-world execution, not the canonical CLI-agent wrapper contract.

## Current State Summary

- Modeling is ahead of runtime:
  - agent inventory,
  - derived `backend_id`,
  - role/scope validation,
  - tuple-aware structured events,
  - and observational CLI status surfaces are mostly in place.
- Gateway runtime is real for nested LLM/gateway lifecycle work, but that is not yet the same thing as live agent-hub orchestration.
- The main missing layer is the actual runtime control plane that opens, tracks, resumes, forks, stops, and correlates orchestrator/member sessions.

## Gap Matrix

| Area | Status | What exists now | What is missing for the intended v1 path |
|---|---|---|---|
| Product intent | `Now explicit` | This file now records the intended v1 and later product shape | Keep downstream docs aligned to this statement |
| Unified Agent API adoption | `Decision made; runtime not landed` | ADR and contract work already point toward Unified Agent API; local authority now explicitly confirmed | Add a normal crates.io dependency on `unified-agent-api` and wire real usage into Substrate runtime paths |
| UAA vs local `agent-api-*` boundary | `Clarified conceptually` | Local crates exist for host/world-agent transport; UAA exists separately for CLI agents | Make the naming/ownership boundary explicit in code/docs so future runtime work does not mix them together |
| Agent config and inventory | `Landed` | Inventory schema, capabilities, scope, and derived `backend_id` in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:13) | Integrate inventory selection with live UAA backend registration and launch paths |
| Agent CLI inspection surface | `Partially landed` | `substrate agent list|status|doctor|toolbox status|env` in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:399) and handlers in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:31) | The surfaces are still mostly inspection/projection, not live control-plane actions |
| Orchestrator eligibility rules | `Landed` | Host-only orchestrator validation and capability checks in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1201) | Open a real host orchestrator session through UAA instead of stopping at validation |
| Agent event schema and trace flattening | `Landed` | Tuple-aware `AgentEvent` envelope in [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:62) and persistence in [telemetry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/telemetry.rs:187) | Real producers from orchestrator/member runtime rather than only shell/demo/world-lifecycle projections |
| Nested gateway identity split | `Landed observationally` | `agent status` separates pure-agent rows and nested gateway-backed rows from trace in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:386) | Drive this from live orchestrator/member runtime state, not only trace replay |
| Gateway lifecycle | `Landed for nested LLM runtime` | Real `status|sync|restart` endpoints in [world-agent lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs:265) and shell lifecycle client in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640) | Separate and add agent-session orchestration runtime; current gateway lifecycle is not yet agent-hub session lifecycle |
| Backend selection / allowlisting | `Landed for selection logic` | Inventory-backed selection for gateway backends in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:191) | Apply the same selection and allowlist logic to live UAA backend registration and dispatch for orchestrator/member sessions |
| Live agent session registry | `Missing runtime` | Planning/history exists; observational status surfaces exist | Add a live source of truth for orchestration sessions and member sessions; this does not need to start as a full `/v1/agents` service if a thinner v1 slice is chosen |
| Agent session control plane | `Missing runtime` | ADR/pack semantics exist; UAA defines `run`, `run_control`, and session extensions | Add Substrate runtime paths for `start/resume/fork/stop` that delegate session semantics to UAA and add Substrate-owned orchestration metadata around them |
| Session handles | `Partial` | UAA has canonical session-handle capability/extension semantics; Substrate planning has `AgentSessionHandleV1` semantics | Add a Substrate-owned registry/store that records orchestration session id, role, world binding, and UAA session-handle metadata together |
| Host orchestrator process management | `Missing runtime` | Orchestrator can be selected and validated on paper | Launch and track a real host-scoped orchestrator backend via UAA |
| In-world member dispatch | `Missing runtime` | World scope and fail-closed checks exist | Launch and track one or more member backends via UAA inside the selected shared world boundary |
| World reuse across member sessions | `Partial` | World lifecycle alerting and generation metadata exist in trace/tests | Add actual shared-world ownership and reuse rules for live member sessions |
| Toolbox surface | `Partial` | Config keys exist in [config_model.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/config_model.rs:485); CLI `toolbox status|env` surface now exists; ADR contract exists in [ADR-0045](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:99) | Add the real internal MCP server, auth token flow, and tool-call audit emission |
| Toolbox role in orchestration | `Constrained by design` | ADR-0045 is introspection-only in [ADR-0045](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:77) | Nothing here should be treated as a substitute for member launch or control-plane execution |
| Custom Substrate harness | `Deferred by intent` | Product intent allows for a future Substrate-native harness layer | Do not block v1 on this; revisit only after CLI-agent orchestration through UAA is working cleanly |

## Still-Open Decisions

These are the decisions that still need to be made to keep the path forward clean.

1. Should Substrate’s first shipped runtime slice be:
   - a thin vertical slice:
     - one host orchestrator,
     - one in-world member,
     - one shared-world binding model,
     - and a minimal session registry,
   - or a broader hub service with a general `/v1/agents` registry and richer multi-agent session management from the start?

2. What exact Substrate-owned session record should wrap the UAA session semantics?
   - UAA already defines backend-facing session extensions and session-handle surfacing.
   - Substrate still needs its own orchestration record for:
     - `orchestration_session_id`,
     - role,
     - world binding,
     - world generation,
     - trace correlation,
     - and restart invalidation semantics.

3. Should `uaa.agent.session` remain as a Substrate-local trace/identity token, or should it be replaced?
   - It should not be treated as if it were the canonical upstream UAA protocol token.
   - If it remains, it should be explicitly documented as a Substrate-local identity label rather than an upstream UAA contract surface.

4. How soon should the local `agent-api-*` crates be renamed or otherwise deconflicted?
   - They are already easy to confuse with external `agent_api`.
   - The longer runtime work continues without an explicit naming boundary, the more expensive the cleanup becomes.

## Recommended v1 Runtime Slice

If the goal is to move quickly without overbuilding the control plane, the recommended first executable slice is:

1. Register supported UAA backends from inventory.
2. Open a host-scoped orchestrator session through UAA.
3. Open one world-scoped member session through UAA inside a Substrate-managed shared world.
4. Record a Substrate-owned orchestration session object that wraps:
   - orchestrator/member role,
   - world binding,
   - world generation,
   - and any surfaced UAA session handle.
5. Emit canonical trace/agent events from that runtime.

That slice is enough to prove the product direction without requiring a full general-purpose agent hub service surface up front.
