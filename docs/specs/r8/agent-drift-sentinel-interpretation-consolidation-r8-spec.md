# R8 Spec: Agent Drift Sentinel Interpretation Consolidation

Status: **R8-SPEC ACTIVE / IN PROGRESS; ACTIVE PACKET `none`; `CTX-R8-01` PROVEN; PLAN/TASKS
REVIEW PENDING; R8-IMPLEMENT BLOCKED/BOUNDARY-ONLY**.

R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. Fresh independent built-in `default` review of the complete family at
`0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five scoped documentation findings.
Bounded docs-only fix `2b9565fb9` landed and remains pending fresh independent re-review. The
current review/fix round identified three later scoped documentation findings; this bounded
Markdown-only fix addresses only those three and claims no review result. All R8 implementation
tasks remain unchecked and unstarted. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven;
`CTX-R8-03` through `CTX-R8-06` remain `BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and
no R8 code has started. No phase transition, Prompt 1 eligibility, implementation authorization,
complete-family `CLEAN`, or review result for this progress receipt is claimed. This document specifies a future
implementation; it does not authorize R8 code. The complete R8 MAP/SPEC/PLAN/TASKS family must be
fresh-review-clean before implementation starts.

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

pub(crate) struct CheckpointInterpretation {
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

pub(crate) fn validate_serialized_checkpoint(value: &serde_json::Value)
    -> Result<CheckpointSchemaVersion, CheckpointContractError>;

pub(crate) fn interpret_checkpoint(input: CheckpointInterpretationInput<'_>)
    -> Result<CheckpointInterpretation, CheckpointContractError>;

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

Names may change only in the reviewed PLAN; the invariants may not. `interpret_checkpoint` is the
single replay/live semantic entry point. It has no replay/live mode flag. The typed result contains
facts, not console strings or scheduler/adjudicator decisions. `try_render_replay_report` is the
fallible replay core used by `execute`; `LiveRuntime::observe` calls `interpret_checkpoint` directly.
Both validate/interpret before scheduler observation and before `present_interpretation`.

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
presentation/report APIs above remain compatibility facades with their current behavior, but they
are not core-path validation boundaries. They delegate to centralized non-validating compatibility
projection plus `present_interpretation`; they may not own or duplicate schema-version,
analyzer-state, evidence, or delegation semantics. `execute` calls the additive fallible replay core
and preserves its existing result signature. No facade or core entry point may use `unwrap`, panic,
or error-swallowing/legacy fallback to cross the fallible seam.

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

It must not inspect schema strings, classify analyzer state, apply legacy evidence prefixes, or
infer delegation. Replay and live render the same interpretation object for matching inputs.

`CheckpointInterpretation.delegation: Option<DelegationContext>` is the only typed delegation
presence projection used by core interpretation/presentation. It is not currently representable by
`CheckpointPresentation.checkpoint.delegation` alone. Analyzer `Checkpoint.delegation` is a
non-optional `DelegationContext`; `RawCheckpoint.delegation` is optional only during deserialization,
and pre-v0.8 absence becomes `DelegationContext::default()` in the resulting `Checkpoint`. Public
`CheckpointPresentation` carries that `Checkpoint` but no separate presence bit or optional
delegation projection. Therefore a pre-v0.8 absent value and a typed default value are
indistinguishable at that public rendering boundary.

The future operator gate `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` must choose the public-boundary
representation before any R8-4 symbol edit. Option A adds an explicit optional presence/projection
field to public `CheckpointPresentation`, accepting the public-struct/source-compatibility cost and
locking the new shape with exact construction/render/parity tests. Option B preserves the existing
public shape and output by allowing exactly one localized schema-presence check inside the legacy
compatibility facade, while core interpretation and core presentation use the internal optional
value; this accepts a localized exception/coupling cost and requires exact facade/core parity and
static-localization tests. Option B is recommended because it best preserves the stable public API
while keeping centralized core semantics, but this specification neither resolves nor authorizes
the gate. Because no production symbol is edited during R8-SPEC, the future gate does not block
docs completion or fresh review of this family. Changing the upstream analyzer `Checkpoint` schema
is outside R8.
`format_delegation_summary`, `format_delegation_topology`, and
`format_child_work_visibility` remain formatting-only and do not validate analyzer-owned facts.

## Unchanged Scheduling, Adjudication, and Delivery

- `ReplayScheduler::observe`, `SchedulerPolicy`, trigger meanings, cooldown/deduplication, and
  scheduler state are unchanged.
- Adjudication request shaping, fallback, response handling, and operator notes are unchanged.
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
4. Add `present_interpretation` as the core typed renderer. Keep public `present_checkpoint*` and
   `render_replay_report` signatures/current behavior as non-core compatibility facades backed by
   centralized non-validating projection; remove their independent version/analyzer semantics only
   after signature/behavior and replay/live parity tests pass.
5. Keep adjudication, sink emission, real-session delivery, acceptance, and persistence after
   successful interpretation, then run the full sentinel wall. Each migration packet must be
   independently review-clean before the next cutover.

## Exact Future Source Contract

The PLAN/TASKS may packetize only this inventory unless a fresh spec amendment expands it:

| Path | Permitted R8 change |
|---|---|
| `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` | New crate-private sole owner of schema profiles, serialized contract rules, typed interpretation, posture/evidence normalization, and v0.8 typed delegation projection; contains its own `#[cfg(test)]` matrix. |
| `crates/agent-drift-sentinel/src/lib.rs` | Minimal crate-private module wiring only; no public re-export or API expansion. |
| `crates/agent-drift-sentinel/src/input.rs` | Delegate raw replay validation/interpretation; map `CheckpointContractError` into `InputError`; retain bundle, sorting, cursor, and replay-specific errors. |
| `crates/agent-drift-sentinel/src/live_input.rs` | Delegate raw/typed compatibility; map `CheckpointContractError` into `LiveInputError`; retain event/source/sequence validation and compatibility facade. |
| `crates/agent-drift-sentinel/src/live_runtime.rs` | Consume the shared interpretation before state mutation and the unchanged scheduler/presentation sequence; propagate via `LiveRuntimeError::Input`. |
| `crates/agent-drift-sentinel/src/real_session_live.rs` | No production edit authorized. Preserve pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping and prove the narrower post-interpretation boundary from tests/static ordering. |
| `crates/agent-drift-sentinel/src/operator_surface.rs` | Add the fallible internal replay report path and typed renderer; preserve exact public facade signatures/current behavior while removing duplicated semantic/version decisions. |

No analyzer, compactor, schema, fixture corpus, scheduler, adjudication, CLI, or operator-sink
production edit is authorized by this spec.

## Exact Future Test Contract

| Path | Required proof |
|---|---|
| `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` in-module `#[cfg(test)]` matrix | Exact v0.2-v0.8 matrix; exact non-empty sentinel field set (`session_id`, `checkpoint_id`, `task_frame.objective`, `expected_next_step`); same-session history; explicit-state precedence; structured failures; typed v0.8 delegation projection; analyzer maps conflicting-link input to its typed `DelegationContext` projection and sentinel consumes the resulting `DelegationTopology::MixedOrAmbiguous` plus `ChildWorkVisibility::Opaque` without interpreting raw conflict; `ChildWorkVisibility::{Partial, Opaque}` accepted without field-level revalidation; parent-orchestration negative witness. |
| `crates/agent-drift-sentinel/tests/replay_input.rs` | Replay raw-field/version behavior, sorting, mixed-version failure, and cursor behavior unchanged. |
| `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs` | v0.2 and v0.3-v0.8 compatibility/presentation behavior unchanged, including v0.8 state-backed evidence/delegation. |
| `crates/agent-drift-sentinel/tests/live_input.rs` and `tests/live_input_adapter.rs` | Append-only event/cursor and fixture adapter errors unchanged. |
| `crates/agent-drift-sentinel/tests/live_runtime.rs` | `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`; shared interpretation precedes unchanged scheduling/state mutation; no decision/presentation/acceptance/cursor advance on failure; repeated-failure trigger remains distinct from posture. |
| `crates/agent-drift-sentinel/tests/real_session_live.rs` | Per-session cursors, verified closure, sparse startup, restart, and regression failures remain unchanged; static/behavior proof permits existing pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping while proving no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance after interpretation fails. |
| `crates/agent-drift-sentinel/tests/operator_surface.rs` | Compile-time function-pointer assertions lock the exact public functions/method signatures; decision-locked construction tests cover either Option A's explicit optional field or Option B's unchanged public struct shape; v0.2-v0.7 absence/default and v0.8 presence render exactly as the selected compatibility contract requires; core presentation consumes typed facts and owns no version/analyzer semantics. |
| `crates/agent-drift-sentinel/tests/live_end_to_end.rs` | Replay/live parity for diagnostics, headlines, turn context, archetype, progress, posture, session locality, and trigger/posture separation. |

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
recorded from the implementation run; this candidate spec claims none.

## Success Criteria

| Gate | Testable criterion |
|---|---|
| `CTX-R8-01` | R7 analyzer/delegation output remains the sole semantic input; no analyzer contract change is required. |
| `CTX-R8-02` | MAP/SPEC/PLAN/TASKS are fresh-review-clean before the first source/test edit. |
| `CTX-R8-03` | Replay and live call one fallible `interpret_checkpoint` seam before scheduling/presentation; error tests prove `CheckpointContractError -> InputError -> SentinelError::Input` and `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`. |
| `CTX-R8-04` | One literal compatibility matrix covers v0.2 and every version v0.3-v0.8, including fail-closed serialized gaps and exactly the four sentinel-owned typed non-empty fields. |
| `CTX-R8-05` | `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` is explicitly decided before R8-4 edits; exact compile-time shape/signature, v0.2-v0.7 absence/default, v0.8 presence, static-localization, facade/core, and replay/live parity tests prove the selected representation; core operator presentation has no schema/state/delegation inference. |
| `CTX-R8-06` | Failure produces no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance; pre-observe transport bookkeeping is permitted; success preserves existing scheduler/adjudication outputs, real-session closure, delivery order, and per-session cursor behavior. |

`CTX-R8-01` is proven. `CTX-R8-02` remains `OPEN` / `REVIEW PENDING` and not proven;
`CTX-R8-03..06` remain blocked. Fresh independent built-in `default` review of the complete family
through this bounded docs-only fix is the next gate after the earlier `0ed3d8f04` + `cfcf65507`
review returned `CHANGES_REQUIRED`. All implementation tasks remain unchecked and unstarted, and R8
implementation remains blocked. This progress receipt claims no review result for itself.
