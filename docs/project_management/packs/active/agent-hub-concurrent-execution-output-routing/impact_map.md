# agent-hub-concurrent-execution-output-routing — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- Spec manifest:
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

### Create
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/spec_manifest.md` — required spec ownership map (planning v4)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/impact_map.md` — impact map (planning v4)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/contract.md` — operator-facing output routing + config contract
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md` — authoritative structured event schema
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/telemetry-spec.md` — authoritative trace record requirements
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/platform-parity-spec.md` — platform contract + validation evidence requirements
- Slice specs:
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-spec.md` — event envelope + trace persistence foundation
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md` — REPL routing during PTY passthrough + buffering + warnings + config knob
- Planning Pack completion (required before execution triads begin):
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/tasks.json`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/ci_checkpoint_plan.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/session_log.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/kickoff_prompts/*`
- Validation artifacts (required by ADR-0017):
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/manual_testing_playbook.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`
- Execution gates (required because tasks.json `meta.execution_gates=true`):
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/execution_preflight_report.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-closeout_report.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-closeout_report.md`
- Planning quality gate artifact (required before any triads start; written by the reviewer, not by execution triads):
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/quality_gate_report.md`
- (implementation-time; code/test surface)
  - `crates/shell/tests/repl_output_routing.rs` (or similar) — integration coverage for PTY passthrough + concurrent structured events
  - `crates/common/tests/agent_event_schema.rs` (or similar) — serde/schema invariants for envelope fields and required keys

### Edit
- `crates/common/src/agent_events.rs` — expand structured agent event envelope to include required attribution/correlation fields (ADR-0017 / DR-0003 / DR-0008)
- `crates/shell/src/execution/agent_events.rs` — ensure event creation helpers can populate the enriched envelope; keep formatting stable for human rendering
- `crates/shell/src/repl/async_repl.rs` — enforce “no PTY injection”, bounded buffering + drop-count summary, and trace persistence hooks during passthrough
  - Replace the current marker-only warning line behavior with the deterministic dropped-count summary contract.
  - Ensure buffering cap is read from effective config instead of a constant.
- `crates/shell/src/execution/config_model.rs` — add `repl.max_pty_buffered_lines` to the strict config schema, precedence model, and explain surfaces
- `docs/CONFIGURATION.md` — document `repl.max_pty_buffered_lines` (default, bounds, precedence, invalid handling)
- `docs/TRACE.md` — document new trace record types (`agent_event`, warning codes) and required fields for correlation
- `docs/project_management/packs/sequencing.json` — add a sprint entry for this feature so task dependencies and sequencing are aligned before execution begins
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` — remove placeholder sequencing language and link to `tasks.json` and `ci_checkpoint_plan.md`

### Deprecate
- None (this ADR refines interactive behavior; no stable public APIs are deprecated).

### Delete
- None.

## Cascading implications (behavior/UX)

### CLI / UX
- Change: PTY passthrough no longer prints live structured agent events; it buffers/drops and flushes after passthrough ends.
  - Direct impact:
    - TUIs remain correct (no corruption from host-printed structured lines).
    - Operators may not see live structured status updates during long passthrough sessions.
  - Cascading impact:
    - A deterministic suppression summary warning is required so operators and tools can see that output was suppressed.
    - Structured events MUST be persisted to trace so operators/tools can recover suppressed context.
  - Contradiction risks:
    - Existing behavior appends a single marker string (`structured output dropped during :pty (buffer full)`); this is not sufficiently deterministic for programmatic consumers and conflicts with the “dropped-count summary” requirement.

### Config / env vars / paths
- Change: `repl.max_pty_buffered_lines` becomes an operator-tunable config key in the strict schema.
  - Direct impact:
    - Operators can tune suppression behavior per workspace (noisy agents) vs globally (defaults).
  - Cascading impact:
    - Invalid type/parse must fail deterministically at the config boundary (exit code `2`).
    - Out-of-range clamping must be observable via a structured warning (persisted; not PTY-injected).
  - Contradiction risks:
    - If the key is added without strict schema updates (`deny_unknown_fields`), the config system will fail closed in unexpected places; schema + error posture must be updated together.

### Policy / isolation / security posture
- Change: structured event channels and attribution fields become audit-relevant surfaces.
  - Direct impact:
    - Structured events must be safe-to-print and must not leak secrets.
  - Cascading impact:
    - The envelope schema must constrain `channel` and enforce top-level attribution/correlation fields for deterministic joins.
  - Contradiction risks:
    - Allowing arbitrary user-provided strings into `channel` would risk secret leakage into trace; producers must drop unsafe values.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - Overlap surfaces: PTY passthrough contract, out-of-band PTY bytes during prompt, “no injection” invariant.
  - Conflict: no.
  - Resolution (explicit): ADR-0017 refines the structured-event path to remain out-of-band and explicitly non-injective during passthrough.
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces: correlation vocabulary (`orchestration_session_id`, `run_id`, `agent_id`, `role`, `world_id`, `span_id`), trace record expectations.
  - Conflict: potential (field naming/placement drift).
  - Resolution (explicit): ADR-0017 requires top-level correlation fields on agent events; ADR-0028 is additive and must remain consistent with this envelope (no reshapes).
- ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - Overlap surfaces: event plane semantics, world reuse (`world_id`), attribution requirements.
  - Conflict: no.
  - Resolution (explicit): ADR-0017 defines the envelope + rendering contract; hub-core ADRs consume the envelope and must not redefine it.
- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Overlap surfaces: backend identity (`backend_id`) and correlation propagation for agent-driven execution.
  - Conflict: no.
  - Resolution (explicit): CLI-backend routing defers to ADR-0017 for structured event envelope rules and to ADR-0028 for trace vocabulary.

### Related Phase 8 tracks (cross-cutting; use ADRs/registry)
- Phase 8 registry (cross-cutting lock): `docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md`
- CLI backend engine: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
- Router daemon: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- Workflow engine: `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

## Follow-ups (explicit)

- Decision Register entries required:
  - None (ADR-0017 already has DR-0001..DR-0010 capturing the key A/B choices).
- Spec updates required (if any):
  - If ADR-0028 requires additional required correlation fields, update:
    - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md`
    - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/telemetry-spec.md`
