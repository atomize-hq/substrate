# ADR-0021 — Substrate Workflow Engine (DAG runner + node executors)

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Substrate maintainers

## Scope
- Feature directory: `docs/project_management/next/workflow-engine/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/CONTRACT_SURFACE_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/workflow-engine/plan.md` (not created; ADR draft phase)
- Tasks: `docs/project_management/next/workflow-engine/tasks.json` (not created; ADR draft phase)
- Spec manifest: `docs/project_management/next/workflow-engine/spec_manifest.md` (not created; ADR draft phase)
- Contract (if present): `docs/project_management/next/workflow-engine/contract.md` (not created; ADR draft phase)
- Decision Register: `docs/project_management/next/workflow-engine/decision_register.md` (required before Accepted; not created; ADR draft phase)
- Impact Map: `docs/project_management/next/workflow-engine/impact_map.md` (not created; ADR draft phase)
- Manual Playbook: `docs/project_management/next/workflow-engine/manual_testing_playbook.md` (not created; ADR draft phase)

## Executive Summary (Operator)

ADR_BODY_SHA256: f3b261e6d6b0b1463b43542e6a524e8d79567d812c32d35224f8f5256aecbe2e
### Changes (operator-facing)
- Add a first-class “workflow run” capability to Substrate
  - Existing: Substrate executes single commands (interactive or non-interactive) and records spans for replay/audit; operators orchestrate multi-step flows outside of Substrate (scripts, makefiles, ad-hoc tooling).
  - New: Substrate can run a user-defined DAG workflow made of heterogeneous nodes (agent calls, tool/script exec, HTTP calls, sub-workflows) while emitting a single traceable workflow run with per-node spans and replay hooks.
  - Why: make multi-step automation observable/replayable under the same policy+trace model, and provide a stable substrate for “agentic workflows” without coupling to any specific agent framework.
  - Links:
    - `crates/trace/src/span.rs`
    - `crates/agent-api-types/src/lib.rs`
    - `crates/common/src/agent_events.rs`

## Problem / Context
- Substrate already provides secure execution, policy enforcement, and trace/replay, but it does not provide a native workflow graph runner.
- Users want to define and execute DAG-like workflows composed of AI agents, tools, scripts, and other actions, and have those workflows be observable and replayable under Substrate’s existing guarantees.
- If workflow orchestration stays “outside” Substrate, replay/trace becomes fragmented across ad-hoc runners, and policy enforcement/attribution becomes harder to reason about for multi-step agent systems.

## Goals
- Provide a workflow runtime that:
  - executes a DAG of nodes with explicit dependencies,
  - supports fanout/join and bounded concurrency,
  - records a stable trace structure for “workflow run → node run → subspans” using `substrate-trace`,
  - is extensible via a node-executor interface (new node kinds without rewriting the scheduler),
  - can be used as a library (and optionally surfaced via CLI).
- Ensure workflow execution remains compatible with Substrate’s security posture:
  - tool/script execution must still go through policy evaluation and world isolation,
  - failures and denies must be observable and attributable to a node/span.

