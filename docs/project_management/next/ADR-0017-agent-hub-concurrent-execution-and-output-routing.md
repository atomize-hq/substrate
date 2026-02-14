# ADR-0017 — Agent Hub Concurrent Execution and Output Routing

## Status
- Status: Draft
- Date (UTC): 2026-01-25
- Owner(s): Substrate maintainers

## Scope
- Feature directory: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`

## Related Docs
- Plan: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/plan.md`
- Decision Register: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md`
- Related ADRs:
  - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- Historical context:
  - `docs/project_management/_archived/p0-agent-hub-isolation-hardening/`
- Grounding code references:
  - Concurrent structured output in REPL: `crates/shell/src/execution/agent_events.rs`, `crates/shell/src/repl/async_repl.rs`
  - Example command that emits concurrent events: search for `:demo-agent`
  - World PTY byte streaming plumbing: `crates/world-agent/src/pty.rs`

## Executive Summary (Operator)

ADR_BODY_SHA256: 41fcd002c7e017054b2e4812420598a60ad6041277ebcfc9e41f881b5a83b29f
### Changes (operator-facing)
- Make concurrent outputs predictable and non-corrupting when multiple agents run
  - Existing: Substrate can render concurrent **structured** agent output during the REPL (e.g., `:demo-agent`), but there is no explicit output contract that separates:
    - raw PTY byte streams (world sessions, TUIs), from
    - structured “agent hub” events (status/progress/log lines).
  - New: Substrate treats these as distinct output classes with explicit rendering rules:
    - PTY bytes are forwarded as bytes (binary-safe) and are never mixed with structured event printing.
    - Structured agent events are rendered via a structured output path and are buffered during PTY passthrough to avoid corrupting TUIs.
  - Why: Agent hub orchestration will run multiple agent CLIs concurrently (via bindings/SDK wrappers). Without an output contract, concurrent outputs can corrupt terminal state or be mis-attributed, undermining usability and auditability.
  - Links:
    - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md#user-contract-authoritative`
    - `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`
    - `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`

## Problem / Context
- Substrate’s future direction includes an “agent hub” that runs multiple concurrent agent CLIs (Codex / Claude Code / Gemini CLI and others) via wrappers/bindings.
- Substrate already supports concurrent **structured** event printing in the REPL today, but it is string-based and not designed to interleave with raw PTY byte streams.
- The world-first REPL work (ADR-0016) introduces a long-lived world PTY byte stream where:
  - bytes can arrive while the REPL is idle (out-of-band output),
  - and PTY passthrough must preserve TUIs (no injected host messages into PTY bytes).
- Without an explicit output routing contract, concurrency will cause:
  - terminal corruption (especially during TUIs),
  - confusing “mystery output” without attribution,
  - and inconsistent operator expectations as agent hub capabilities expand.

## Goals
- Define an explicit output routing contract that separates:
  - PTY byte streams (binary-safe), and
  - structured agent hub events (typed, attributable).
- Ensure concurrent structured events never corrupt PTY passthrough / TUIs.
- Ensure out-of-band PTY bytes can be rendered while the line editor is active without corrupting the current input buffer.
- Provide a forward-compatible surface for future UI work (including block-based terminal UIs), without changing core execution semantics.

## Non-Goals
- Designing the full agent hub configuration model (providers, credentials, tool installation).
- Providing a full block-based terminal UI as part of this ADR.
- Solving job control/backgrounding in the REPL (out of scope for ADR-0016 and remains out of scope here).
- Adding new world-agent wire protocols beyond what ADR-0016 requires.

## User Contract (Authoritative)
- Output classes:
  - **PTY output**:
    - Raw bytes from a PTY stream (world sessions, TUIs, interactive commands).
    - Must be forwarded and rendered as bytes (binary-safe).
  - **Structured agent events**:
    - Substrate-managed, typed events associated with background/concurrent agent activity (status/progress/log lines).
    - Must be rendered via a structured output path that does not require injecting bytes into a PTY stream.

