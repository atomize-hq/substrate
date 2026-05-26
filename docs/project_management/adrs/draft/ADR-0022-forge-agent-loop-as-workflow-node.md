# ADR-0022 — Forge (agent loop) as a composite workflow node

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Substrate maintainers

## Scope
- Feature directory: `docs/project_management/_archived/next/forge/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Plan: `docs/project_management/_archived/next/forge/plan.md` (not created; ADR draft phase)
- Tasks: `docs/project_management/_archived/next/forge/tasks.json` (not created; ADR draft phase)
- Spec manifest: `docs/project_management/_archived/next/forge/spec_manifest.md` (not created; ADR draft phase)
- Contract (if present): `docs/project_management/_archived/next/forge/contract.md` (not created; ADR draft phase)
- Decision Register: `docs/project_management/_archived/next/forge/decision_register.md` (required before Accepted)
- Impact Map: `docs/project_management/_archived/next/forge/impact_map.md` (not created; ADR draft phase)
- Manual Playbook: `docs/project_management/_archived/next/forge/manual_testing_playbook.md` (not created; ADR draft phase)
- Dependency foundations (must remain compatible):
  - Workflow engine (node executor hook): `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`
  - LLM gateway front door: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - Backend/provider engines: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Trace/event foundations: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Config/policy surface (no new roots): `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 22f105ab816130a537072cd0fca3a75dec6c106edd965fc161ad57b34d1f22fe
### Changes (operator-facing)
- Introduce Forge as a first-class “agent loop” node kind usable inside Substrate workflows
  - Existing: Substrate does not provide a native “LLM-driven critique/refine/review loop” runner; operators wire together agent loops externally.
  - New: A workflow can include a `forge.run` node that performs bounded iterative refinement (execute → critique → refine → review with retries) under explicit budgets, emitting nested spans/events for observability.
  - Why: keep the general workflow engine simple (DAG scheduling) while enabling advanced looping/leadership behavior as a reusable composite node.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md#L1`
    - `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md#L1`
    - `docs/project_management/_archived/next/forge/decision_register.md`

## Problem / Context
- Forge (as implemented in the current Python repo) is a specialized orchestration loop with role-based prompts/config, retries, and “leadership” adjustments.
- Substrate is expected to gain a general workflow engine for DAG execution. Forge should not become the workflow engine; it should be a reusable node type within it.
- Forge must integrate via Substrate’s stable LLM gateway and trace/event contracts, without embedding provider SDKs or inventing a new event schema.

## Goals
- Provide a Rust Forge implementation that:
  - exposes a library API (no required CLI surface in the first cut),
  - runs a bounded multi-step agent loop with clear stop conditions and budgets,
  - is provider/gateway agnostic (calls an `LlmGateway` trait),
  - emits structured events/spans that nest cleanly under a workflow node span.
- Ensure Forge can be embedded as a workflow node executor (composite node) without dictating workflow DAG semantics.

## Non-Goals
- Do not implement a full provider/model catalog inside Forge; model/provider definition belongs in a separate crate/service.
- Do not invent a new “agent event schema” for Forge; Forge emits spans/events using Substrate’s trace/event foundations.
- Do not implement tool execution inside Forge in the MVP (tool calls should remain explicit workflow nodes unless/until a dedicated tool-call interface is standardized).
- Do not implement reinforcement learning training pipelines in the initial Rust Forge.

## User Contract (Authoritative)

### CLI
- No new top-level CLI commands are introduced in the initial Forge MVP.
- Forge is exposed as a workflow node kind (`forge.run`).
- `crates/forge-core` is a library crate and is not a stable public API in v1.

