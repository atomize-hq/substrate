# R8 Spec: Agent Drift Sentinel Interpretation Consolidation

Status: **CANDIDATE / AWAITING FRESH REVIEW**. This document specifies a future implementation; it
does not authorize R8 code. The complete R8 MAP/SPEC/PLAN/TASKS family must be fresh-review-clean
before implementation starts.

## Objective

Collapse replay/live checkpoint interpretation duplication into one typed sentinel seam without
moving semantic ownership out of `agent-drift-analyzer`. For the same valid checkpoint sequence and
same-session history, replay and live must produce the same interpretation before scheduling and
presentation.

## Fixed Assumptions

1. The analyzer `Checkpoint` is the semantic source of truth for drift state, session progress,
   turn/archetype context, and v0.8 `DelegationContext`.
2. Replay/live origin is transport context, not a semantic input.
3. A previous checkpoint is usable only when its `session_id` matches the current checkpoint.
4. Parent waits, spawn/results, or other orchestration do not establish child-local semantics.
5. Existing scheduler, adjudication, cursor, and delivery behavior is a protected contract.

## Current Contract Topology

| File | Current responsibility |
|---|---|
| `crates/agent-drift-sentinel/src/input.rs` | `CheckpointCursor`, `ReplayCheckpointBundle`, replay loading/sorting, supported-version checks, raw `validate_checkpoint_contract`. |
| `crates/agent-drift-sentinel/src/live_input.rs` | `LiveCheckpointEvent`, sequence validation, duplicated raw version/field checks, typed `verify_live_checkpoint_compatibility`. |
| `crates/agent-drift-sentinel/src/live_runtime.rs` | `observe`/`drain`, per-session previous checkpoint, scheduler call, presentation call. |
| `crates/agent-drift-sentinel/src/real_session_live.rs` | `poll_once`, analyzer pipeline, per-session freshness, delivery, and persisted cursor state. |
| `crates/agent-drift-sentinel/src/operator_surface.rs` | diagnostics/presentation types plus version-aware posture and evidence interpretation. |

Today the literal supported-version set and raw required-field rules are duplicated between replay
and live. Replay and live also reach posture/evidence through different pre-presentation paths.

## Proposed Interface

The reviewed plan must introduce one explicitly owned interpretation module rather than place
shared semantics under a replay- or live-named adapter. The intended contract is:

```rust
enum CheckpointSchemaVersion {
    V0_2, V0_3, V0_4, V0_5, V0_6, V0_7, V0_8,
}

struct CheckpointInterpretationInput<'a> {
    checkpoint: &'a Checkpoint,
    previous_same_session: Option<&'a Checkpoint>,
}

struct CheckpointInterpretation {
    checkpoint: Checkpoint,
    schema_version: CheckpointSchemaVersion,
    cursor: CheckpointCursor,
    warning_fingerprint: String,
    flagged: bool,
    max_flagged_score: Option<u8>,
    posture: Option<CheckpointPosture>,
    evidence: Vec<EvidenceRef>,
    delegation: Option<DelegationContext>,
}

fn validate_serialized_checkpoint(value: &serde_json::Value)
    -> Result<CheckpointSchemaVersion, CheckpointContractError>;

fn interpret_checkpoint(input: CheckpointInterpretationInput<'_>)
    -> Result<CheckpointInterpretation, CheckpointContractError>;
```

Names may change only in the reviewed PLAN; the invariants may not. `interpret_checkpoint` is the
single replay/live semantic entry point. It has no replay/live mode flag. The typed result contains
facts, not console strings or scheduler/adjudicator decisions.

Existing supported public APIs (`LiveCheckpointCompatibility`,
`verify_live_checkpoint_compatibility`, `CheckpointPresentation`, and `present_checkpoint*`) must
remain behavior-compatible for supported inputs. Migration should be additive or use compatibility
facades; do not force callers onto parallel versioned APIs.

## Compatibility Matrix