- Rendering rules (interactive REPL):
  - During PTY passthrough (TUIs/interactive commands), Substrate MUST NOT inject structured agent events into the PTY byte stream.
    - Structured events MUST be handled via a bounded buffer and MUST NOT backpressure execution.
      - If the buffer overflows, Substrate MUST drop additional structured lines for the duration of the passthrough and MUST emit an explicit dropped-count summary after passthrough ends.
        - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0006)
        - Phase 8 additive clarification: when `channel` is used, the dropped-count summary SHOULD optionally include a per-channel breakdown so suppressed output remains explainable without heuristics.
  - While the line editor is active (`Idle`), PTY bytes MAY arrive (out-of-band) and Substrate MUST render them without corrupting the current input buffer.
    - PTY bytes MUST be rendered as raw bytes with prompt/input redraw semantics (byte fidelity preserved).

- Attribution:
  - PTY bytes are not attributed to a specific agent/task by default (session-level stream).
  - Structured agent events MUST include stable attribution fields suitable for deterministic joins across the LLM/agent/workflow/router tracks.
    - Required (minimum):
      - `orchestration_session_id`
      - `run_id`
      - `agent_id`
    - Required when applicable:
      - `thread_id` (LLM/conversation grouping)
      - `role` (agent hub role: orchestrator vs executor)
      - `backend_id` when the emitting backend’s kind is known (v1 default: known) to avoid heuristic inference from `agent_id` alone.
    - Join keys (required when the event is tied to execution/trace):
      - `cmd_id` and/or `span_id`
    - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
    - Placement: attribution fields are top-level keys on the serialized event envelope.
      - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0008)
  - Review / decision hook (agent hub alignment):
    - Phase 8 additive alignment: structured agent events MUST carry `world_id` when the emitting backend executes inside a world boundary, so operators can verify whether concurrently-running in-world agents share (or intentionally do not share) the same filesystem/isolation boundary.
      - Source of truth: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0004) and `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003).
    - Phase 8 additive alignment: the structured-event envelope MAY carry an optional event-plane routing hint (`channel`) so future subscribe/filter behavior is expressible without PTY injection or attribution ambiguity.
      - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003).

### Structured agent event envelope (v1; Phase 8 additive clarifications)

Structured agent events are serialized as a top-level envelope with stable correlation fields for deterministic joins. This envelope is the surface emitted to the shell/UI and (after adding `session_id` and any trace-writer metadata) persisted to canonical trace.

Authoritative shape and field requirements live in:
- `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003, DR-0008, DR-0009)
- Phase 8 correlation vocabulary (canonical field names): `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

Envelope fields (top-level; no nesting required for joinability):
- `ts` (RFC3339 UTC timestamp)
- `kind` (enum; `registered|status|task_start|task_progress|task_end|pty_data|alert`)
- Correlation (required/conditional/optional):
  - `orchestration_session_id` (required)
  - `run_id` (required)
  - `agent_id` (required; actor/principal id: `human` for operator actions; for agent-driven structured events, this is the agent inventory id)
  - `backend_id` (conditional required when the emitting backend kind/name is known; v1 default: known; `<kind>:<name>`)
  - `world_id` (conditional required when the emitting backend executes inside a world boundary)
  - `thread_id` (optional)
  - `role` (optional)
  - `cmd_id` / `span_id` (optional; required when tied to execution/trace joins)
  - `channel` (optional; event-plane routing hint; not used for policy gating)
- `data` (JSON object; schema depends on `kind`)

`channel` constraints (non-negotiable):
- Meaning: a producer-declared event-plane routing hint (pub/sub-style “topic”) for subscribe/filter and for explainable suppression summaries; it MUST NOT be used for policy gating.
- MUST be producer-declared (not arbitrary user-provided freeform) and MUST NOT contain secrets.
- MUST be capped (implementation-defined cap) and MUST be safe to print and persist.

### World session reuse + restart attribution (Phase 8 additive; operator-verifiable)

World-scoped member agents share a world boundary by default (same `world_id` per `orchestration_session_id`) per Agent Hub core decisions. Operators MUST be able to verify:
- whether multiple agents shared the same world boundary (`world_id` on world-scoped events), and
- when/why a world was restarted (explicit structured alert events; never implied).

Authoritative sources:
- World reuse semantics: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0004)
- Drift handling + reason taxonomy: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0008)
- `world_restarted` alert schema: `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0010)
- `world_restart_required` alert schema (fail-closed drift posture): `docs/project_management/next/agent_hub_core/decision_register.md` (DR-0009)