## Non-Goals
- Do not introduce distributed execution or cross-host scheduling in the initial version.
- Do not standardize the final “agent event payload schema” in this ADR (only require span nesting + correlation fields).
- Do not require a durable workflow database for the MVP; “resume after process restart” is out of scope for the initial cut.
- Do not embed a third-party workflow framework as the canonical runtime (see Decision Summary).

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate workflow validate <workflow_spec_path>`:
    - validates schema + DAG invariants (acyclic, known node ids, resolvable references).
    - exit codes:
      - `0`: valid
      - `2`: invalid spec / failed validation
      - `5`: unexpected internal error
  - `substrate workflow run <workflow_spec_path> [--input <k=v>...] [--concurrency <n>]`:
    - runs the workflow DAG and streams node-level progress.
    - defaults:
      - concurrency defaults to a safe value (e.g., `min(4, num_cpus)`), but must be bounded.
    - exit codes:
      - `0`: workflow completed successfully (all required nodes succeeded)
      - `3`: workflow failed (one or more nodes failed and the workflow terminated)
      - `4`: workflow denied by policy (a node requested execution that was denied and the workflow is fail-closed)
      - `5`: unexpected internal error
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (unless explicitly overridden here)

### Config
- Workflow spec file:
  - file format: YAML or JSON (YAML recommended), loaded from `<workflow_spec_path>`.
  - schema requirements (MVP):
    - `schema_version: 1`
    - `workflow_id: <string>`
    - `nodes: [{ id, kind, inputs?, config? }, ...]`
    - `edges: [{ from, to }, ...]` (acyclic; defines dependencies)
    - `outputs?: { <name>: <ref> }` (optional; references node outputs)
    - `defaults?: { budget?, policy_profile?, world_fs_mode? }` (optional)
- Precedence:
  - CLI flags override spec defaults for runtime knobs (e.g., concurrency, inputs).
  - Policy/profile selection follows Substrate’s existing policy/config precedence rules; the workflow runner may pass an explicit profile into node executors, but does not redefine policy resolution.

### Platform guarantees
- Linux/macOS/Windows:
  - The workflow runner must not bypass policy evaluation and must not execute commands directly on the host outside of Substrate’s normal execution mechanisms.
  - Streaming output must be supported when the underlying node kind supports it (e.g., command execution via Agent API stream).
  - Unsupported node kinds on a platform must fail deterministically with an explicit error result and a node span describing the reason.

## Architecture Shape
- Components (new crates; names are proposals):
  - `crates/workflow-types`:
    - versioned spec structs + result/event structs (serde-only; no IO).
  - `crates/workflow-core`:
    - DAG validation and scheduling semantics (acyclic enforcement, topo order, concurrency gates, retry policy).
    - node executor trait(s) (e.g., `NodeExecutor` per `NodeKind`).
    - internal graph representation uses `petgraph` as a library detail.
  - `crates/workflow-runtime`:
    - concrete executors that integrate with existing Substrate capabilities (Agent API, trace spans, policy profile selection).
  - `crates/trace`:
    - used to emit workflow and node spans; node spans must be linked to underlying command spans using `graph_edges` and/or parent span relationships where applicable.
- End-to-end flow:
  - Inputs: `workflow_spec_path`, optional CLI inputs, current working directory, policy/profile environment.
  - Derived state: validated DAG + execution plan + per-node runtime context.
  - Actions:
    - create a workflow-run root span,
    - schedule node execution with bounded concurrency,
    - for each node, create a node-run span and invoke the node executor,
    - record success/failure and any produced artifacts/outputs.
  - Outputs:
    - human-readable streaming progress,
    - workflow result summary (success/failure and per-node statuses),
    - trace spans written to `~/.substrate/trace.jsonl` (default path) with graph linkage.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → TBD
- Prerequisite integration task IDs:
  - TBD (to be created in the Planning Pack task set)
- Dependencies:
  - Reuse existing Agent API request/response + streaming frame model (`crates/agent-api-*`) for tool/script execution nodes.
  - Reuse `substrate-trace` for span persistence and replay association.

## Security / Safety Posture
- Fail-closed rules:
  - If a node requests command/tool execution and policy evaluation denies it, the workflow MUST fail-closed by default (exit `4`) and record:
    - node span with `policy_decision.action = "deny"` (or equivalent structured field),
    - the reason in span metadata without leaking secrets.
- Protected paths/invariants:
  - Workflow execution must not write outside the allowed filesystem scopes; command execution remains under existing world isolation rules.
  - Workflow runner must never execute arbitrary host commands directly; all execution goes through existing Substrate execution pathways.
- Observability:
  - Every node execution creates a traceable span with:
    - `parent_span` = workflow root span,
    - stable node id in metadata,
    - timing, status, and (when applicable) linkages to underlying command spans via `graph_edges`.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - DAG validation: cycles rejected; unknown node ids rejected; disconnected nodes behavior is explicit.
  - Scheduling: bounded concurrency enforcement; deterministic error propagation.
- Integration tests:
  - Minimal workflow with two command nodes via Agent API mock executor emits:
    - workflow root span,
    - node spans,
    - linked command spans (or mock links).
  - Cross-platform: verify identical spec validation semantics (OS-independent).

### Manual playbook
- Provide a manual playbook (required before Accepted) that validates:
  - a small workflow run with streaming output,
  - deny behavior (policy deny causes workflow fail-closed),
  - trace inspection (`~/.substrate/trace.jsonl` contains workflow+node spans).

## Rollout / Backwards Compatibility
- Default: greenfield addition.
- No existing CLI contract is changed in the MVP; the workflow command is additive.
- Any workflow spec schema changes must be versioned (`schema_version`) and old versions must either:
  - remain supported, or
  - fail with a clear “unsupported schema version” error and exit `2`.

## Decision Summary
- This ADR intentionally avoids adopting a third-party workflow framework (e.g., durable workflow engines or opinionated agent-graph runtimes) as the canonical Substrate workflow runtime.
  - Rationale: Substrate requires tight coupling to its trace/replay/policy model; framework-imposed eventing/persistence models would either be bypassed or force Substrate to contort/duplicate its observability surface.
- Before this ADR is moved to Accepted, a decision register will be created at:
  - `docs/project_management/next/workflow-engine/decision_register.md`
  - to record A/B decisions (e.g., spec format YAML-only vs YAML+JSON, fail-closed defaults, durability scope).
