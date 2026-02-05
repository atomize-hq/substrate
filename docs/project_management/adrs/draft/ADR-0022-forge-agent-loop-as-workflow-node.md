# ADR-0022 — Forge (agent loop) as a composite workflow node

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Substrate maintainers

## Scope
- Feature directory: `docs/project_management/next/forge/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Plan: `docs/project_management/next/forge/plan.md` (not created; ADR draft phase)
- Tasks: `docs/project_management/next/forge/tasks.json` (not created; ADR draft phase)
- Spec manifest: `docs/project_management/next/forge/spec_manifest.md` (not created; ADR draft phase)
- Contract (if present): `docs/project_management/next/forge/contract.md` (not created; ADR draft phase)
- Decision Register: `docs/project_management/next/forge/decision_register.md` (required before Accepted; not created; ADR draft phase)
- Impact Map: `docs/project_management/next/forge/impact_map.md` (not created; ADR draft phase)
- Manual Playbook: `docs/project_management/next/forge/manual_testing_playbook.md` (not created; ADR draft phase)

## Executive Summary (Operator)

ADR_BODY_SHA256: 4b06590a244058a7f96eb3b0130afb88ddd693f18501f50c6391dfbabb1ee14f
### Changes (operator-facing)
- Introduce Forge as a first-class “agent loop” node kind usable inside Substrate workflows
  - Existing: Substrate does not provide a native “LLM-driven critique/refine/review loop” runner; operators wire together agent loops externally.
  - New: A workflow can include a `forge.run` node that performs bounded iterative refinement (execute → critique → refine → review with retries) under explicit budgets, emitting nested spans/events for observability.
  - Why: keep the general workflow engine simple (DAG scheduling) while enabling advanced looping/leadership behavior as a reusable composite node.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`
    - `crates/common/src/agent_events.rs`
    - `crates/trace/src/span.rs`

## Problem / Context
- Forge (as implemented in the current Python repo) is a specialized orchestration loop with role-based prompts/config, retries, and “leadership” adjustments.
- Substrate is expected to gain a general workflow engine for DAG execution. Forge should not become the workflow engine; it should be a reusable node type within it.
- The LLM gateway/proxy and the final agent-event payload shapes are still being finalized; Forge must be able to integrate without hard-coding to unstable external schemas.

## Goals
- Provide a Rust Forge implementation that:
  - exposes a library API (no required CLI surface in the first cut),
  - runs a bounded multi-step agent loop with clear stop conditions and budgets,
  - is provider/gateway agnostic (calls an `LlmGateway` trait),
  - emits structured events/spans that nest cleanly under a workflow node span.
- Ensure Forge can be embedded as a workflow node executor (composite node) without dictating workflow DAG semantics.

## Non-Goals
- Do not implement a full provider/model catalog inside Forge; model/provider definition belongs in a separate crate/service.
- Do not require finalization of the “agent event payload schema” to build Forge; only require correlation + nesting.
- Do not implement tool execution inside Forge in the MVP (tool calls should remain explicit workflow nodes unless/until a dedicated tool-call interface is standardized).
- Do not implement reinforcement learning training pipelines in the initial Rust Forge.

## User Contract (Authoritative)

### CLI
- No new top-level CLI commands are introduced in the initial Forge MVP.
- Forge is exposed as a workflow node kind (`forge.run`) and/or as a library API for internal callers.

### Config
- Forge configuration surface is split intentionally:
  - Workflow-level: `forge.run` node config references a forge “profile” and provides budgets.
  - Provider/model-level: resolved by an external gateway/catalog (out of scope for Forge).
- Minimum `forge.run` node config (schema is part of `workflow-types` spec):
  - `profile: <string>` (e.g., `default`, `planning`, `coding`)
  - `max_attempts: <u32>`
  - `budgets: { max_runtime_ms?, max_tokens?, max_cost_usd? }`
  - `roles: { execute, critique, refine, review, reflect }`:
    - each role may specify output constraints (max chars/tokens) and validation mode.

### Platform guarantees
- Forge must behave consistently across Linux/macOS/Windows because:
  - it is pure orchestration and eventing, and
  - it delegates all network access (LLM calls) to the gateway adapter used by the host process.
- If the gateway is unavailable, Forge must fail deterministically with an explicit error result and a recorded span chain.

