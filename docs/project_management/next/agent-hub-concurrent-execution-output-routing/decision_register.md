# Decision Register — Agent Hub Concurrent Execution Output Routing

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Output classes: PTY bytes vs structured agent events

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- Substrate needs a deterministic output contract for concurrent agent activity that preserves PTY correctness (TUIs) and avoids corrupting the interactive line editor.
- Repo grounding (already exists today; extend it, do not rebuild in parallel):
  - Structured out-of-band events are already handled during the REPL prompt wait loop (used by `:demo-agent`):
    - `crates/shell/src/repl/async_repl.rs` (prompt loop `tokio::select!` over `agent_rx.recv()` → `handle_agent_event` → `agent_printer.print(...)`)
    - `crates/shell/src/execution/agent_events.rs` (`schedule_demo_events()` publishes demo events)

**Option A — Two output classes with explicit rendering separation**
- **Pros:**
  - Prevents terminal corruption during TUIs / PTY passthrough.
  - Preserves PTY byte fidelity (binary-safe) by treating PTY as bytes, not strings.
  - Keeps attribution clean for structured events without injecting host messages into PTY streams.
- **Cons:**
  - Requires maintaining two output paths (byte printer vs structured printer) and explicit rules for interaction.
- **Cascading implications:**
  - PTY bytes and structured agent events must never share the same rendering channel during passthrough.
  - Structured events require stable attribution fields (see DR-0003).
- **Risks:**
  - Implementation complexity; needs tests to prevent regressions in interactive mode.
- **Unlocks:**
  - Safe concurrent operation for agent hub + world-first REPL.
- **Quick wins / low-hanging fruit:**
  - Reuse Reedline external printer + external byte printer APIs for safe output.

**Option B — Single string channel for all output**
- **Pros:**
  - Simplest mental model: one printer/stream.
- **Cons:**
  - Unsafe for PTY passthrough; cannot represent arbitrary PTY bytes reliably.
  - High risk of terminal corruption and user input buffer corruption.
- **Cascading implications:**
  - Forces lossy encoding choices and/or “best-effort” behavior (not acceptable for TUIs).
- **Risks:**
  - Breaks core UX correctness for interactive workloads.
- **Unlocks:**
  - None that preserve fidelity/correctness.
- **Quick wins / low-hanging fruit:**
  - None compatible with safe interactive behavior.

**Recommendation**
- **Selected:** Option A — Two output classes with explicit rendering separation.
- **Rationale (crisp):** PTY correctness and binary fidelity are non-negotiable; structured events must not corrupt TUIs or the line editor.

**Follow-up tasks (explicit)**
- Ensure REPL rendering uses a byte-safe PTY output path and a separate structured-event printer path.
- Add tests that assert structured events do not inject into PTY passthrough.

---

### DR-0002 — Structured events during PTY passthrough (buffer/drop vs backpressure)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- During PTY passthrough (TUIs), structured agent events must not be injected into the PTY byte stream, but we still need deterministic behavior for concurrent structured output.

**Option A — Bounded buffer + drop-with-summary (UI path)**
- **Pros:**
  - Avoids liveness coupling: structured output cannot stall execution.
  - Deterministic memory bound; prevents OOM during long passthrough sessions.
  - Preserves TUI correctness by avoiding injection into PTY bytes.
- **Cons:**
  - Some structured lines may not be displayed live during long passthrough.
- **Cascading implications:**
  - Overflow behavior must be explicit (single warning marker and/or dropped-count summary).
  - Rendering semantics must remain separate from persistence semantics (do not conflate “displayed” with “recorded”).
- **Risks:**
  - Operators may miss live events during a long TUI; mitigated by summary + durable sinks (trace/session logs in future tracks).
- **Unlocks:**
  - Safe concurrent agent activity during interactive PTY workloads.
- **Quick wins / low-hanging fruit:**
  - Implement buffer with a fixed default and add a later config knob (see DR-0004).

