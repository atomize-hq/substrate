# Decision Register — forge

Template standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

### DR-0001 — Loop shape + step semantics

**Decision owner(s):** Shell / LLM maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`, `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- Forge is intended to be a reusable composite node inside the workflow engine, not a general workflow runtime.
- The loop shape must be deterministic and testable, with a single definition of PASS/FAIL and retry semantics.

**Option A — 4-step loop: execute → critique → refine → review (retry full loop)**
- **Pros:** simple and deterministic; easy to test; minimal surface area for v1.
- **Cons:** lacks “leadership” steps (monitor/reflect) from the Python Forge baseline.
- **Cascading implications:**
  - Each attempt runs the full 4 steps in order.
  - PASS/FAIL is decided by the `review` step only.
  - On FAIL, Forge retries the full 4-step attempt until `max_attempts` is exhausted.
- **Risks:** parity gaps vs Python baseline; mitigated by explicit deferrals.
- **Unlocks:** a stable composite node that can be embedded in DAG workflows without adding graph semantics.
- **Quick wins / low-hanging fruit:** enables v1 quickly while keeping room for later extensions.

**Option B — Extended loop (monitor/reflect + dynamic leadership)**
- **Pros:** closer parity with Python Forge; potentially better outputs.
- **Cons:** larger scope; more ambiguous semantics; couples to provider/model routing and leadership policy earlier than needed.
- **Cascading implications:** requires additional step types and stop-condition definitions; increases event/trace requirements.
- **Risks:** scope creep and non-determinism.
- **Unlocks:** richer iterations and adaptive behavior.
- **Quick wins / low-hanging fruit:** none; complexity is front-loaded.

**Recommendation**
- **Selected:** Option A — 4-step loop with retry-full-attempt semantics
- **Rationale (crisp):** delivers a deterministic, embed-friendly composite node in v1 while deferring higher-complexity leadership behavior.

**Follow-up tasks (explicit)**
- Implement attempt state machine with exactly these steps: `execute`, `critique`, `refine`, `review`.
- Define review stop condition parsing (PASS/FAIL) and enforce it strictly (invalid review output is a FAIL).

### DR-0002 — Budget enforcement + fail-closed rules

**Decision owner(s):** Shell / LLM maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`

**Problem / Context**
- Forge must enforce operator-specified budgets deterministically and must not silently exceed them.

**Option A — Budgets enforced only by the workflow engine (outer node timeout only)**
- **Pros:** simplest Forge core; budgets handled in one place (workflow runtime).
- **Cons:** cannot enforce attempt/token budgets inside the loop; timeouts are coarse; makes retry semantics hard to bound.
- **Cascading implications:** Forge becomes a “black box” node from a budget perspective.
- **Risks:** runaway LLM calls inside a node; poor explainability.
- **Unlocks:** minimal initial implementation.
- **Quick wins / low-hanging fruit:** quick.

**Option B — Forge enforces loop budgets (attempts + runtime + tokens)**
- **Pros:** deterministic enforcement; clear failure modes; attempt budget is native to Forge semantics.
- **Cons:** requires gateway usage metadata for token accounting.
- **Cascading implications:**
  - `max_attempts` is required and enforced by Forge core.
  - `max_runtime_ms` is optional and enforced by Forge core (wall-clock).
  - `max_tokens` is optional; if set, Forge must fail closed when token usage cannot be accounted.
- **Risks:** token usage metadata varies by provider; mitigated by normalizing in the gateway layer (ADR-0023/ADR-0024).
- **Unlocks:** safe defaults for repeated refinement loops.
- **Quick wins / low-hanging fruit:** enables robust tests around budgets and stop conditions.

**Recommendation**
- **Selected:** Option B — Forge enforces loop budgets
- **Rationale (crisp):** budgets are core to safe looping behavior and must be enforced at the loop boundary, not only at the DAG boundary.

**Follow-up tasks (explicit)**
- Implement a budget tracker in `forge-core` (attempt counter + runtime + tokens when available).
- Ensure “missing token usage metadata when `max_tokens` is configured” fails closed with an actionable error.

### DR-0003 — Output extraction contract (`FINAL:` marker) vs JSON-only

**Decision owner(s):** Shell / LLM maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`, `docs/project_management/future/json-mode/json_mode_plan.md`

**Problem / Context**
- Forge must extract deterministic step outputs for traceability and to support downstream node wiring.
- The repo has a separate “JSON mode” planning track; Forge should not force that decision early.