| Schema | Central requirements | Interpretation |
|---|---|---|
| v0.2 | Existing base checkpoint fields; drift `state` may be absent in serialized input. | Preserve the current bounded legacy posture/evidence rules using only same-session history. |
| v0.3 | Explicit `drift_scores[*].state`. | Analyzer state is authoritative. |
| v0.4 | v0.3 plus non-null `turn_context`. | Analyzer state/context are authoritative. |
| v0.5 | v0.4 plus non-null `session_archetype`. | Analyzer state/context are authoritative. |
| v0.6 | v0.5 plus non-null `session_progress`. | Analyzer state/progress are authoritative. |
| v0.7 | Same required semantic fields as v0.6. | Preserve current v0.7 behavior. |
| v0.8 | v0.7 plus non-null serialized `delegation`. | Render typed analyzer delegation; never reconstruct links from orchestration. |

Only the exact literal versions above are supported. A missing or malformed v0.3-v0.8 explicit
field is a contract error, never a request to run v0.2 compatibility inference.

## Semantic Ownership

- Sentinel may normalize analyzer `DriftState` into `CheckpointPosture` and select the analyzer
  evidence associated with that posture.
- Sentinel must not rescore drift, synthesize progress, reinterpret objectives, discover delegation
  links, or aggregate child state into a parent.
- v0.8 delegation output is copied from typed `DelegationContext`. No parsing of `spawn_agent`,
  wait/result prose, filenames, timing, or parent messages is permitted.
- When child evidence is unavailable, the analyzer's `opaque`/`partial` visibility and insufficient
  evidence remain intact. Parent orchestration alone never proves child progress, drift, or
  completion.

## Presentation-First Operator Surface

`CheckpointPresentation` and `CheckpointDiagnosticsSummary` remain presentation models.
Presentation receives `CheckpointInterpretation` plus trigger/decision/policy, then only:

1. truncates and formats fields;
2. orders evidence and typed context lines;
3. labels the independent scheduler trigger and analyzer-derived posture; and
4. chooses visible/silent rendering from the existing decision/policy contract.

It must not inspect schema strings, classify analyzer state, apply legacy evidence prefixes, or
infer delegation. Replay and live render the same interpretation object for matching inputs.

## Unchanged Scheduling, Adjudication, and Delivery

- `ReplayScheduler::observe`, `SchedulerPolicy`, trigger meanings, cooldown/deduplication, and
  scheduler state are unchanged.
- Adjudication request shaping, fallback, response handling, and operator notes are unchanged.
- `LiveSessionCoordinator::poll_once` retains append-only rollout checks, sparse-startup behavior,
  verified-closure enforcement, per-session cursor freshness, and delivery ordering.
- Interpretation occurs before presentation and before a cursor is recorded/persisted as delivered.
  A failed interpretation produces no scheduler decision, operator event, or cursor advancement.

No edit to `scheduler.rs`, `adjudication.rs`, or their tests belongs in R8 unless a later separately
reviewed spec authorizes it.

## Failure Semantics

1. Unsupported schema, missing required serialized fields, empty required typed identifiers/text,
   or invalid same-session assumptions fail closed with a structured contract error.
2. Replay adapters retain artifact path/line context and existing mixed-version/empty-bundle errors;
   live fixture adapters retain path/line and event-sequence errors.
3. Adapter error types may wrap the central error, but supported observable error categories and
   useful path/line/schema/field detail must not regress.
4. No v0.3-v0.8 failure falls back to legacy inference. No prior checkpoint from another session is
   consulted.
5. A v0.8 delegation gap fails contract validation; it never triggers raw-event reconstruction.

## Migration Contract

1. Add the central schema profile, raw validator, typed interpretation, and focused matrix tests.
2. Route replay raw validation and replay presentation inputs through it without changing replay
   sorting, cursor filtering, report grouping, or scheduler behavior.
3. Route fixture/live validation and `LiveRuntime::observe` through the same seam while preserving
   `LiveCheckpointCompatibility` behavior.
4. Reduce `operator_surface.rs` to formatting/policy projection; remove its independent version and
   posture/evidence classification only after replay/live parity tests pass.
5. Keep real-session delivery and persistence after successful interpretation, then run the full
   sentinel wall. Each migration packet must be independently review-clean before the next cutover.