### Config
- `forge.run` node config is part of the workflow spec file defined by `ADR-0021`.
- Strict config schema (v1; unknown keys rejected):
  - Node inputs (in the workflow spec node `inputs`):
    - `task: <string>` (required)
    - `context?: <string>` (optional; default empty)
  - Node config (in the workflow spec node `config`):
    - `schema_version: 1`
    - `max_attempts: <u32>` (required; must be `>= 1`)
    - `budgets?: { max_runtime_ms?: <u64>, max_tokens?: <u64> }`
      - If `max_tokens` is set but the gateway does not return token usage metadata, Forge MUST fail closed with an explicit error.
    - `roles: { execute, critique, refine, review }` (required)
      - Each role value is a strict object:
        - `template: <string>` (required; supports placeholders described below)
  - Template placeholders (v1):
    - `${task}`, `${context}`, `${attempt}`
    - `${execute}`, `${critique}`, `${refine}`, `${review}` (the prior step outputs within the same attempt)
  - Step output contract (v1; deterministic):
    - Each role output MUST contain a `FINAL:` marker.
    - Forge extracts the role output as:
      1) remove any `<think>…</think>` blocks,
      2) take the content starting at the last `FINAL:` marker (inclusive of any following lines),
      3) trim leading whitespace on the first extracted line and trim trailing whitespace at the end.
    - Missing `FINAL:` is a step failure and causes the attempt to be marked FAIL.
    - Review PASS/FAIL contract (v1; deterministic):
      - The extracted `review` block’s first line MUST be exactly `FINAL: PASS` or `FINAL: FAIL` (case-sensitive).
      - PASS ends the loop. Forge `result` is the extracted content after the PASS line (may be empty).
      - FAIL marks the attempt FAIL (and triggers retry if attempts remain).

### Platform guarantees
- Forge must behave consistently across Linux/macOS/Windows because:
  - it is pure orchestration and eventing, and
  - it delegates all network access (LLM calls) to the gateway adapter used by the host process.
- If the gateway is unavailable, Forge must fail deterministically with an explicit error result and a recorded span chain.

## Architecture Shape
- Components (new crates):
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
      - implements `LlmGateway` by calling the Substrate LLM gateway/proxy client (Phase 4; see ADR-0023/ADR-0024).
  - `crates/workflow-runtime`:
    - provides a `NodeExecutor` implementation for node kind `forge.run` that:
      - creates a node span,
      - invokes `forge-core`,
      - maps forge step spans as children (or uses `graph_edges` when appropriate).
- End-to-end flow:
  - Inputs: a `forge.run` node invocation (task + budgets + role templates) and a parent node span id.
  - Derived state: resolved role templates, resolved gateway routing info (via adapter), per-attempt budget state.
  - Actions:
    - run `execute → critique → refine → review`,
    - if FAIL and attempts remain: retry the full 4-step loop under budgets,
    - stop when PASS or budgets/attempts exhausted.
  - Outputs:
    - final `result` string (on PASS: extracted content after the `FINAL: PASS` review line; on FAIL: empty),
    - per-step extracted outputs + attempt counters,
    - nested spans/events suitable for replay/inspection.

## Sequencing / Dependencies
- Sequencing entry: this ADR must add a `forge` entry to `docs/project_management/packs/sequencing.json` before it can be marked `Accepted`.
- Prerequisite integration task IDs: none yet (this ADR is Draft). Before `Accepted`, this section must be updated to reference the Planning Pack task IDs for forge.
- Dependencies:
  - Workflow engine (`ADR-0021`) provides the host DAG runtime and the `forge.run` node hook point.
  - Forge depends on the Substrate LLM gateway client surface, but does not embed provider SDKs (ADR-0023/ADR-0024).

## Security / Safety Posture
- Fail-closed rules:
  - If budgets are exceeded (runtime/tokens), Forge MUST abort the loop and return a failed node result.
  - If the gateway call fails, Forge MUST:
    - record the error in the relevant step span/event (redacting sensitive data),
    - either retry (if allowed by retry policy) or fail deterministically.
- Protected paths/invariants:
  - Forge core must not read/write arbitrary filesystem paths by default.
  - Any artifact persistence must be delegated to a store interface and constrained to Substrate-owned directories (to be specified in the Planning Pack).
- Observability requirements:
  - Emit stable step names (`execute`, `critique`, `refine`, `review`) and attempt counters.
  - Every gateway call must be attributable to a step span with correlation to the parent workflow node span.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - budget enforcement (attempt count, runtime budget, token accounting when provided by gateway),
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
- Decision Register: `docs/project_management/_archived/next/forge/decision_register.md`
  - DR-0001: Loop shape + step semantics
  - DR-0002: Budget enforcement + fail-closed rules
  - DR-0003: Output extraction contract (`FINAL:` marker) vs JSON-only
  - DR-0004: Artifact persistence posture (trace-only vs file store)
  - DR-0005: Event/streaming surface (workflow-runtime sink vs direct emitter)

## Appendix — Python Forge parity checklist (triage A/B/C)

