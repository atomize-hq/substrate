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

ADR_BODY_SHA256: d20d6f66be403daeafeea8a53e33dc7bf0b25451e1480827952366e2dd4e5e6a

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
    - Join keys (required when the event is tied to execution/trace):
      - `cmd_id` and/or `span_id`
    - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0003)
    - Placement: attribution fields are top-level keys on the serialized event envelope.
      - Source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md` (DR-0008)

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