## Exact Future Source Contract

The PLAN/TASKS may packetize only this inventory unless a fresh spec amendment expands it:

| Path | Permitted R8 change |
|---|---|
| `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` | New sole owner of schema profiles, serialized contract rules, typed interpretation, posture/evidence normalization, and v0.8 typed delegation projection. |
| `crates/agent-drift-sentinel/src/lib.rs` | Minimal module/export wiring; preserve supported public surfaces. |
| `crates/agent-drift-sentinel/src/input.rs` | Delegate raw replay validation/interpretation; retain bundle, sorting, cursor, and replay-specific errors. |
| `crates/agent-drift-sentinel/src/live_input.rs` | Delegate raw/typed compatibility; retain event/source/sequence validation and compatibility facade. |
| `crates/agent-drift-sentinel/src/live_runtime.rs` | Consume the shared interpretation before the unchanged scheduler/presentation sequence. |
| `crates/agent-drift-sentinel/src/real_session_live.rs` | Only integration ordering needed to guarantee no delivery/persistence on interpretation failure. |
| `crates/agent-drift-sentinel/src/operator_surface.rs` | Render typed interpretation; remove duplicated semantic/version decisions. |

No analyzer, compactor, schema, fixture corpus, scheduler, adjudication, CLI, or operator-sink
production edit is authorized by this spec.

## Exact Future Test Contract

| Path | Required proof |
|---|---|
| `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs` | Exact v0.2-v0.8 matrix; same-session history; explicit-state precedence; structured failures; typed v0.8 delegation; parent-orchestration negative witness. |
| `crates/agent-drift-sentinel/tests/replay_input.rs` | Replay raw-field/version behavior, sorting, mixed-version failure, and cursor behavior unchanged. |
| `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs` | v0.2 and v0.3-v0.8 compatibility/presentation behavior unchanged, including v0.8 state-backed evidence/delegation. |
| `crates/agent-drift-sentinel/tests/live_input.rs` and `tests/live_input_adapter.rs` | Append-only event/cursor and fixture adapter errors unchanged. |
| `crates/agent-drift-sentinel/tests/live_runtime.rs` | Shared interpretation precedes unchanged scheduling; repeated-failure trigger remains distinct from posture. |
| `crates/agent-drift-sentinel/tests/real_session_live.rs` | Per-session cursors, verified closure, sparse startup, restart, regression failures, and no delivery on interpretation error. |
| `crates/agent-drift-sentinel/tests/operator_surface.rs` | Presentation consumes typed facts; v0.2 legacy and explicit-state posture/evidence rendering remain stable. |
| `crates/agent-drift-sentinel/tests/live_end_to_end.rs` | Replay/live parity for diagnostics, headlines, turn context, archetype, progress, posture, session locality, and trigger/posture separation. |

Required verification wall after implementation: focused changed targets, then
`cargo test -p agent-drift-sentinel -- --nocapture`, workspace formatting/clippy/tests required by
repository policy, staged GitNexus change detection, and cached-diff inspection. Counts must be
recorded from the implementation run; this candidate spec claims none.

## Success Criteria

| Gate | Testable criterion |
|---|---|
| `CTX-R8-01` | R7 analyzer/delegation output remains the sole semantic input; no analyzer contract change is required. |
| `CTX-R8-02` | MAP/SPEC/PLAN/TASKS are fresh-review-clean before the first source/test edit. |
| `CTX-R8-03` | Replay and live call one `interpret_checkpoint` seam and parity tests compare its outputs. |
| `CTX-R8-04` | One literal compatibility matrix covers v0.2 and every version v0.3-v0.8, including fail-closed gaps. |
| `CTX-R8-05` | Operator presentation has no schema/state/delegation inference and renders matching replay/live inputs identically. |
| `CTX-R8-06` | Scheduler/adjudication outputs, real-session closure, delivery order, and per-session cursor behavior are unchanged. |

These criteria remain unproven. Fresh review of the complete docs family is the next gate; R8
implementation remains blocked.
