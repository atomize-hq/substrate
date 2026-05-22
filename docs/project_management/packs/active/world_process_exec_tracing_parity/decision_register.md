# Decision Register — world_process_exec_tracing_parity

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:

- This decision register supports `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — In-world subprocess tracing mechanism (ptrace vs in-world shims)

**Decision owner(s):** Shell + World-Agent + World runtime  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- We need subprocess-level exec/exit telemetry for commands executed inside the world boundary, including parent/child relationships, without weakening the security posture or requiring invasive changes to the world filesystem.

**Option A — ptrace-based process tree capture (Linux backend)**

- **Pros:**
  - Captures parent/child relationships deterministically (we own the root process launch).
  - Avoids mutating the world filesystem or PATH to deploy shims.
  - Produces a fully structured event stream (no fragile log parsing).
- **Cons:**
  - Requires Linux-specific implementation and careful hardening to avoid performance cliffs.
  - Some environments may restrict ptrace (requires degrade behavior + explicit diagnostics).
- **Cascading implications:**
  - Requires Linux-only capture implementation in the world backend and a stable transport contract from world-service to host.
  - Requires explicit caps/truncation and diagnostics to prevent large dependency graphs from exploding response sizes.
- **Risks:**
  - Overhead for process-heavy workloads if not carefully capped.
  - Compatibility variance across kernels/container environments (must be treated as “best-effort observability”).
- **Unlocks:**
  - Enables future streaming of process events (incremental) without redesigning the capture mechanism.
  - Enables reliable audit/debug tooling without stdout/stderr inference.
- **Quick wins / low-hanging fruit:**
  - Start with batched capture (return on `/v1/execute` and stream Exit frame) and tighten later.

**Option B — In-world shim deployment + PATH interception (world filesystem mutation)**

- **Pros:**
  - Reuses existing shim logging mechanics and redaction patterns.
  - Avoids ptrace restrictions in environments where ptrace is disabled.
- **Cons:**
  - Requires mutating the world filesystem and/or PATH to ensure coverage, which is a security and maintenance hazard.
  - Harder to guarantee completeness (workloads can bypass PATH; static binaries; direct syscalls).
- **Cascading implications:**
  - Requires shim deployment lifecycle inside the world boundary (versioning, upgrades, cleanup).
  - Requires stronger “no bypass” posture assumptions than Substrate currently makes for in-world workloads.
- **Risks:**
  - Increases attack surface and policy complexity.
  - Creates parity drift between host and world capture mechanisms.
- **Unlocks:**
  - Potential cross-kernel portability if ptrace is unavailable, but at the cost of invasive change.
- **Quick wins / low-hanging fruit:**
  - None that preserve the ADR’s “no world mutation” posture.

**Recommendation**

- **Selected:** Option A — ptrace-based process tree capture (Linux backend).
- **Rationale (crisp):** It provides structured, complete-enough process tree telemetry without mutating world PATH/filesystem, keeping the world boundary posture coherent.

**Follow-up tasks (explicit)**

- WPEP2: extend world backend execution path to capture exec/exit events (ptrace) with caps + truncation summaries and explicit `argv_omitted: true`.
- WPEP1/WPEP2: extend world-service response types to return `process_events` plus deterministic diagnostics.
- WPEP2: add a Linux-backed smoke command that deterministically spawns children and asserts parent/child relationships are present.

---

### DR-0002 — Env capture minimization (allowlist-only vs full redacted map)

**Decision owner(s):** Shell + World-Agent + World runtime  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Per-process env capture is high-risk for secret leakage. We need a deterministic policy that is safe by default and still useful for debugging.

**Option A — Allowlist-only env capture (redacted values)**

- **Pros:**
  - Strong safety posture: dramatically reduces the chance of persisting secrets.
  - Keeps event sizes bounded and predictable.
  - Easier to reason about and test.
- **Cons:**
  - Less convenient for debugging issues tied to arbitrary env keys.
- **Cascading implications:**
  - Requires an explicit allowlist contract and tests that enforce it.
  - Requires aggressive redaction for proxy vars and URL-like values.
- **Risks:**
  - Operators may request “just log everything” in incident scenarios; must resist by default.
- **Unlocks:**
  - Enables safe default observability while leaving room for future opt-in modes.
- **Quick wins / low-hanging fruit:**
  - Start with no env capture (or allowlist-only) and add allowlisted keys incrementally.

**Option B — Full env map capture with redaction**

- **Pros:**
  - Maximizes debugging information.
- **Cons:**
  - Redaction is error-prone at scale; secret leakage becomes a matter of “when,” not “if”.
  - Increases trace volume substantially.
- **Cascading implications:**
  - Requires a robust, well-tested redaction engine that can handle key/value patterns and “flag consumes next arg” analogs for env sources.
- **Risks:**
  - Security incident risk due to leaked credentials in trace logs.
- **Unlocks:**
  - Potentially richer diagnostics, but it conflicts with Substrate’s default safety posture.
- **Quick wins / low-hanging fruit:**
  - None aligned with safe defaults.

**Recommendation**

- **Selected:** Option A — Allowlist-only env capture (redacted values).
- **Rationale (crisp):** The safety posture dominates; full env capture is too risky to be a default trace surface.

**Follow-up tasks (explicit)**

- WPEP3: define the env allowlist contract and redaction rules in the schema/spec docs.
- WPEP3: add unit tests asserting (a) non-allowlisted keys are omitted and (b) allowlisted proxy vars redact credentials.

---

### DR-0003 — Failure posture when tracing is unavailable (degrade vs fail execution)

**Decision owner(s):** Shell + World-Agent + World runtime  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Subprocess tracing is observability. Command execution correctness must not depend on tracing availability (ptrace restrictions, kernel variance, or transient internal failures).

**Option A — Degrade gracefully (execution succeeds; events omitted; explicit diagnostics)**

- **Pros:**
  - Preserves command execution correctness and avoids breaking workflows.
  - Works across environments where ptrace is restricted.
  - Keeps observability honest via explicit “unavailable/error” diagnostics (no silent omission).
- **Cons:**
  - Observability is not guaranteed; some runs will lack process events.
- **Cascading implications:**
  - Requires deterministic diagnostics in the world-service response and stable logging to trace.jsonl.
  - Requires tests that distinguish “no subprocesses spawned” vs “tracing unavailable”.
- **Risks:**
  - Operators may misinterpret missing events if diagnostics are not surfaced clearly.
- **Unlocks:**
  - Allows shipping v1 parity quickly while platform restrictions are explored.
- **Quick wins / low-hanging fruit:**
  - Implement diagnostics first even before ptrace capture is complete (plumbing path).

**Option B — Fail execution when tracing fails/unavailable**

- **Pros:**
  - Guarantees process events exist for every execution (if it runs at all).
- **Cons:**
  - Turns observability into an availability dependency; breaks user workflows in restricted environments.
  - Encourages unsafe workarounds (disabling world, running outside Substrate) in order to “get work done”.
- **Cascading implications:**
  - Requires a new policy/config surface to choose when to fail.
  - Requires operator docs for ptrace/kernel prerequisites on every platform.
- **Risks:**
  - Net-negative outcome: reduced adoption and more policy bypass.
- **Unlocks:**
  - Stronger audit posture only in environments where ptrace is guaranteed.
- **Quick wins / low-hanging fruit:**
  - None compatible with Substrate’s “secure execution layer” UX.

**Recommendation**

- **Selected:** Option A — Degrade gracefully (execution succeeds; events omitted; explicit diagnostics).
- **Rationale (crisp):** Observability must not become an availability gate; “explicit degrade” preserves workflow and keeps auditability honest.

**Follow-up tasks (explicit)**

- Implement deterministic `process_events_status` + `process_events_reason` plumbing for both `/v1/execute` and `/v1/stream` Exit frames.
- Ensure shell trace writes include a single, structured indicator when process events were unavailable for a run.

---

### DR-0004 — Span parent linkage correctness (capture at start vs read env at finish)

**Decision owner(s):** Shell + Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- At planning time, `ActiveSpan.finish()` read `SHIM_PARENT_SPAN` at finish time. Since span start could mutate `SHIM_PARENT_SPAN`, this yielded self-parent spans and broke trace tree reconstruction. The landed fix now captures the parent at span start and restores `SHIM_PARENT_SPAN` through span-lifecycle guards, but this decision remains the rationale for that invariant.

**Option A — Capture parent at span start; enforce env stack discipline**

- **Pros:**
  - Eliminates self-parent bugs deterministically.
  - Produces stable span trees required for correlating subprocess events.
  - Provides a clear contract for nested execution scopes (push/pop parent span).
- **Cons:**
  - Requires a small change to span lifecycle state and some additional tests.
- **Cascading implications:**
  - The span lifecycle must store the captured parent span id and restore `SHIM_PARENT_SPAN` on drop/finish, whether that logic lives in `ActiveSpan` directly or in the owning shell/shim call site.
  - Requires tests covering nested spans and multiple finish paths.
- **Risks:**
  - Minimal; mostly implementation correctness.
- **Unlocks:**
  - Improves trace reliability independent of process event capture.
- **Quick wins / low-hanging fruit:**
  - Fix can land independently and immediately improves replay/graph reconstruction.

**Option B — Continue reading `SHIM_PARENT_SPAN` at finish time**

- **Pros:**
  - No code changes.
- **Cons:**
  - Produces incorrect parent linkage in common nested execution patterns.
  - Makes process event correlation unreliable.
- **Cascading implications:**
  - Forces downstream consumers to attempt to “repair” trees heuristically (not acceptable).
- **Risks:**
  - Continued correctness bugs in trace tree reconstruction.
- **Unlocks:**
  - None.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Capture parent at span start; enforce env stack discipline.
- **Rationale (crisp):** Without correct parent linkage, process events cannot be correlated reliably; the fix is small and broadly beneficial.

**Follow-up tasks (explicit)**

- Update `crates/trace/src/span.rs` to store parent span id at start and reuse it on finish.
- Add tests proving no self-parent spans and correct parent restoration for nested spans.

---

### DR-0005 — Cross-component correlation bridge (shell `cmd_id` ↔ shim `span_id`)

**Decision owner(s):** Shell + Shim + Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Analysts need to join shell command summary events (`cmd_id`) to shim spans (`span_id`) and world process events without relying on “IDs look similar” heuristics. This becomes mandatory once we add high-volume process events.

**Option A — Add explicit bridge fields (incremental, additive)**

- **Pros:**
  - Minimal change surface; no refactor of existing IDs.
  - Enables deterministic joins in `jq`/SQL: shell `command_*` → `span_id`, and shim spans → `parent_cmd_id`.
  - Works across host/world/backends because it is carried via env (`SHIM_PARENT_CMD_ID`) and trace fields.
- **Cons:**
  - Adds some duplication (IDs appear in multiple records).
- **Cascading implications:**
  - Shell `command_start`/`command_complete` events MUST include `span_id` when a span exists.
  - Shim span records MUST include `parent_cmd_id` (from env `SHIM_PARENT_CMD_ID`) when present.
- **Risks:**
  - None material; additive fields only.
- **Unlocks:**
  - Enables reliable “follow the story” workflows and cross-component attribution.
- **Quick wins / low-hanging fruit:**
  - Add `SHIM_PARENT_CMD_ID` propagation to script mode.

**Option B — Introduce a new unified ID and refactor all emitters**

- **Pros:**
  - One canonical identifier for everything.
- **Cons:**
  - Requires a cross-repo refactor and migration of existing traces/consumers.
  - Easy to accidentally break joinability during the transition.
- **Cascading implications:**
  - Touches shell, shim, world-service, replay tooling, docs, and any downstream analytics.
- **Risks:**
  - High coordination cost for limited immediate value.
- **Unlocks:**
  - A cleaner long-term model, but not required for this track.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Add explicit bridge fields (incremental, additive).
- **Rationale (crisp):** It solves joinability now with minimal risk and without delaying the core parity work.

**Follow-up tasks (explicit)**

- Add `span_id` to shell `command_*` events when a span exists.
- Add `parent_cmd_id` to span records when `SHIM_PARENT_CMD_ID` is present.
- Ensure script mode sets `SHIM_PARENT_CMD_ID` and emits world-fs-strategy contract fields.

---

### DR-0006 — Deny outcome clarity on completion records (outcome field vs origin split)

**Decision owner(s):** Shell + Shim + Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Completion records currently can be misread for policy denies (e.g., `execution_origin: host`, `exit: 126`) unless the reader joins to the start record.
- This becomes higher-stakes once ADR-0029 lands: the host router daemon’s v1 trigger set includes `command_complete` / root span completion events, so “deny vs executed” must be detectable from a single completion record to avoid accidentally triggering follow-on work from denied commands.

**Option A — Add `outcome` to completion spans (set explicitly on deny)**

- **Pros:**
  - Minimal change and easy for analysts to filter: `outcome == "denied"`.
  - Avoids redefining what `execution_origin` means.
  - Aligns with ADR-0029’s trace-tailing trigger model (bus rules can gate on `outcome` without needing to join across events).
- **Cons:**
  - Requires discipline: emitters must set it on deny paths.
- **Cascading implications:**
  - Shim deny path MUST set `outcome: "denied"` on the completion span.
  - Shell deny path (if/when present) MUST do the same.
- **Risks:**
  - If not consistently set, ambiguity remains; tests must enforce it.
- **Unlocks:**
  - Clear outcome classification for future router/workflow triggers.
- **Quick wins / low-hanging fruit:**
  - Add a unit/integration test that asserts deny completion includes `outcome: "denied"`.

**Option B — Split `execution_origin` into planned vs actual**

- **Pros:**
  - More expressive for nuanced routing decisions.
- **Cons:**
  - Larger schema change; touches many emitters and docs.
  - Still needs a deny marker to avoid confusion (`actual_origin: null` is easy to miss).
- **Cascading implications:**
  - Requires updating trace schema docs and all record writers.
- **Risks:**
  - Higher coordination cost; not needed to unblock parity work.
- **Unlocks:**
  - A richer model for future scheduling/routing analysis.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Add `outcome` to completion spans (set explicitly on deny).
- **Rationale (crisp):** It resolves the ambiguity with the smallest change surface and clear analyst ergonomics.

**Follow-up tasks (explicit)**

- Add optional `outcome` field to span schema and set it on deny completion paths.

---

### DR-0007 — Policy decision visibility on completion spans (duplicate vs minimal summary)

**Decision owner(s):** Shell + Shim + Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Many consumers summarize outcomes from `command_complete` only. If policy decision detail is present only on `command_start`, deny/allow-with-restrictions reasoning is easy to miss.

**Option A — Duplicate `policy_decision` onto completion spans**

- **Pros:**
  - Maximizes usability: one record contains both outcome and “why”.
  - Avoids inventing a parallel “summary” schema.
- **Cons:**
  - Some duplication in JSONL size.
- **Cascading implications:**
  - Span lifecycle must carry the start decision through to finish.
- **Risks:**
  - Minimal; additive and deterministic.
- **Unlocks:**
  - Simplifies downstream analytics and audit workflows.
- **Quick wins / low-hanging fruit:**
  - Add a test that asserts completion spans include policy decision when set at start.

**Option B — Add minimal completion fields (`policy_action`, `policy_reason`, `policy_restrictions_count`)**

- **Pros:**
  - Smaller record size than duplicating the object.
- **Cons:**
  - Creates a second schema surface that can drift from `policy_decision`.
  - Loses information (restrictions detail) unless we keep expanding the summary.
- **Cascading implications:**
  - Requires documenting and maintaining two parallel representations.
- **Risks:**
  - “Summary drift” causes audits to become unreliable.
- **Unlocks:**
  - Slight size savings.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Duplicate `policy_decision` onto completion spans.
- **Rationale (crisp):** Usability and auditability dominate; the duplicated bytes are worth the clarity.

**Follow-up tasks (explicit)**

- Persist `policy_decision` on both `command_start` and `command_complete` spans when known.

---

### DR-0008 — Shim completion ergonomics (emit `duration_ms` vs derive from timestamps)

**Decision owner(s):** Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- Shell command summaries include `duration_ms`, but shim completion spans do not. Deriving duration from timestamps is possible but brittle when timestamps are inconsistent in precision/format or when clocks skew across emitters.

**Option A — Emit `duration_ms` on `command_complete` spans**

- **Pros:**
  - Improves analyst ergonomics and avoids timestamp parsing pitfalls.
  - Uses monotonic time (`Instant`) for correctness.
- **Cons:**
  - Adds one more field to the span schema.
- **Cascading implications:**
  - `ActiveSpan` must record a start instant and compute duration at finish.
- **Risks:**
  - Minimal; additive.
- **Unlocks:**
  - Enables quick “what was slow?” queries without additional tooling.
- **Quick wins / low-hanging fruit:**
  - Add a basic test that `duration_ms` exists and is non-negative.

**Option B — Derive duration from `ts` fields**

- **Pros:**
  - No schema changes.
- **Cons:**
  - Requires consistent timestamp formatting/precision across emitters.
  - Makes query tooling harder (string parsing, timezone pitfalls).
- **Cascading implications:**
  - Forces standardization work first.
- **Risks:**
  - Analysts get wrong durations silently.
- **Unlocks:**
  - None.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**

- **Selected:** Option A — Emit `duration_ms` on `command_complete` spans.
- **Rationale (crisp):** It is a low-effort improvement with high usability and correctness value.

**Follow-up tasks (explicit)**

- Add optional `duration_ms` field to completion spans and compute it using a monotonic clock.

---

### DR-0009 — Preexec/builtin command privacy posture (marker vs silent raw)

**Decision owner(s):** Shell + Trace  
**Date:** 2026-02-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**

- When `SUBSTRATE_ENABLE_PREEXEC=1`, the bash preexec hook can emit `builtin_command` records containing raw `BASH_COMMAND`. This can leak credentials/tokens unless we provide explicit, queryable safety metadata.

**Option A — Omit command body from canonical trace (metadata-only) + optional raw debug log**

- **Pros:**
  - Eliminates the primary secret-leak risk from canonical `trace.jsonl` (no `BASH_COMMAND` body recorded).
  - Still preserves attribution/value: metadata + correlation (`parent_cmd_id`) is enough for “what was happening” joins and noise scoping.
  - Keeps an escape hatch: raw body can be written to a separate, explicit debug-only file when operators accept the risk.
- **Cons:**
  - Reduces immediate human legibility of `builtin_command` records (no “what exact command string?” without the debug log).
- **Cascading implications:**
  - Canonical trace `builtin_command` records MUST NOT include the command body.
  - Canonical trace `builtin_command` records MUST include correlation (`parent_cmd_id`) and a deterministic marker (e.g., `command_omitted: true`).
  - Raw debug logging (if enabled) MUST write to a separate file path (never to `trace.jsonl`) and MUST include `may_contain_secrets: true`.
- **Risks:**
  - Operators may enable raw debug logging in sensitive contexts; docs must warn clearly.
- **Unlocks:**
  - Keeps preexec usable while preserving Substrate’s “no secret logging by default” posture.
- **Quick wins / low-hanging fruit:**
  - Implement metadata-only canonical records immediately; raw debug log stays opt-in.

**Option B — Include command body, but require hardened redaction (codebase-wide)**

- **Pros:**
  - Retains maximum debug value in the canonical trace (exact command string).
- **Cons:**
  - Requires stronger redaction than currently exists (especially for token/header/URL patterns and “flag consumes next arg” semantics inside shell commands).
  - Higher risk surface: any redaction miss becomes a secret leak in the canonical trace.
- **Cascading implications:**
  - Requires a shared, well-tested redaction engine usable across shim/shell/world-service/preexec (not ad-hoc).
  - Requires a clear “raw capture is on” trace meta signal (future) and/or per-record marker.
- **Risks:**
  - Elevated risk of leaking secrets into canonical trace files even with best-effort redaction.
- **Unlocks:**
  - Richer built-in debugging without needing separate files.
- **Quick wins / low-hanging fruit:**
  - None; redaction hardening is the prerequisite.

**Recommendation**

- **Selected:** Option A — Omit command body from canonical trace (metadata-only) + optional raw debug log.
- **Rationale (crisp):** Canonical trace must be safe-by-default; raw preexec capture is too likely to contain secrets to record without a hardened redaction system.

**Follow-up tasks (explicit)**

- Update bash preexec `builtin_command` emission in `trace.jsonl` to omit the body while preserving correlation (`parent_cmd_id`) + `command_omitted: true`.
- Add an explicit opt-in env var for raw debug logging to a separate file (not `trace.jsonl`).
- Add a backlog item to support “include body with hardened redaction” in the future (codebase-wide redaction improvements).