## Architecture Shape
- Components (new crates; names are proposals):
  - `crates/forge-types`:
    - role enums, request/response structs, event envelopes (serde-only).
  - `crates/forge-core`:
    - loop runner, budget manager, stop conditions, validation/repair hooks (no HTTP, no SDKs).
    - traits:
      - `LlmGateway`: minimal chat/completion surface (sync/async + optional streaming).
      - `ForgeEventSink`: emit forge events; default no-op.
  - `crates/forge-substrate`:
    - adapter layer that:
      - implements `ForgeEventSink` by writing spans/events via `substrate-trace` and/or `substrate_common::agent_events::AgentEvent`,
      - implements `LlmGateway` by calling the Substrate LLM gateway/proxy client (to be defined elsewhere).
  - `crates/workflow-runtime`:
    - provides a `NodeExecutor` implementation for node kind `forge.run` that:
      - creates a node span,
      - invokes `forge-core`,
      - maps forge step spans as children (or uses `graph_edges` when appropriate).
- End-to-end flow:
  - Inputs: a `forge.run` node invocation (task + profile + budgets) and a parent node span id.
  - Derived state: selected role prompts/config (from profile), resolved gateway routing info (via adapter), per-attempt budget state.
  - Actions:
    - run `execute → critique → refine → review`,
    - if FAIL and attempts remain: emit `monitor/reflect` step (optional) and retry under budgets,
    - stop when PASS or budgets/attempts exhausted.
  - Outputs:
    - final “result” string (or structured output when configured),
    - per-role artifacts (optional) and step summaries,
    - nested spans/events suitable for replay/inspection.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → TBD
- Prerequisite integration task IDs:
  - TBD (to be created in the Planning Pack task set)
- Dependencies:
  - Workflow engine (`ADR-0021`) provides the host DAG runtime and the `forge.run` node hook point.
  - LLM gateway/proxy contract is external; Forge depends only on the `LlmGateway` trait and the adapter crate, not on the final gateway schema.

## Security / Safety Posture
- Fail-closed rules:
  - If budgets are exceeded (runtime/tokens/cost), Forge MUST abort the loop and return a failed node result.
  - If the gateway call fails, Forge MUST:
    - record the error in the relevant step span/event (redacting sensitive data),
    - either retry (if allowed by retry policy) or fail deterministically.
- Protected paths/invariants:
  - Forge core must not read/write arbitrary filesystem paths by default.
  - Any artifact persistence must be delegated to a store interface and constrained to Substrate-owned directories (to be specified in the Planning Pack).
- Observability requirements:
  - Emit stable step names (`execute`, `critique`, `refine`, `review`, `reflect`, `monitor`, `finalize`) and attempt counters.
  - Every gateway call must be attributable to a step span with correlation to the parent workflow node span.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - budget enforcement (attempt count, runtime budget, token/cost accounting when provided by gateway),
  - stop conditions (PASS ends loop; FAIL retries until max).
- Integration tests:
  - mock `LlmGateway` with scripted responses to drive deterministic loop behavior.
  - verify span nesting and correlation fields via a test `ForgeEventSink`.

### Manual playbook
- Provide a manual playbook (required before Accepted) that demonstrates:
  - one `forge.run` node inside a small workflow,
  - a forced FAIL→retry→PASS run,
  - a budget-exceeded abort with clear trace output.

## Rollout / Backwards Compatibility
- Default: greenfield addition.
- Forge is introduced as a workflow node kind; no existing Substrate behavior is changed.
- Forge config schemas must be versioned and validated; unknown schema versions fail with exit `2` (invalid config/spec).

## Decision Summary
- Forge is explicitly modeled as a composite node inside the workflow engine rather than a general workflow runtime.
- Forge depends on a minimal gateway trait and an adapter crate instead of embedding provider SDKs directly.
- Before this ADR is moved to Accepted, a decision register will be created at:
  - `docs/project_management/next/forge/decision_register.md`
  - to record A/B decisions (e.g., artifact storage policy, streaming event surface, role config schema).

## Appendix — Python Forge parity checklist (triage A/B/C)

This appendix is a deliberate “parity inventory” against the current Python Forge repo.
Each line item below needs triage into one bucket:
- **A**: required in Rust Forge v1 (must ship in the initial Forge implementation)
- **B**: deferred (explicitly not in v1; tracked for later)
- **C**: belongs outside Forge (Workflow Engine / Gateway / Config / Trace / Store crates)

Triage is intentionally left as **TBD** for now.