**Option B — Backpressure (block producers; never drop)**
- **Pros:**
  - No structured events are dropped.
- **Cons:**
  - Couples observability to execution; can stall/lock up agent execution when output is heavy.
  - Higher risk of deadlocks or priority inversion in async paths.
- **Cascading implications:**
  - Requires careful design to ensure PTY byte rendering never blocks on structured-event flow control.
- **Risks:**
  - Terminal/agent hub becomes unstable under load.
- **Unlocks:**
  - None required for v1 correctness.
- **Quick wins / low-hanging fruit:**
  - None without significant correctness risk.

**Recommendation**
- **Selected:** Option A — Bounded buffer + drop-with-summary (UI path).
- **Rationale (crisp):** Rendering must not become an availability dependency; bounded buffering preserves PTY correctness without risking stalls/deadlocks.

**Follow-up tasks (explicit)**
- Define and test deterministic overflow signaling for structured events during passthrough.

---

### DR-0006 — Overflow signaling for structured events during PTY passthrough (marker-only vs dropped-count summary)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- When the bounded structured-event buffer overflows during PTY passthrough, Substrate must provide an explicit, programmatically extractable signal describing what was suppressed, without injecting messages into the PTY byte stream.

**Option A — Single marker line only**
- **Pros:**
  - Minimal implementation.
  - Provides a human-visible “something was dropped” hint after passthrough ends.
- **Cons:**
  - Not programmatically useful: consumers cannot reliably tell how much was suppressed.
  - Harder to build future session logs / UIs that summarize suppressed activity.
- **Cascading implications:**
  - Encourages downstream heuristics (“guess from time window”) which are fragile.
- **Risks:**
  - Operators under-estimate suppressed output volume and misdiagnose missing logs.
- **Unlocks:**
  - None; purely a stopgap.
- **Quick wins / low-hanging fruit:**
  - None that preserve structured extraction.

**Option B — Emit an explicit dropped-count summary (structured)**
- **Pros:**
  - Programmatically extractable: downstream consumers can render accurate summaries and link to durable sinks.
  - Keeps the UI contract honest: “N lines suppressed” is explicit and deterministic.
- **Cons:**
  - Slightly more plumbing: must track a dropped counter and emit one summary record.
- **Cascading implications:**
  - The summary MUST be emitted via the structured-event path (not PTY), after passthrough ends.
  - Summary payload MUST include:
    - `dropped_structured_event_lines: <int>`
    - `max_pty_buffered_lines: <int>` (effective configured cap)
- **Risks:**
  - Minimal; additive metadata.
- **Unlocks:**
  - Future session logs and block-based UIs can collapse high-volume output into a stable, user-friendly summary without losing auditability.
- **Quick wins / low-hanging fruit:**
  - Implement as a single shell warning record emitted once per passthrough session when drops occurred (consistent with existing `substrate: warning:` emission patterns).

**Recommendation**
- **Selected:** Option B — Emit an explicit dropped-count summary (structured).
- **Rationale (crisp):** Downstream tools (session logs, block UIs, router-era consumers) need deterministic, machine-readable suppression metrics; a marker-only string is insufficient.

**Follow-up tasks (explicit)**
- Track `dropped_structured_event_lines` during PTY passthrough once the buffer cap is hit.
- After passthrough ends (before returning to the prompt), emit exactly one structured summary warning containing:
  - `dropped_structured_event_lines: <int>`
  - `max_pty_buffered_lines: <int>` (effective configured cap)
  - Emission mechanism (non-negotiable):
    - Write a canonical trace record to `trace.jsonl` with:
      - `component: "shell"`
      - `event_type: "warning"`
      - stable correlation fields when available (e.g., `session_id`, and `cmd_id`/`span_id` when the passthrough is tied to a command)
      - the structured payload above
    - Print a human-readable warning line via the normal warning channel (e.g., `substrate: warning: …`), without injecting into PTY bytes.

---