**Option A — `FINAL:` marker extraction + `<think>` stripping (strict)**
- **Pros:** deterministic; works with non-JSON model outputs; does not require global JSON mode adoption.
- **Cons:** relies on prompt discipline; requires strict validation and retry behavior for missing markers.
- **Cascading implications:** all role templates must require `FINAL:`; review must encode PASS/FAIL deterministically.
- **Risks:** models omit `FINAL:`; mitigated by retries and clear errors.
- **Unlocks:** stable extraction without blocking on JSON mode.
- **Quick wins / low-hanging fruit:** aligns with existing patterns in the repo.

**Option B — JSON-only structured outputs for every role**
- **Pros:** easiest downstream parsing; more resilient to whitespace/formatting.
- **Cons:** couples Forge to the repo-wide JSON mode strategy; increases prompt strictness and failure rates.
- **Cascading implications:** requires defining a stable JSON schema and enforcement across all providers/models.
- **Risks:** brittle across models; user friction.
- **Unlocks:** richer typed contracts.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — `FINAL:` marker extraction + `<think>` stripping
- **Rationale (crisp):** preserves determinism without forcing the repo-wide JSON mode decision.

**Follow-up tasks (explicit)**
- Implement extraction utility used by all steps (strip `<think>`, extract last `FINAL:` block).
- Enforce review PASS/FAIL contract:
  - first extracted line must be `FINAL: PASS` or `FINAL: FAIL` (exact; case-sensitive).
  - on PASS, the final Forge `result` is the extracted content after the PASS line.

### DR-0004 — Artifact persistence posture (trace-only vs file store)

**Decision owner(s):** Shell / Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`

**Problem / Context**
- Forge produces intermediate artifacts (role outputs, step summaries). We must decide whether v1 writes these to disk or relies on trace.

**Option A — Trace-only artifacts (no run store)**
- **Pros:** minimal new persistence surfaces; consistent with Substrate’s trace-first posture; less security risk (fewer files with potentially sensitive content).
- **Cons:** harder to browse without tooling; large outputs may require truncation/caps.
- **Cascading implications:** trace caps/redaction rules must be applied consistently (align to trace foundations).
- **Risks:** operators want files; mitigated by allowing workflow CLI to emit a single summary JSON (`--output-json`).
- **Unlocks:** ship v1 without durable storage design.
- **Quick wins / low-hanging fruit:** straightforward.

**Option B — File-backed artifact store under `SUBSTRATE_HOME`**
- **Pros:** easy manual inspection; structured filesystem layout.
- **Cons:** introduces a durable store design problem (rotation, GC, permissions, redaction); increases attack surface.
- **Cascading implications:** needs a full contract (paths, schema versions, lifecycle).
- **Risks:** secret leakage to disk; GC bugs.
- **Unlocks:** richer “run browser” experiences.
- **Quick wins / low-hanging fruit:** none; requires careful design.

**Recommendation**
- **Selected:** Option A — Trace-only artifacts (no run store) in v1
- **Rationale (crisp):** keeps v1 safe and bounded; durable run storage can land later as an explicit workflow/store track.

**Follow-up tasks (explicit)**
- Ensure Forge step outputs are recorded in spans/events with redaction/caps.
- Add a workflow-run summary JSON output path option in the workflow CLI (owned by workflow-engine).

### DR-0005 — Event/streaming surface (workflow-runtime sink vs direct emitter)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`, `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

**Problem / Context**
- Forge needs to emit progress for UX and for trace correlation, but must not bypass the workflow runtime’s streaming and attribution model.

**Option A — Forge emits directly to the router/event bus**
- **Pros:** fewer layers; direct integration with trigger mechanisms.
- **Cons:** bypasses workflow-runtime attribution; creates two competing streaming/event planes.
- **Cascading implications:** Forge must know about router queues and policy evaluation boundaries.
- **Risks:** security boundary mistakes; output attribution drift vs ADR-0017.
- **Unlocks:** direct triggers on sub-step events.
- **Quick wins / low-hanging fruit:** none; increases coupling.

**Option B — Forge emits to workflow-runtime via `ForgeEventSink`**
- **Pros:** preserves a single streaming/event plane; workflow runtime remains the integrator for UX and trace linkage.
- **Cons:** adds an adapter layer.
- **Cascading implications:** `forge-substrate` implements `ForgeEventSink` and maps events to spans/stream frames.
- **Risks:** none material.
- **Unlocks:** Forge remains a reusable library without knowing about transport/policy.
- **Quick wins / low-hanging fruit:** aligns with the workflow-engine node executor model.

**Recommendation**
- **Selected:** Option B — Emit via workflow-runtime sink
- **Rationale (crisp):** keeps the architecture layered and prevents Forge from bypassing the workflow runtime’s attribution and policy integration points.

**Follow-up tasks (explicit)**
- Define Forge event envelope types in `forge-types`.
- Implement `forge-substrate` sink that:
  - writes spans/events using `substrate-trace`,
  - forwards progress updates through workflow runtime streaming.
