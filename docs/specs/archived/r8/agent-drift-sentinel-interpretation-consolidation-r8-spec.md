# R8 Spec: Agent Drift Sentinel Interpretation Consolidation

Status: **R8-IMPLEMENT COMPLETE / R8 FAMILY TERMINALLY COMPLETE / R8-6 RECEIPT `549160ebc` FRESH
INDEPENDENT BUILT-IN `default` `CLEAN` / `CTX-R8-01..06` PROVEN / TERMINAL TRANSITION COMMIT
`65eac5ab` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` / R8-IMPLEMENT EXIT GATE REVIEW-CLEAN /
CURRENT REVIEW RECEIPT PENDING ITS OWN FRESH INDEPENDENT REVIEW**.

R8-SPEC and R8-IMPLEMENT are `COMPLETE`; the R8 family is terminally complete. R8-1
`ec2c5da7d`, R8-2 `08e0d0e2a` + `5669e1f6e` + `911dd49b`, R8-3 `963a8202f`, R8-5.1
`8ef705d8a`, and R8-5.2 `2616c4651` + `bc64fe962` + `cb1a276b7` + `813e1db17` + `ad6340190`
each received fresh independent built-in `default` `CLEAN` with no actionable findings. The exact
R8-4 implementation/review-fix series
`f3d19687a` + `4027e2e82` + `ae5b45408` + `ba7979b4f` + `348b34038` + `246b2fb72` +
`888553555` + `c7bf1dcff` + `3fdbf4bd5` + `87963d46d` + `4227ae920` + `b05c7843d` +
`c39126c1b` + `3cc2a8aba` + `49a1e7dc8` + `4552b89f6` + `910597cb7` + `c172e252a` +
`293bbf708` + `0692cd1bc` + `00111d66a` received fresh independent built-in `default` `CLEAN` with
no actionable findings. Five-doc R8-6 receipt `549160ebc1d93b26fdbe8203d748dfb1a64787ef`
received fresh independent built-in `default` `CLEAN` with no findings.

At implementation HEAD `ad6340190`, the exact final wall is green: Sentinel tests pass `233` with
zero failures, and workspace tests pass `2,657` with zero failures and `2` ignored. `CTX-R8-01..06`
are `PROVEN`, and all `57` R8 implementation checkboxes/tasks are complete. Recorded decisions are
`R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A`,
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A`,
`R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A`,
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`,
`R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`,
`R8-4-HIGH-IMPACT-CENTRAL-PROJECTION-01: A`,
`R8-4-HIGH-IMPACT-COMPATIBILITY-INPUT-01: A`, and
`R8-4-CRITICAL-ZERO-EVIDENCE-LIMIT-01: A`.

Current active phase is `none` and active packet is `none` because the master sequence ends at
R8-IMPLEMENT. **NO NEXT ELIGIBLE PHASE IS DEFINED IN THIS CONTROL PACK.** No executable Prompt 1
selector exists; do not recycle R8-IMPLEMENT or invent a successor phase. Terminal transition commit `65eac5ab2ecf354a731d127014f91da5c87e6e8d` received fresh independent
built-in `default` `CLEAN` with no findings, so the terminal R8 transition and R8-IMPLEMENT exit gate
are review-clean. This Markdown-only review receipt records that already-reviewed terminal transition;
the receipt assigns itself no commit hash or review result, claims no `CLEAN` result for itself, remains
pending its own fresh independent review, and starts no further phase work.

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
6. Real-session transport-observation bookkeeping that already happens before
   `LiveRuntime::observe` is preserved and outside R8: monitor-closure tracking, pending-poll state,
   and emission-ordinal allocation may occur before a later interpretation error. The protected
   zero-effect boundary begins at the interpretation call, not at the start of `poll_once`.

## Current Contract Topology

| File | Current responsibility |
|---|---|
| `crates/agent-drift-sentinel/src/input.rs` | `CheckpointCursor`, `ReplayCheckpointBundle`, replay loading/sorting, supported-version checks, raw `validate_checkpoint_contract`. |
| `crates/agent-drift-sentinel/src/live_input.rs` | `LiveCheckpointEvent`, sequence validation, duplicated raw version/field checks, typed `verify_live_checkpoint_compatibility`. |
| `crates/agent-drift-sentinel/src/live_runtime.rs` | `observe`/`drain`, per-session previous checkpoint, scheduler call, presentation call. |
| `crates/agent-drift-sentinel/src/real_session_live.rs` | `poll_once`, analyzer pipeline, per-session freshness, delivery, and persisted cursor state. |
| `crates/agent-drift-sentinel/src/operator_surface.rs` | diagnostics/presentation types plus version-aware posture and evidence interpretation. |
| `crates/agent-drift-sentinel/src/adjudication.rs` | Read-only dependency/call-path evidence: `shape_request` calls `CheckpointPresentation::render_console_block` and preserves that rendered content, subject only to its existing truncation limit, in `operator_summary`; no R8 edit is authorized. |

Today the literal supported-version set and raw required-field rules are duplicated between replay
and live. Replay and live also reach posture/evidence through different pre-presentation paths.

## Proposed Interface

The reviewed plan must introduce one explicitly owned interpretation module rather than place
shared semantics under a replay- or live-named adapter. The intended contract is:

```rust
pub(crate) enum CheckpointSchemaVersion {
    V0_2, V0_3, V0_4, V0_5, V0_6, V0_7, V0_8,
}