### DR-0003 — Attribution requirements for structured agent events (minimal vs rich required)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- Upcoming LLM/agent/workflow/router ADRs require stable correlation fields to support trace-driven routing, session logging, and multi-agent attribution without heuristics.

**Option A — Minimal required attribution (extend later)**
- **Pros:**
  - Lower initial coupling and smaller immediate change surface.
- **Cons:**
  - High likelihood of later breaking/additive churn across multiple emitters once router/workflow/LLM specs land.
  - Forces consumers to rely on heuristics or partial joins early.
- **Cascading implications:**
  - Session logs and router rules will have weaker joinability guarantees until later upgrades.
- **Risks:**
  - Rework risk; drift across components.
- **Unlocks:**
  - Faster prototype iteration.
- **Quick wins / low-hanging fruit:**
  - None that preserve long-term contract stability.

**Option B — Rich required attribution (correlation-first)**
- **Pros:**
  - Enables deterministic joins across LLM gateway, agent hub, workflow engine, and router daemon.
  - Reduces future churn by locking correlation surfaces early.
- **Cons:**
  - Requires more plumbing in early implementation to ensure fields are always available.
- **Cascading implications:**
  - Structured events MUST carry stable correlation identifiers suitable for routing and audit.
- **Risks:**
  - Slightly higher upfront coordination cost.
- **Unlocks:**
  - Trace-driven routing/session logging can be built without heuristic joins.
- **Quick wins / low-hanging fruit:**
  - Define field requirements now and enforce via tests/validation in subsequent planning packs.

**Recommendation**
- **Selected:** Option B — Rich required attribution (correlation-first).
- **Rationale (crisp):** Later ADRs (LLM gateway/engines, agent hub, workflow engine, router daemon) require stable joins; locking attribution now avoids drift and heuristic routing.

**Follow-up tasks (explicit)**
- Require structured events to include (at minimum) correlation fields such as:
  - `orchestration_session_id`
  - `run_id`
  - `thread_id` (when applicable)
  - `agent_id` and `role` (when applicable)
  - join keys like `cmd_id` / `span_id` when tied to execution/trace
  - Phase 8 additive clarifications (cross-cutting; no reshapes):
    - `agent_id` is the actor/principal identifier:
      - `human` for direct operator actions.
      - for agent-driven structured events, this is the agent inventory id (so attribution is audit-friendly and stable).
    - When a specific backend is involved and its kind/name is known, the event MUST include `backend_id` in `<kind>:<name>` form to make allowlist/routing joins explicit and avoid heuristic inference from `agent_id` alone.
    - The trace vocabulary and required/non-required matrix for correlation fields is finalized additively in ADR-0028 Phase 8 circle-back.
  - Initial serialized shape (v1; additive-only extensions allowed in the circle-back pass):
    - Envelope fields (top-level; not nested in `data`):
      - `ts` (RFC3339 UTC timestamp)
      - `agent_id` (string; actor/principal id)
      - `backend_id` (string; non-required; `<kind>:<name>`)
      - `kind` (enum; `registered|status|task_start|task_progress|task_end|pty_data|alert`)
      - `orchestration_session_id` (string; required)
      - `run_id` (string; required)
      - `thread_id` (string; non-required)
      - `role` (string; non-required taxonomy label; v1 reserved values include `orchestrator|member`)
      - `world_id` (string; non-required; required when the emitting backend executes inside a world boundary)
      - `cmd_id` (string; non-required)
      - `span_id` (string; non-required)
      - `channel` (string; non-required; event-plane routing hint for subscribe/filter behavior; not used for policy gating)
        - Constraints (non-negotiable):
          - Meaning: a producer-declared “topic” for subscribe/filter; must never be required for joins, and must never affect policy gating decisions.
          - MUST be producer-declared (not arbitrary user-provided freeform).
          - MUST be capped (recommendation: <= 64 bytes) and safe to print/persist.
          - MUST NOT contain secrets; if a producer attempts to set a channel containing obvious secret material, the value MUST be dropped deterministically and no warning record is emitted.
    - Payload:
      - `data` is a JSON object whose schema depends on `kind` (e.g., `message`, or stream `chunk` + `stream`).