| Area | Capability | Observed in Python Forge | Proposed home in Substrate | Notes | Triage (A/B/C) |
|---|---|---|---|---|---|
| Core loop | `execute → critique → refine → review` with retries | Yes | `forge-core` | Primary loop semantics; deterministic stop conditions | TBD |
| Core loop | `monitor` adjustments + `reflect` step | Yes | `forge-core` | Optional but present in current graph; may be simplified | TBD |
| Budgets | `max_attempts` retry ceiling | Yes | `forge-core` | Must be enforced inside Forge loop | TBD |
| Budgets | runtime caps (per-run / per-role timeouts) | Partial (CLI heartbeat, env vars, max attempts) | `forge-core` + `workflow-core` | Decide which caps are enforced by workflow runner vs forge loop | TBD |
| Budgets | token/cost caps | Partial (estimation + reporting) | `forge-core` + `forge-substrate` | Forge core can track counters; gateway provides usage/cost metadata | TBD |
| Output hygiene | `FINAL:` extraction and `<think>` stripping | Yes | `forge-core` (utilities) | Should be explicit contract for downstream determinism | TBD |
| Provider selection | role-based provider selection (execute/critique/…) | Yes | **C**: `gateway`/catalog + workflow node config | Substrate-wide: provider/model definitions should live outside Forge | TBD |
| Config resolution | hierarchical config (defaults → role → model/role wildcard → runtime overrides) | Yes | **C**: `forge-config` + provider/model catalog | Forge should consume resolved config; not own provider catalogs | TBD |
| Lazy initialization | true lazy provider init + optional prewarm | Yes (module exists) | **C**: gateway / provider catalog | For Rust: belongs to gateway client/pool; not forge core | TBD |
| Checkpointing | memory/sqlite checkpointing of graph execution | Yes (LangGraph checkpointers) | **C**: `workflow-core` / `workflow-runtime` | For workflow DAG: checkpointing/resume is workflow-engine concern | TBD |
| Streaming | stream “node start/end” progress events | Yes | **C**: `workflow-runtime` + `forge-substrate` | Forge emits step events; workflow owns streaming transport surface | TBD |
| Streaming | verbose callback/event stream filtering | Yes (CLI filter) | **C**: workflow CLI layer | UI/CLI policy; not forge core | TBD |
| Run identity | stable run id / thread id | Yes | `workflow-runtime` + `forge-core` | Forge run id should correlate to parent workflow node run | TBD |
| Telemetry | per-step timings | Yes | `forge-core` + `forge-substrate` | Store timings in forge result; emit spans for timings | TBD |
| Telemetry | provider/model breakdown per role | Yes (CLI summary) | **C**: gateway + workflow CLI | Forge can surface “what it was told”; gateway provides ground truth | TBD |
| Telemetry | token usage extraction (provider/library responses) | Yes | **C**: gateway | Gateway should normalize usage across providers; Forge consumes usage | TBD |
| Telemetry | cost estimation table by model | Yes (best-effort) | **C**: gateway/catalog | Pricing is volatile; should be centralized | TBD |
| Persistence | persistence manager storing perf/leadership stats across runs | Yes | **C**: Substrate runtime/telemetry | Substrate already has trace persistence; extra stats may be workflow-level | TBD |
| Leadership | separate “leadership orchestrator” (provider selection + parameter tuning decisions) | Yes | Split: `forge-core` loop policy + **C** gateway/catalog | Keep the “loop policy” in Forge; selection policy likely central | TBD |
| Parameter tuning | dynamic per-role kwargs adjustments across retries | Yes | `forge-core` + **C** gateway | Forge can request adjustments; gateway enforces/validates | TBD |
| RL hooks | meta-learning reward computation and policy update placeholder | Yes (placeholder) | **B** or **C** (separate crate) | Likely deferred; if kept, isolate in optional crate/feature | TBD |
| CLI | `anvil run` command and rich summary output | Yes | **C**: Substrate CLI / workflow CLI | Forge as library/node; Substrate owns CLI UX | TBD |
| CLI | `list`, `test`, `hotswap-demo` | Yes | **C**: Substrate CLI / gateway tooling | These map better to gateway diagnostics | TBD |
| Artifacts | structured per-run artifacts (plan, deltas, ledgers) | Partial (logs/state only) | `forge-core` + **C** store | Forge should define artifact types; store is separate crate | TBD |
| Run Store | durable run store database (vNext roadmap) | No (in Python baseline) | **C**: workflow/store | This is future vNext; not parity-required | TBD |