Structured alert event: `world_restarted` (required on auto-restart)
- Envelope:
  - `kind: "alert"`
  - correlation: `orchestration_session_id`, `run_id`, `agent_id`, `role: "orchestrator"` (required)
- `data` (required fields; stable schema):
  - `code: "world_restarted"`
  - `reason: <one of DR-0008 taxonomy strings>`
  - `on_drift: "auto_restart"`
  - `previous_world_id`, `new_world_id`
  - `previous_world_generation`, `new_world_generation`
  - `message` (human-readable; safe to print/persist)

Fail-closed drift posture (`agents.hub.world_restart.on_drift=fail_closed`)
- The hub MUST NOT restart implicitly; it MUST fail closed and MUST emit a structured alert event with `data.code="world_restart_required"` (schema in DR-0009), using the same DR-0008 reason taxonomy.

### Config (buffer tuning)
- Files and locations (existing layering model):
  - Global config patch: `$SUBSTRATE_HOME/config.yaml`
  - Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
- Key:
  - `repl.max_pty_buffered_lines: <int>`
    - Meaning: maximum number of **structured event lines** buffered during PTY passthrough before dropping begins (does not affect PTY bytes).
    - Default: `2048`.
    - Precedence: workspace overrides global (consistent with the existing config layering model).
    - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0004)
    - Bounds: `min=0`, `max=16384`.
    - Invalid/out-of-range handling:
      - Invalid type/parse: hard error (exit code `2` at config/CLI boundary).
      - Out-of-range integer: clamp and emit a structured warning (no PTY injection).
      - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0007)
    - Scope: applies to any PTY passthrough session (explicit `:pty` and implicit PTY-required runs).
      - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0010)
    - Example:
      ```yaml
      repl:
        max_pty_buffered_lines: 2048
      ```

## Architecture Shape
- `crates/shell`:
  - Maintains two concurrent output paths:
    - PTY-bytes rendering path (binary-safe).
    - Structured-event rendering path (string/typed messages; existing external-printer style).
  - Ensures structured-event output is not injected into PTY passthrough.
- `crates/world-agent`:
  - Continues to treat `stdout` as a raw PTY byte stream.
- `crates/common` / `crates/trace`:
  - Structured agent events MUST be traceable with stable correlation identifiers (persisted to canonical `trace.jsonl`).
    - This ADR defines the minimum required attribution fields; trace/span schema alignment remains additive-only and is finalized in the “circle-back” pass.
  - Trace persistence (authoritative):
    - Each structured agent event MUST be persisted as its own JSONL record in canonical trace.
    - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0009)

## Sequencing / Dependencies
- Depends on ADR-0016’s PTY passthrough and out-of-band PTY output rules for the REPL.
- This ADR is intentionally minimal and can be executed incrementally:
  - First: lock output classification + rendering rules.
  - Later: expand agent hub orchestration and config.

## Security / Safety Posture
- Prevent terminal corruption and output spoofing risks by separating:
  - PTY byte streams from
  - structured host/agent events.
- Maintain fail-closed posture for world execution where required by policy:
  - This ADR does not introduce any implicit host execution path.

## Validation Plan (Authoritative)
- Tests (high-level):
  - During PTY passthrough, structured agent events are buffered and do not corrupt the TUI output.
  - While idle, out-of-band PTY bytes are rendered without corrupting the input buffer.
  - Structured events include stable attribution fields.
- Manual playbook:
  - Verify `:demo-agent` (or equivalent) can run concurrently with a PTY passthrough command without corrupting the terminal.

## Rollout / Backwards Compatibility
- Greenfield breaking is acceptable: behavior changes are allowed so long as the operator-facing contract is explicit and high-signal.
- This ADR does not deprecate any stable public API; it only refines interactive output behavior.

## Decision Summary
- Decisions live in:
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md`