---

### DR-0004 — Configurability of structured-event buffer cap during PTY passthrough

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- The PTY passthrough structured-event buffer must be bounded by default, but operators may need to tune it per workspace/global context (CI vs local dev, noisy agents vs quiet agents).

**Option A — Fixed constant only**
- **Pros:**
  - No new config surface; simplest.
  - Deterministic behavior across environments.
- **Cons:**
  - Cannot tune for very noisy agents or constrained environments.
- **Cascading implications:**
  - Future consumers may add ad-hoc env vars; risk of inconsistent knobs.
- **Risks:**
  - Misfit defaults for some workloads.
- **Unlocks:**
  - None beyond simplicity.
- **Quick wins / low-hanging fruit:**
  - Ship quickly with a constant.

**Option B — Config knob in existing global + workspace YAML config**
- **Pros:**
  - Tunable without changing code; aligns with existing config layering model.
  - Workspace-specific overrides allow “project policy” for noisy vs quiet workflows.
- **Cons:**
  - Adds a new config key to the strict schema.
- **Cascading implications:**
  - Key must be defined in config schema and documented; precedence must match the existing model.
- **Risks:**
  - Incorrectly sized values can increase memory usage; must validate and clamp.
- **Unlocks:**
  - Safer ops tuning across different environments (local dev vs CI vs demos).
- **Quick wins / low-hanging fruit:**
  - Default remains the current constant; if the key is absent, the default applies.

**Recommendation**
- **Selected:** Option B — Config knob in existing global + workspace YAML config.
- **Rationale (crisp):** Keep a safe default but allow deterministic tuning through the standard config surfaces without inventing new env-var one-offs.

**Follow-up tasks (explicit)**
- Add config key:
  - Global: `$SUBSTRATE_HOME/config.yaml`
  - Workspace: `<workspace_root>/.substrate/workspace.yaml`
  - Key path: `repl.max_pty_buffered_lines`
  - Default: `2048`
  - Validation: integer, bounded (`min=0`, `max=16384`), with deterministic invalid handling (see DR-0007).

---

### DR-0005 — Out-of-band PTY bytes while the line editor is active (safe encoding vs raw bytes + redraw)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- While the line editor is active (`Idle`), PTY output may arrive out-of-band. Substrate must render this output without corrupting the current input buffer.
- Repo grounding (already exists today; extend it, do not rebuild in parallel):
  - PTY bytes already use a byte-safe printer when the prompt is active:
    - `crates/shell/src/repl/async_repl.rs` (`make_world_stdout_callback(...)` uses `ExternalBytePrinter` when `prompt_active` is true)

**Option A — Safe render (escape/encode output)**
- **Pros:**
  - Minimizes risk of terminal/prompt corruption by treating out-of-band bytes as data.
  - Simpler to reason about for arbitrary input (non-UTF8, control sequences).
- **Cons:**
  - Loses fidelity for real PTY output (ANSI/control sequences and TUI-like output will not render correctly).
- **Cascading implications:**
  - Requires a “safe encoding” definition and consistent rendering across platforms/terminals.
- **Risks:**
  - Operators see degraded output and may misinterpret interactive program state.
- **Unlocks:**
  - A conservative fallback mode for extreme environments.
- **Quick wins / low-hanging fruit:**
  - None without accepting fidelity loss.

**Option B — Raw bytes + prompt/input redraw (fidelity render)**
- **Pros:**
  - Preserves PTY byte fidelity (binary-safe) and terminal semantics.
  - Aligns with the core output contract: PTY output is bytes.
- **Cons:**
  - Requires correct “print bytes then restore prompt/input” mechanics.
- **Cascading implications:**
  - REPL must use a byte-safe printer that cooperates with the line editor (prompt redraw).
- **Risks:**
  - Incorrect redraw integration can still corrupt the input buffer (must be covered by tests).