This appendix is a deliberate “parity inventory” against the current Python Forge repo.
Each line item below needs triage into one bucket:
- **A**: required in Rust Forge v1 (must ship in the initial Forge implementation)
- **B**: deferred (explicitly not in v1; tracked for later)
- **C**: belongs outside Forge (Workflow Engine / Gateway / Config / Trace / Store crates)

Triage below is authoritative for Forge v1.

| Area | Capability | Observed in Python Forge | Proposed home in Substrate | Notes | Triage (A/B/C) |
|---|---|---|---|---|---|
| Core loop | `execute → critique → refine → review` with retries | Yes | `forge-core` | Primary loop semantics; deterministic stop conditions | A |
| Core loop | `monitor` adjustments + `reflect` step | Yes | `forge-core` | Deferred from v1 (no monitor/reflect steps in v1) | B |
| Budgets | `max_attempts` retry ceiling | Yes | `forge-core` | Enforced in core; retry boundary is per attempt | A |
| Budgets | runtime caps (per-run / per-role timeouts) | Partial (CLI heartbeat, env vars, max attempts) | `forge-core` + `workflow-core` | v1 supports per-node runtime budget only (per-role timeouts deferred) | A |
| Budgets | token/cost caps | Partial (estimation + reporting) | `forge-core` + `forge-substrate` | v1 supports token caps; cost caps are deferred | A |
| Output hygiene | `FINAL:` extraction and `<think>` stripping | Yes | `forge-core` (utilities) | Required for deterministic extraction | A |
| Provider selection | role-based provider selection (execute/critique/…) | Yes | **C**: `gateway`/catalog + workflow node config | Provider/model routing is a gateway/catalog concern | C |
| Config resolution | hierarchical config (defaults → role → model/role wildcard → runtime overrides) | Yes | **C**: `forge-config` + provider/model catalog | Substrate-wide config layering belongs outside Forge | C |
| Lazy initialization | true lazy provider init + optional prewarm | Yes (module exists) | **C**: gateway / provider catalog | Gateway client/pool concern | C |
| Checkpointing | memory/sqlite checkpointing of graph execution | Yes (LangGraph checkpointers) | **C**: `workflow-core` / `workflow-runtime` | Workflow-engine concern | C |
| Streaming | stream “node start/end” progress events | Yes | **C**: `workflow-runtime` + `forge-substrate` | Workflow runtime owns streaming transport | C |
| Streaming | verbose callback/event stream filtering | Yes (CLI filter) | **C**: workflow CLI layer | CLI concern | C |
| Run identity | stable run id / thread id | Yes | `workflow-runtime` + `forge-core` | Correlates to parent workflow node run | A |
| Telemetry | per-step timings | Yes | `forge-core` + `forge-substrate` | Required for observability | A |
| Telemetry | provider/model breakdown per role | Yes (CLI summary) | **C**: gateway + workflow CLI | Gateway provides ground truth | C |
| Telemetry | token usage extraction (provider/library responses) | Yes | **C**: gateway | Gateway normalizes usage across providers | C |
| Telemetry | cost estimation table by model | Yes (best-effort) | **C**: gateway/catalog | Centralize volatile pricing | C |
| Persistence | persistence manager storing perf/leadership stats across runs | Yes | **C**: Substrate runtime/telemetry | v1 relies on trace only | C |
| Leadership | separate “leadership orchestrator” (provider selection + parameter tuning decisions) | Yes | Split: `forge-core` loop policy + **C** gateway/catalog | Deferred from v1 | B |
| Parameter tuning | dynamic per-role kwargs adjustments across retries | Yes | `forge-core` + **C** gateway | Deferred from v1 | B |
| RL hooks | meta-learning reward computation and policy update placeholder | Yes (placeholder) | **B** or **C** (separate crate) | Deferred from v1 | B |
| CLI | `anvil run` command and rich summary output | Yes | **C**: Substrate CLI / workflow CLI | CLI concern | C |
| CLI | `list`, `test`, `hotswap-demo` | Yes | **C**: Substrate CLI / gateway tooling | Tooling concern | C |
| Artifacts | structured per-run artifacts (plan, deltas, ledgers) | Partial (logs/state only) | `forge-core` + **C** store | Deferred from v1 | B |
| Run Store | durable run store database (vNext roadmap) | No (in Python baseline) | **C**: workflow/store | Workflow/store concern | C |