pub(crate) struct CheckpointInterpretationInput<'a> {
    checkpoint: &'a Checkpoint,
    previous_same_session: Option<&'a Checkpoint>,
}

pub(crate) struct CheckpointCompatibilityProjectionInput<'a> {
    checkpoint: &'a Checkpoint,
    previous_checkpoint: Option<&'a Checkpoint>,
}

pub(crate) enum CheckpointProjectionProfile {
    Schema(CheckpointSchemaVersion),
    CompatibilityOnly,
}

pub(crate) struct CheckpointInterpretation {
    checkpoint: Checkpoint,
    projection_profile: CheckpointProjectionProfile,
    cursor: CheckpointCursor,
    warning_fingerprint: String,
    flagged: bool,
    max_flagged_score: Option<u8>,
    posture: Option<CheckpointPosture>,
    evidence: Vec<EvidenceRef>,
    delegation: Option<DelegationContext>,
}

pub(crate) fn validate_serialized_checkpoint(value: &serde_json::Value)
    -> Result<CheckpointSchemaVersion, CheckpointContractError>;

pub(crate) fn interpret_checkpoint(input: CheckpointInterpretationInput<'_>)
    -> Result<CheckpointInterpretation, CheckpointContractError>;

pub(crate) fn project_checkpoint_compatibility(
    input: CheckpointCompatibilityProjectionInput<'_>,
) -> CheckpointInterpretation;

pub(crate) fn try_render_replay_report(
    bundle: &ReplayCheckpointBundle,
    checkpoints: &[Checkpoint],
    scheduler_policy: &SchedulerPolicy,
    warning_policy: &WarningPolicy,
) -> Result<ReplayReport, InputError>;

pub(crate) fn present_interpretation(
    interpretation: &CheckpointInterpretation,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation;
```

These amended names are implementation authority. `interpret_checkpoint` remains the single
fallible replay/live semantic entry point and has no replay/live mode flag. After validation it and
the total `project_checkpoint_compatibility` facade adapter call one private projection routine for
posture, evidence, delegation, cursor, fingerprint, and flagged-score facts; both return the same
`CheckpointInterpretation`. Validated replay/live pass it to `present_interpretation`; total facades
pass it to `present_compatibility_interpretation`; both share the private
`present_interpretation_with_evidence_limit` renderer with distinct locked evidence-limit modes.
`Schema(V0_2)` remains distinct from `CompatibilityOnly`. The total adapter maps exact supported literals to `Schema(...)` without
validating their typed shape and maps every other typed facade literal to `CompatibilityOnly`,
which preserves the existing legacy-compatible total projection without claiming the input is
v0.2. It treats its optional previous checkpoint exactly as the existing public facade does and
performs no schema, field, explicit-state, or same-session validation. It neither calls
`interpret_checkpoint` nor catches a `CheckpointContractError`.

`try_render_replay_report` remains the fallible replay core used by `execute`, and the already-clean
`LiveRuntime::observe` continues to call `interpret_checkpoint` directly. Core replay/live cannot
construct or select `CompatibilityOnly`; their validation and error mapping remain unchanged. The
typed result contains facts, not console strings or scheduler/adjudicator decisions.

The new `checkpoint_interpretation` module is crate-private. `lib.rs` may declare it only as a
crate-private module, every new item is `pub(crate)` only where another Sentinel module needs it,
and R8 adds no public re-export or public API surface. R8-1's RED/GREEN matrix lives in
`checkpoint_interpretation.rs` under an in-module `#[cfg(test)] mod tests`; no external
interpretation integration target is authorized.

The following public source signatures are locked exactly:

```rust
pub fn present_checkpoint(
    checkpoint: &Checkpoint,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation;

pub fn present_checkpoint_with_previous(
    checkpoint: &Checkpoint,
    previous_checkpoint: Option<&Checkpoint>,
    trigger: TriggerClass,
    decision: &EvaluationDecision,
    warning_policy: &WarningPolicy,
) -> CheckpointPresentation;

pub fn render_replay_report(
    bundle: &ReplayCheckpointBundle,
    checkpoints: &[Checkpoint],
    scheduler_policy: &SchedulerPolicy,
    warning_policy: &WarningPolicy,
) -> ReplayReport;

pub fn execute(request: &SentinelRequest) -> Result<SentinelResult, SentinelError>;
```

`LiveCheckpointCompatibility`, `verify_live_checkpoint_compatibility`, and
`CheckpointPresentation` also remain behavior-compatible for supported inputs. The three infallible
presentation/report APIs above remain total compatibility facades with their current behavior,
including unsupported typed facade inputs, but they are not core-path validation boundaries. They
delegate to `project_checkpoint_compatibility` plus `present_compatibility_interpretation`; the
validated cores use `present_interpretation`, and both presentation adapters share one private
renderer with their separately locked evidence-limit modes. They may not own or duplicate
schema-version, analyzer-state, evidence, or delegation semantics except for the single
presence-only legacy predicate authorized by recorded decision
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`. Under that decision, the unchanged
`CheckpointPresentation::render_console_block` facade may serve the exact allowed downstream
production owner set `{ReplayReport::to_console_text, cli::run_live,
adjudication::shape_request}` after the unchanged public shapes erase the internal presence bit.
Fail-closed AST/call-path proof must independently assert both that exact owner-set equality and
exact raw direct `render_console_block` call-expression count `4`: two replay calls in
`ReplayReport::to_console_text`, one live call in `cli::run_live`, and one adjudication call in
`adjudication::shape_request` while constructing `operator_summary`. Any fifth direct call fails,
even if added within an allowed owner. None of these calls is a validation boundary. `execute` calls the additive
fallible replay core and preserves its existing result signature. No facade or core entry point may
use `unwrap`, panic, or error-swallowing/legacy fallback to cross the fallible seam. An explicit
`CompatibilityOnly` profile is a total facade contract, not an error conversion, default acceptance,
or v0.2 fallback.

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

Beyond those serialized version/required-shape rules, sentinel-owned typed non-empty validation is
the exact closed set `session_id`, `checkpoint_id`, `task_frame.objective`, and
`expected_next_step`. R8 does not add non-empty, syntax, reciprocity, topology, identifier, marker,
visibility, confidence, or evidence checks inside `DelegationContext`.

## Semantic Ownership

- Sentinel may normalize analyzer `DriftState` into `CheckpointPosture` and select the analyzer
  evidence associated with that posture.
- Sentinel must not rescore drift, synthesize progress, reinterpret objectives, discover delegation
  links, or aggregate child state into a parent.
- v0.8 delegation output is copied from typed `DelegationContext`. No parsing of `spawn_agent`,
  wait/result prose, filenames, timing, or parent messages is permitted.
- Validation and semantic interpretation of `DelegationContext` internals remain analyzer-owned.
  Sentinel may consume/project typed delegation facts and reject only already-defined whole-contract
  gaps: missing/null serialized `delegation` for v0.8, typed absence that contradicts v0.8, or a
  conflicting schema/same-session contract state. Analyzer-declared
  `DelegationTopology::MixedOrAmbiguous` and `ChildWorkVisibility::{Partial, Opaque}` states remain
  valid typed facts and must not be revalidated or reinterpreted field by field.
- Raw conflicting delegation links remain analyzer-owned. The analyzer maps them to its typed
  `DelegationContext` projection, including `DelegationTopology::MixedOrAmbiguous` and
  `ChildWorkVisibility::Opaque`; sentinel consumes that projection without interpreting raw conflict.
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

The interpretation-to-presentation core beginning with a completed `CheckpointInterpretation` must
not inspect schema strings, classify analyzer state, apply legacy evidence prefixes, or infer
delegation. Contract validation, typed interpretation, posture/evidence/delegation projection,
scheduling, and `CheckpointPresentation`/`ReplayReport`/`LiveObservation` construction must contain
zero R8-4 schema-presence predicates. The central validator's typed compatibility matrix remains
outside this presentation rule.

`CheckpointInterpretation.delegation: Option<DelegationContext>` is the only typed delegation
presence projection used by core interpretation/presentation. It is not currently representable by
`CheckpointPresentation.checkpoint.delegation` alone. Analyzer `Checkpoint.delegation` is a
non-optional `DelegationContext`; `RawCheckpoint.delegation` is optional only during deserialization,
and pre-v0.8 absence becomes `DelegationContext::default()` in the resulting `Checkpoint`. Public
`CheckpointPresentation` carries that `Checkpoint` but no separate presence bit or optional
delegation projection. Therefore a pre-v0.8 absent value and a typed default value are
indistinguishable at that public rendering boundary. With the public shape unchanged, placing the
presentation in `ReplayReport` or `LiveObservation` does not preserve
`CheckpointInterpretation.delegation: Option<_>` for a later renderer; Option B must not claim that
the internal `Option` reaches final console rendering.

Recorded decision `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` preserves the existing public shape
and output by allowing exactly one direct field use and one equality predicate,
`self.checkpoint.schema_version == "v0.8"`, inside the named legacy public facade
`CheckpointPresentation::render_console_block`. That frozen facade may serve the exact allowed
owner set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` with exact
raw direct `render_console_block` call-expression count `4`: two replay, one live, and one
adjudication. The AST/call-path proof fails on any fifth call, including a new call inside an allowed
owner.
Every other item in `operator_surface.rs`, including `present_interpretation` and its formatting
helpers, has zero `schema_version` uses/predicates. This accepts a localized
compatibility/presentation schema-coupling and centralization exception plus exact facade/core and
three-consumer call-path parity cost. It is not an adjudication exception: `adjudication.rs` stays
unchanged and read-only, adjudication policy/request-shaping logic and decision semantics do not
change. On the adjudication path, the predicate may affect only preserved rendered
`operator_summary` content. The operator selected B rather than Option A's public-field/source-
compatibility cost and also accepted the R8-4 dev-only AST parser/lockfile cost. That decision does
not alter the analyzer contract. The review-clean R8-4 series implemented and locked that decision;
changing the upstream analyzer `Checkpoint` schema remains outside R8.
`format_delegation_summary`, `format_delegation_topology`, and
`format_child_work_visibility` remain formatting-only and do not validate analyzer-owned facts.

## Unchanged Scheduling, Adjudication, and Delivery

- `ReplayScheduler::observe`, `SchedulerPolicy`, trigger meanings, cooldown/deduplication, and
  scheduler state are unchanged.
- Adjudication request shaping, fallback, response handling, operator notes, request eligibility,
  and decision semantics are unchanged. Under Option B, only the already-rendered content preserved
  in `operator_summary` may reflect the facade's sole presence predicate; full request parity and
  exact `operator_summary` byte parity are required, and `adjudication.rs` remains unedited.
- `LiveSessionCoordinator::poll_once` retains append-only rollout checks, sparse-startup behavior,
  verified-closure enforcement, per-session cursor freshness, and delivery ordering.
- Interpretation occurs before presentation and before a cursor is recorded/persisted as delivered.
  Replay first validates/interprets the complete selected checkpoint set and only then constructs
  scheduler/report state. Live validates/interprets the event before mutating accepted-checkpoint or
  cursor state. A failed interpretation produces no scheduler decision, presentation, adjudication,
  operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance.
- Existing real-session transport-observation bookkeeping before `runtime.observe` is explicitly
  preserved and may occur: `monitor_linked_closure` tracking, `begin_poll` pending-poll state, and
  `checkpoint_ready_event` emission-ordinal allocation. R8 does not roll those observations back and
  authorizes no production edit to `real_session_live.rs`. The protected failure proof is exactly:
  after interpretation fails, there is no scheduler decision, presentation, adjudication, operator
  sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance.

No edit to `scheduler.rs`, `adjudication.rs`, or their tests belongs in R8 unless a later separately
reviewed spec authorizes it.

## Failure Semantics

1. Unsupported schema; missing/malformed required serialized fields; an empty `session_id`,
   `checkpoint_id`, `task_frame.objective`, or `expected_next_step`; or invalid same-session
   assumptions fail closed with `CheckpointContractError`. No other typed text/identifier field is
   sentinel-owned validation.
2. Replay maps each `CheckpointContractError` deterministically to `InputError` with retained
   artifact path/line when the error came from serialized input and checkpoint/schema/field detail
   for typed interpretation. `execute` then uses the existing `From<InputError>` path to return
   `SentinelError::Input`; no new top-level `SentinelError` category is introduced.
3. Live fixture/adapter and runtime paths map each `CheckpointContractError` deterministically to
   `LiveInputError`, retaining fixture path/line or checkpoint/source detail as applicable. The
   existing `From<LiveInputError>` path returns `LiveRuntimeError::Input`; no parallel runtime
   contract-error category is introduced.
4. No v0.3-v0.8 failure falls back to legacy inference. No prior checkpoint from another session is
   consulted.
5. A missing/null v0.8 delegation whole-contract value fails contract validation; analyzer-owned
   fields inside a present `DelegationContext` are not field-level sentinel validation and never
   trigger raw-event reconstruction.
6. No fallible path uses `unwrap`, panic, default acceptance, or error-swallowing fallback. On any
   contract failure there is no scheduler decision, presentation, adjudication, operator sink
   emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance. Existing
   pre-observe monitor-closure tracking, pending-poll bookkeeping, and emission-ordinal allocation
   may already have occurred and are not rollback targets.

## Migration Contract

1. Add the central schema profile, raw validator, typed interpretation, deterministic error
   adapters, and the in-module `#[cfg(test)]` matrix in
   `src/checkpoint_interpretation.rs`; do not add an external interpretation integration target.
2. Add `try_render_replay_report`; validate/interpret the complete replay selection before any
   scheduler call, then render only typed interpretations. Route `execute` through this fallible core
   so `CheckpointContractError -> InputError -> SentinelError::Input`, without changing sorting,
   cursor filtering, report grouping, or the public `execute` signature.
3. Route fixture/live validation and `LiveRuntime::observe` through the same seam before any runtime
   mutation or scheduler call, so `CheckpointContractError -> LiveInputError ->
   LiveRuntimeError::Input`, while preserving `LiveCheckpointCompatibility` behavior.
4. Add `present_interpretation` as the validated-core typed adapter and
   `present_compatibility_interpretation` as the total-facade adapter over one private renderer with
   separately locked evidence-limit modes. Keep public `present_checkpoint*` and
   `render_replay_report` signatures/current total behavior as non-core compatibility facades backed
   by central `project_checkpoint_compatibility`, including explicit `CompatibilityOnly` treatment
   of unsupported typed facade input without v0.2 relabeling. Remove their independent
   version/analyzer semantics only after signature/behavior and replay/live parity tests pass.
5. Keep adjudication, sink emission, real-session delivery, acceptance, and persistence after
   successful interpretation, then run the full sentinel wall. Each migration packet must be
   independently review-clean before the next cutover.

## Exact Future Source Contract

The PLAN/TASKS may packetize only this inventory unless a fresh spec amendment expands it:

| Path | Permitted R8 change |
|---|---|
| `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` | Crate-private sole owner of schema profiles, serialized contract rules, typed interpretation, and shared posture/evidence/delegation normalization. R8-4 adds the total `project_checkpoint_compatibility` path and explicit `CompatibilityOnly` profile here; supported valid inputs share exact projection with the validated path, while unsupported typed facade input is never relabeled v0.2. Contains its own `#[cfg(test)]` matrix. |
| `crates/agent-drift-sentinel/src/lib.rs` | Minimal crate-private module wiring only; no public re-export or API expansion. |
| `crates/agent-drift-sentinel/src/input.rs` | Delegate raw replay validation/interpretation; map `CheckpointContractError` into `InputError`; retain bundle, sorting, cursor, and replay-specific errors. |
| `crates/agent-drift-sentinel/src/live_input.rs` | Delegate raw/typed compatibility; map `CheckpointContractError` into `LiveInputError`; retain event/source/sequence validation and compatibility facade. |
| `crates/agent-drift-sentinel/src/live_runtime.rs` | Consume the shared interpretation before state mutation and the unchanged scheduler/presentation sequence; propagate via `LiveRuntimeError::Input`. |
| `crates/agent-drift-sentinel/src/real_session_live.rs` | No production edit authorized. Preserve pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping and prove the narrower post-interpretation boundary from tests/static ordering. |
| `crates/agent-drift-sentinel/src/operator_surface.rs` | Add the fallible internal replay report path and typed renderer; preserve exact public facade signatures/current behavior while removing duplicated semantic/version decisions. |
| `crates/agent-drift-sentinel/Cargo.toml` | R8-4 only: add dev-only `syn = { version = "2", features = ["full", "visit"] }` for the AST ownership/predicate test after the presence decision and HIGH-impact gate authorize R8-4; no runtime dependency. |
| `Cargo.lock` | R8-4 only: add `syn` to the `agent-drift-sentinel` dependency list while retaining the already locked transitive `syn` `2.0.117`; any additional resolver/package change is a stop. |

Recorded decision `R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A` makes the exact R8-4 code
manifest these six files only: `crates/agent-drift-sentinel/Cargo.toml`, `Cargo.lock`,
`crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`,
`crates/agent-drift-sentinel/src/operator_surface.rs`,
`crates/agent-drift-sentinel/tests/operator_surface.rs`, and
`crates/agent-drift-sentinel/tests/live_end_to_end.rs`. In particular, the already-review-clean
`input.rs`, `live_input.rs`, and `live_runtime.rs` fallible call sites remain unedited.

Before R8-4 code, run upstream GitNexus impact for each existing central symbol actually edited.
The minimum expected set is `CheckpointInterpretation`,
`CheckpointSchemaVersion::from_literal`, `interpret_checkpoint`, `checkpoint_posture`, and
`checkpoint_evidence`, in addition to the operator-surface inventory in the PLAN/TASKS. New
`CheckpointCompatibilityProjectionInput`, `CheckpointProjectionProfile`,
`project_checkpoint_compatibility`, and any new private shared-projection helper have no
pre-existing graph target. A HIGH/CRITICAL result not already covered by the recorded R8-4
decisions requires its own structured gate before the owning edit.

Outside that edit inventory, `crates/agent-drift-sentinel/src/adjudication.rs` is read-only R8-4
dependency/call-path evidence only. Preserve `shape_request`, including request eligibility,
existing truncation, all fields, request-shaping logic, and decision semantics; it is not an edit-
manifest file.

No analyzer, compactor, schema, fixture corpus, scheduler, adjudication, CLI, or operator-sink
production edit is authorized by this spec.

## Exact Future Test Contract

| Path | Required proof |
|---|---|
| `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` in-module `#[cfg(test)]` matrix | Preserve the existing exact v0.2-v0.8 validation/interpretation matrix. Add exact equality of all renderer-consumed facts between `interpret_checkpoint` and `project_checkpoint_compatibility` for valid typed v0.2-v0.8 inputs; explicit `CompatibilityOnly` selection and legacy-compatible posture/evidence with absent typed delegation for unsupported typed facade input; proof that unsupported input is not `Schema(V0_2)`; total projection for typed supported shapes the public facades currently accept without validation; and negative proof that core `interpret_checkpoint` still rejects unsupported schema, required-field/explicit-state gaps, and cross-session history without selecting `CompatibilityOnly`. The helper must not call/catch the fallible entry point. |
| `crates/agent-drift-sentinel/tests/replay_input.rs` | Replay raw-field/version behavior, sorting, mixed-version failure, and cursor behavior unchanged. |
| `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs` | v0.2 and v0.3-v0.8 compatibility/presentation behavior unchanged, including v0.8 state-backed evidence/delegation. |
| `crates/agent-drift-sentinel/tests/live_input.rs` and `tests/live_input_adapter.rs` | Append-only event/cursor and fixture adapter errors unchanged. |
| `crates/agent-drift-sentinel/tests/live_runtime.rs` | `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`; shared interpretation precedes unchanged scheduling/state mutation; no decision/presentation/acceptance/cursor advance on failure; repeated-failure trigger remains distinct from posture. |
| `crates/agent-drift-sentinel/tests/real_session_live.rs` | Per-session cursors, verified closure, sparse startup, restart, and regression failures remain unchanged; static/behavior proof permits existing pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping while proving no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance after interpretation fails. |
| `crates/agent-drift-sentinel/tests/operator_surface.rs` | Compile-time function-pointer assertions lock the exact public functions/method signatures and Option B's unchanged public struct shape. Behavior locks cover exact current bytes for supported v0.2-v0.8 and unsupported typed facade input after delegation to the total central projection. Parse production source with `syn::parse_file`; a `Visit`-based, fail-closed ownership/data-dependency pass visits every `schema_version` field/path occurrence and every enclosing predicate, including binary `==`/`!=`, boolean nesting, `if`/`while`, `match` scrutinee/guards, parsed `matches!`, method-call, struct field/pattern, constant, and local-alias forms, and attributes each occurrence/predicate to its owning Rust item. Recorded Option B requires whole-file exactly one direct field use and one predicate, both in `CheckpointPresentation::render_console_block`, zero elsewhere, and the allowed predicate must be direct equality to literal `"v0.8"`. The same fail-closed production call-path inventory independently asserts exact owner-set equality `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` and exact raw direct `render_console_block` call-expression count `4`, partitioned as two replay, one live, and one adjudication. Any fifth direct call fails even inside an allowed owner; any unclassified occurrence, call owner, or macro form also fails. AST proof establishes syntax/ownership/count and call ownership only; it does not claim runtime data flow or behavior. |
| `crates/agent-drift-sentinel/tests/live_end_to_end.rs` | Replay/live/adjudication parity for diagnostics, headlines, turn context, archetype, progress, posture, session locality, and trigger/posture separation. Recorded Option B proves legacy v0.2-v0.7/v0.8 facade output matches intended absence/presence output and separately exercises all three production consumers: `ReplayReport::to_console_text`, final live console rendering through `cli::run_live`'s call shape, and `adjudication::shape_request` `operator_summary` construction. Behavior witnesses lock exact pre/post `operator_summary` bytes and full `AdjudicationRequest` equality, proving no request eligibility, shaping, or decision-semantic change. Earlier core interpretation/projection/scheduling/construction remains predicate-free. These behavior tests, not the AST test, prove output and call-path parity. |

Additionally, a compile-time function-pointer assertion must lock
`execute: fn(&SentinelRequest) -> Result<SentinelResult, SentinelError>`. Replay failure tests must
assert `CheckpointContractError -> InputError -> SentinelError::Input` and zero scheduler/report,
presentation, adjudication, or cursor effects. Live failure tests must assert the corresponding
`CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input` chain and the narrower
protected zero-effect boundary; pre-observe transport bookkeeping is permitted. Static source checks
must prove core replay/live call validation/interpretation before
scheduler/presentation and that the compatibility facades are not called from `execute` or
`LiveRuntime::observe`.

Required verification wall after implementation: focused changed targets, then
`cargo test -p agent-drift-sentinel -- --nocapture`, workspace formatting/clippy/tests required by
repository policy, staged GitNexus change detection, and cached-diff inspection. Counts must be
recorded from the implementation run. At `ad6340190`, Sentinel passes `233 / 233` and workspace
tests pass `2,657` with zero failed and `2` ignored. Final source evidence also confirms that
`CheckpointSchemaVersion::from_literal` stayed byte-identical to `ec2c5da7d` (`698` bytes; SHA-256
`4886d6b36e75dcc2df71d5616618e175ee7a6886b1812f85370191435200cbe5`). Validated replay/live
retain common flattened-core zero-limit behavior, while compatibility facades preserve the pre-R8
grouped-stop zero-limit contract; both parity surfaces are locked by review-clean tests.

## Success Criteria

| Gate | Testable criterion |
|---|---|
| `CTX-R8-01` | R7 analyzer/delegation output remains the sole semantic input; no analyzer contract change is required. |
| `CTX-R8-02` | MAP/SPEC/PLAN/TASKS are fresh-review-clean before the first source/test edit. |
| `CTX-R8-03` | Replay and live call one fallible `interpret_checkpoint` seam before scheduling/presentation; error tests prove `CheckpointContractError -> InputError -> SentinelError::Input` and `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`. |
| `CTX-R8-04` | One literal compatibility matrix covers v0.2 and every version v0.3-v0.8, including fail-closed serialized gaps and exactly the four sentinel-owned typed non-empty fields. |
| `CTX-R8-05` | Recorded `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` authorizes the unchanged public shape plus dev-only parser cost. It requires exactly one direct field use and one literal-v0.8 predicate in `CheckpointPresentation::render_console_block`, zero elsewhere, and accepts the localized compatibility/presentation coupling and parity cost because unchanged public shapes erase internal presence before later rendering. AST/call-path tests prove exhaustive syntax/ownership policy, exact allowed owner-set equality `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`, and exact raw direct `render_console_block` call-expression count `4` — two replay, one live, and one adjudication — with any fifth call failing even within an allowed owner. Separate behavior tests prove facade/core output, all three consumer paths, and unchanged `operator_summary` bytes/full adjudication requests. `adjudication.rs` remains read-only; policy, request-shaping logic, and decision semantics stay unchanged. |
| `CTX-R8-06` | Failure produces no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance; pre-observe transport bookkeeping is permitted; success preserves existing scheduler/adjudication outputs, real-session closure, delivery order, and per-session cursor behavior. |

`CTX-R8-01..06` are `PROVEN`. The review-clean authority family through `806e53740`, the
review-clean implementation/proof series, the green R8-6 wall, and fresh independent built-in
`default` `CLEAN` review of five-doc receipt `549160ebc1d93b26fdbe8203d748dfb1a64787ef` satisfy the
R8-IMPLEMENT exit gate. R8-SPEC and R8-IMPLEMENT are complete; the R8 family is terminally complete
with active phase `none` and active packet `none`. Terminal transition commit
`65eac5ab2ecf354a731d127014f91da5c87e6e8d` received fresh independent built-in `default` `CLEAN`
with no findings, so the terminal R8 transition and R8-IMPLEMENT exit gate are review-clean. This
review receipt assigns itself no commit hash or review result, remains pending its own fresh
independent review, and starts no further phase work. **NO NEXT ELIGIBLE PHASE IS DEFINED IN THIS
CONTROL PACK.**