- **Unlocks:**
  - Correct and familiar interactive UX for long-lived PTY sessions with concurrent output.
- **Quick wins / low-hanging fruit:**
  - Reuse Reedline’s external byte printer APIs to render bytes while preserving the input buffer.

**Recommendation**
- **Selected:** Option B — Raw bytes + prompt/input redraw (fidelity render).
- **Rationale (crisp):** PTY fidelity is a core contract; rendering out-of-band bytes must preserve terminal semantics while keeping the line editor stable.

**Follow-up tasks (explicit)**
- Ensure out-of-band PTY bytes are routed through a byte-safe printer that restores prompt/input (no “string channel” fallback).

---

### DR-0007 — Invalid/edge config handling for the PTY structured-event buffer cap (hard error vs clamp + warning)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- The buffer cap value will vary across machines, workloads, and environments (CI vs local dev). We must keep the system safe-by-default while preserving deterministic behavior and avoiding “footgun” values that cause unbounded memory growth.

**Option A — Hard error on any invalid or out-of-range value**
- **Pros:**
  - Strict and predictable: operators must provide a valid value or execution fails fast.
  - No silent behavior changes.
- **Cons:**
  - Operationally brittle: “too large” values become workflow-breaking.
  - Encourages ad-hoc overrides or local patches to recover.
- **Cascading implications:**
  - All call sites that load config must propagate an error and surface an actionable message.
- **Risks:**
  - Non-critical tuning knob becomes a frequent failure mode in mixed environments.
- **Unlocks:**
  - None beyond strictness.
- **Quick wins / low-hanging fruit:**
  - Use default serde/YAML validation and add explicit bounds checks.

**Option B — Hard error on invalid type, clamp out-of-range values with a structured warning**
- **Pros:**
  - Safe-by-default: the runtime never uses an unbounded value.
  - Operationally resilient: users can set “too big” and still run; behavior is deterministic via clamping.
  - Programmatically extractable: warning can be consumed by future session logs/UIs.
- **Cons:**
  - Requires a consistent warning schema and tests to ensure warnings fire only when needed.
- **Cascading implications:**
  - Value bounds must be defined and stable.
  - Warning emission must not inject into PTY bytes and must be traceable.
- **Risks:**
  - Users may not notice clamping without looking at warnings; mitigated by structured warning + trace persistence.
- **Unlocks:**
  - Supports heterogeneous hardware/workloads without making the knob a reliability risk.
- **Quick wins / low-hanging fruit:**
  - Emit a single shell warning record when clamping occurs (include raw value and clamped value).

**Recommendation**
- **Selected:** Option B — Hard error on invalid type, clamp out-of-range values with a structured warning.
- **Rationale (crisp):** The knob is tuning, not correctness; clamping preserves safety and determinism without turning tuning into workflow failure.

**Follow-up tasks (explicit)**
- Bounds (non-negotiable): `min=0`, `max=16384`.
- Invalid handling:
  - Invalid type / parse failure: hard error (exit code `2` at CLI/config boundary).
  - Out-of-range integer: clamp to bounds and emit one structured warning event containing:
    - `raw_value`
    - `effective_value`
    - `min`
    - `max`
    - Emission mechanism (non-negotiable):
      - Write a canonical trace record to `trace.jsonl` with:
        - `component: "shell"`
        - `event_type: "warning"`
        - `session_id`
        - structured payload above
      - Print a human-readable warning line via the normal warning channel (e.g., `substrate: warning: …`), without injecting into PTY bytes.
  - Config shape (YAML; both global `$SUBSTRATE_HOME/config.yaml` and workspace `.substrate/workspace.yaml`):
    - Example:
      ```yaml
      repl:
        max_pty_buffered_lines: 2048
      ```

---

### DR-0008 — Placement of attribution fields in the structured agent event envelope (top-level vs nested attribution object)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- Downstream consumers (router daemon, session logs, block UIs) need deterministic, low-friction extraction of attribution fields. Field placement must avoid drift and minimize ad-hoc JSON parsing.

**Option A — Top-level envelope fields (schema-first)**
- **Pros:**
  - Easy to query (`jq`, SQL) without nested-path fragility.
  - Schema enforcement is simpler (required fields are clearly required).
  - Encourages consistency across all event emitters.
- **Cons:**
  - Adds fields to the envelope type; requires coordination across emitters.
- **Cascading implications:**
  - Attribution fields defined in DR-0003 are top-level keys on every serialized agent event.
- **Risks:**
  - Slightly higher upfront coordination.
- **Unlocks:**
  - Deterministic routing/session summarization without heuristic parsing.
- **Quick wins / low-hanging fruit:**
  - Start with required subset; extend additively in the circle-back pass.

**Option B — Nested `data.attribution` object**
- **Pros:**
  - Flexible payload evolution inside `data`.
- **Cons:**
  - Easier to drift; harder to enforce consistently across emitters.
  - More cumbersome queries and tooling (nested-path handling everywhere).
- **Cascading implications:**
  - Requires stable nested schema conventions and versioning to prevent drift.
- **Risks:**
  - Consumers end up with brittle logic and partial joins.
- **Unlocks:**
  - Slight envelope stability at the cost of higher consumer complexity.
- **Quick wins / low-hanging fruit:**
  - None that preserve determinism.

**Recommendation**
- **Selected:** Option A — Top-level envelope fields (schema-first).
- **Rationale (crisp):** Programmatic extraction and schema enforcement dominate; nested attribution is too easy to drift.

**Follow-up tasks (explicit)**
- Ensure all required attribution fields are top-level keys on serialized agent events (per DR-0003).

---

### DR-0009 — Trace persistence for structured agent events (persist all vs UI-only)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- Substrate is both an interactive REPL and a backend engine for other applications. Structured agent events must be available programmatically and durably; the canonical local source of truth is `trace.jsonl`.

**Option A — Persist every structured agent event to canonical trace**
- **Pros:**
  - Durable, programmatically extractable event stream for session logs and UIs.
  - Enables cross-component joins with LLM gateway, workflow engine, and router daemon.
- **Cons:**
  - Increases trace volume; requires explicit caps where needed (handled by DR-0002/DR-0006 for PTY passthrough display).
- **Cascading implications:**
  - For each structured agent event, append a JSONL record to `trace.jsonl`.
  - Records must include stable attribution fields (DR-0003/DR-0008).
- **Risks:**
  - Trace growth under noisy agents; mitigated by existing rotation and future session-log summarization.
- **Unlocks:**
  - Block UIs and router-era consumers can consume the event stream without depending on terminal rendering.
- **Quick wins / low-hanging fruit:**
  - Start with a single `event_type` (e.g., `agent_event`) and include `kind` + envelope fields; extend additively.

**Option B — UI-only events (no canonical trace persistence)**
- **Pros:**
  - Lower trace volume.
- **Cons:**
  - Breaks the “Substrate as backend” goal; events become un-auditable and non-extractable.
  - Forces future session logs/UIs to depend on terminal output capture (not acceptable).
- **Cascading implications:**
  - Requires a second persistence mechanism later (schema churn risk).
- **Risks:**
  - High; consumers build on unstable, non-durable surfaces.
- **Unlocks:**
  - None aligned with long-term direction.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Persist every structured agent event to canonical trace.
- **Rationale (crisp):** Trace is the durable, programmatic substrate; UI rendering must not be the only place events exist.

**Follow-up tasks (explicit)**
- Canonical trace record shape (v1; additive-only extensions allowed):
  - `component: "agent-hub"`
  - `event_type: "agent_event"`
  - `session_id` (shell trace session id)
  - envelope fields (`agent_id`, `kind`, `data`, and correlation fields) at the record top level (no nested envelope object)
  - required correlation fields (`orchestration_session_id`, `run_id`) present on every record (DR-0003/DR-0008)

---

### DR-0010 — Scope of `repl.max_pty_buffered_lines` (only `:pty` vs all PTY passthrough)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

**Problem / Context**
- PTY passthrough can be entered explicitly (e.g., `:pty`) or implicitly (commands that require a PTY). Buffering/suppression semantics must be consistent regardless of how passthrough starts.

**Option A — Apply only to explicit `:pty` passthrough**
- **Pros:**
  - Smaller initial touch surface.
- **Cons:**
  - Inconsistent behavior: implicit PTY passthrough would behave differently from explicit `:pty`.
  - Harder for operators and UIs to reason about suppression.
- **Cascading implications:**
  - Requires separate policies and docs for multiple passthrough entry points.
- **Risks:**
  - Drift and confusion.
- **Unlocks:**
  - None.
- **Quick wins / low-hanging fruit:**
  - None that preserve consistency.

**Option B — Apply to any PTY passthrough session**
- **Pros:**
  - Consistent, predictable behavior across the REPL.
  - Simplifies documentation and downstream consumers.
- **Cons:**
  - Requires ensuring all passthrough paths share the same buffering/suppression implementation.
- **Cascading implications:**
  - The same bounded buffer + drop-count summary semantics apply whenever PTY passthrough is active.
- **Risks:**
  - Minimal; mostly implementation plumbing.
- **Unlocks:**
  - Coherent user and UI expectations.
- **Quick wins / low-hanging fruit:**
  - Reuse the existing `:pty` handling as the reference implementation.

**Recommendation**
- **Selected:** Option B — Apply to any PTY passthrough session.
- **Rationale (crisp):** Consistency across explicit and implicit PTY passthrough is required for predictable UX and programmatic consumption.

**Follow-up tasks (explicit)**
- Ensure all PTY passthrough entry points use the same buffering/suppression logic and honor `repl.max_pty_buffered_lines`.

---

### DR-0011 — Agent event canonical trace record shape (flattened vs nested payload)

**Decision owner(s):** Shell + Trace maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`, `docs/TRACE.md`

**Problem / Context**
- Phase 8 correlation vocabulary requires join keys to be present as top-level fields on trace records (operator queryability via `jq`, router/workflow joins, and auditability).

**Option A — Flatten envelope fields into the trace record (recommended)**
- **Pros:**
  - Join keys are top-level fields on the `agent_event` record (Phase 8-compatible).
  - Queries do not require nested-path handling.
  - Aligns with `docs/TRACE.md` “all records carry correlation fields” guidance.
- **Cons:**
  - Adds more top-level keys to the record.
- **Cascading implications:**
  - `telemetry-spec.md` must define `agent_event` as a flattened record (no `payload` wrapper).
  - Envelope schema remains authoritative for the flattened fields.
- **Risks:**
  - Low; additive-only evolution is preserved.
- **Unlocks:**
  - Router/workflow consumers can subscribe and join without heuristics.
- **Quick wins / low-hanging fruit:**
  - Emit one record family (`event_type="agent_event"`) with stable `component="agent-hub"`.

**Option B — Nest the envelope under a `payload` object**
- **Pros:**
  - Reduces top-level key count.
- **Cons:**
  - Join keys become nested (`payload.orchestration_session_id`), which violates Phase 8 ergonomics and complicates routing queries.
  - Increased drift risk (two levels of schema ownership).
- **Cascading implications:**
  - Consumers must implement nested extraction in all tooling.
- **Risks:**
  - Medium; downstream systems drift into heuristic joins.
- **Unlocks:**
  - None aligned with Phase 8 correlation goals.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — Flatten envelope fields into the trace record.
- **Rationale (crisp):** Phase 8 joinability requires top-level correlation fields; nested payloads create needless tooling friction and drift risk.

**Follow-up tasks (explicit)**
- In `telemetry-spec.md`, define `agent_event` records as flattened envelope fields + trace-required keys (`ts`, `event_type`, `session_id`, `component`).

---

### DR-0012 — Handling unsafe `channel` values (drop silently vs emit warning record)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md`

**Problem / Context**
- The `channel` field is persisted to canonical trace and may be printed. It must be secrets-safe.
- If a producer attempts to set an unsafe channel value, the system must respond deterministically without leaking the unsafe value.

**Option A — Drop unsafe values silently (recommended)**
- **Pros:**
  - No risk of re-emitting secret material via warnings.
  - No new warning code or telemetry family required.
  - Keeps the v1 schema surface minimal.
- **Cons:**
  - Producers do not receive an explicit warning that a channel was dropped.
- **Cascading implications:**
  - Envelope schema must specify that unsafe channel values are dropped and never emitted in warnings/logs.
- **Risks:**
  - Low; channel is a routing hint, not a correctness join key.
- **Unlocks:**
  - Keeps trace safe-by-default.
- **Quick wins / low-hanging fruit:**
  - Implement drop in a single validation helper used by all producers.

**Option B — Drop unsafe values and emit a warning record**
- **Pros:**
  - Producers/operators get explicit signal that channel validation occurred.
- **Cons:**
  - Adds a new warning record contract and test surface.
  - Even sanitized warnings risk accidental leakage if any part of the value is echoed.
- **Cascading implications:**
  - Requires a new warning code in `telemetry-spec.md` and corresponding docs/tests.
- **Risks:**
  - Medium; warning implementation mistakes can leak secrets.
- **Unlocks:**
  - Slightly better debuggability at the cost of broader surface.
- **Quick wins / low-hanging fruit:**
  - None compatible with a minimal v1.

**Recommendation**
- **Selected:** Option A — Drop unsafe values silently.
- **Rationale (crisp):** Safety dominates; channel is not required for joins and must not expand the warning surface in v1.

**Follow-up tasks (explicit)**
- Enforce: unsafe channel values are dropped, and the dropped value is never emitted in any warning/log/trace field.

---

### DR-0013 — Suppression warning payload detail (total-only vs per-channel breakdown)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/agent-hub-concurrent-execution-output-routing/telemetry-spec.md`

**Problem / Context**
- When structured output is suppressed during PTY passthrough, the warning record must be deterministic and bounded.

**Option A — Total-only suppression record (recommended)**
- **Pros:**
  - Minimal, bounded payload that is easy to validate and query.
  - Avoids expanding the surface area that includes `channel` values.
  - Satisfies operator needs for deterministic “something was suppressed” with a concrete magnitude.
- **Cons:**
  - Less explainable when multiple channels are active.
- **Cascading implications:**
  - `telemetry-spec.md` defines only `dropped_structured_event_lines` and `max_pty_buffered_lines` (plus correlation fields when available).
- **Risks:**
  - Low; the system remains auditable because all events are persisted as `agent_event` records.
- **Unlocks:**
  - Keeps OR1 implementation and tests focused on correctness and safety.
- **Quick wins / low-hanging fruit:**
  - Implement dropped counter and emit one record at passthrough end.

**Option B — Include per-channel breakdown (bounded buckets)**
- **Pros:**
  - More explainable summaries for human and UI consumers.
- **Cons:**
  - Additional complexity and a larger payload surface.
  - Increases the risk of accidentally persisting or printing sensitive routing hints.
- **Cascading implications:**
  - Requires explicit bucketing and cap rules and additional tests.
- **Risks:**
  - Medium; channel values become more prominent in warning payloads.
- **Unlocks:**
  - Richer summaries without scanning `agent_event` records.
- **Quick wins / low-hanging fruit:**
  - None in a minimal v1.

**Recommendation**
- **Selected:** Option A — Total-only suppression record.
- **Rationale (crisp):** The durable `agent_event` stream remains the source of truth; suppression summaries remain minimal and secrets-safe.

**Follow-up tasks (explicit)**
- In `telemetry-spec.md`, omit any per-channel breakdown fields from the v1 suppression warning record schema.
