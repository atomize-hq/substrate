# R8 Spec: Agent Drift Sentinel Interpretation Consolidation

Status: **R8-SPEC COMPLETE / COMPLETE AUTHORITY FAMILY THROUGH `806e53740` FRESH INDEPENDENT BUILT-IN
`default` `CLEAN` / `CTX-R8-01` AND `CTX-R8-02` PROVEN / R8-IMPLEMENT SOLE ACTIVE PHASE AT ENTRY
ONLY / ACTIVE PACKET `none` / TRANSITION UPDATE AWAITING FRESH INDEPENDENT REVIEW**.

R8-SPEC is `COMPLETE`. The complete R8 authority-family authoring/review-fix series
`698c766f9` + `f5865fb7` + `95529809` + `0ed3d8f04` + `cfcf65507` + `2b9565fb9` +
`b04207fb6` + `b9ce44c6f` + `904c93d0d` + `67c81c6ff` + `24e649de6` + `099f4ec2c` +
`806e53740` is landed and received fresh independent built-in `default` `CLEAN` with no findings. `CTX-R8-01`
is `PROVEN`, and `CTX-R8-02` is `PROVEN` / `SATISFIED`. R8-IMPLEMENT is the sole `ACTIVE` phase at
`ENTRY ONLY` with active packet `none`; its entry gate is satisfied by the review-clean R8
MAP/SPEC/PLAN/TASKS, but every R8 implementation task and checkbox remains unchecked and
unstarted, and no R8 source or test work has begun. `CTX-R8-03` is `OPEN` / current at entry;
`CTX-R8-04` through `CTX-R8-06` remain `BLOCKED` / `UNPROVEN` in dependency order. The four future
HIGH symbol-decision gates `R8-2-HIGH-IMPACT-REPLAY-LOADER-01`,
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01`, `R8-3-HIGH-IMPACT-LIVE-RUNTIME-01`, and
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01`, plus unresolved
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01`, remain pending packet-local prerequisites; they
authorize no edits and do not invalidate R8-IMPLEMENT entry. Prompt 1 selectors
`PHASE_ID: R8-IMPLEMENT` / `ACTIVE_PACKET: none` are prepared and eligible but `UNINVOKED`. This
narrow phase-transition update is `AWAITING FRESH INDEPENDENT REVIEW`; it assigns itself no commit
hash or review result, does not claim to be clean, and starts no implementation. This document
specifies the entered implementation phase; entry alone authorizes no symbol edit before the
owning packet's decision, impact, TDD, proof, commit, and fresh-review gates are satisfied.

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
analyzer-state, evidence, or delegation semantics except for the single presence-only legacy
predicate authorized if Option B is selected below. Under Option B, the unchanged
`CheckpointPresentation::render_console_block` facade may serve the exact allowed downstream
production owner set `{ReplayReport::to_console_text, cli::run_live,
adjudication::shape_request}` after the unchanged public shapes erase the internal presence bit.
Fail-closed AST/call-path proof must independently assert both that exact owner-set equality and
exact raw direct `render_console_block` call-expression count `4`: two replay calls in
`ReplayReport::to_console_text`, one live call in `cli::run_live`, and one adjudication call in
`adjudication::shape_request` while constructing `operator_summary`. Any fifth direct call fails,
even if added within an allowed owner. None of these calls is a validation boundary. `execute` calls the additive
fallible replay core and preserves its existing result signature. No facade or core entry point may
use `unwrap`, panic, or error-swallowing/legacy fallback to cross the fallible seam.

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

The future operator gate `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` must choose the public-boundary
representation before any R8-4 symbol edit. Option A adds an explicit optional presence/projection
field to public `CheckpointPresentation`, accepting the public-struct/source-compatibility cost and
locking the new shape with exact construction/render/parity tests; Option A requires zero schema-
version field/path uses and predicates throughout `operator_surface.rs`. Option B preserves the
existing public shape and output by allowing exactly one direct field use and one equality
predicate,
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
`operator_summary` content. Option B is recommended because it best preserves the stable public API
while keeping all earlier semantics centralized, but this specification neither resolves nor
authorizes the gate. Option A has no presentation predicate and must preserve the same public
output. Both options also accept the future R8-4 dev-only AST parser cost described below. Because
no production symbol or manifest was edited during R8-SPEC, the future gate did not block docs
completion or fresh review of this family and does not invalidate R8-IMPLEMENT entry. Changing the upstream analyzer `Checkpoint` schema is
outside R8.
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
| `crates/agent-drift-sentinel/Cargo.toml` | R8-4 only: add dev-only `syn = { version = "2", features = ["full", "visit"] }` for the AST ownership/predicate test after the presence decision and HIGH-impact gate authorize R8-4; no runtime dependency. |
| `Cargo.lock` | R8-4 only: add `syn` to the `agent-drift-sentinel` dependency list while retaining the already locked transitive `syn` `2.0.117`; any additional resolver/package change is a stop. |

Outside that edit inventory, `crates/agent-drift-sentinel/src/adjudication.rs` is read-only R8-4
dependency/call-path evidence only. Preserve `shape_request`, including request eligibility,
existing truncation, all fields, request-shaping logic, and decision semantics; it is not an edit-
manifest file.

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
| `crates/agent-drift-sentinel/tests/operator_surface.rs` | Compile-time function-pointer assertions lock the exact public functions/method signatures; decision-locked construction tests cover either Option A's explicit optional field or Option B's unchanged public struct shape; v0.2-v0.7 absence/default and v0.8 presence render exactly as the selected compatibility contract requires. Parse production source with `syn::parse_file`; a `Visit`-based, fail-closed ownership/data-dependency pass visits every `schema_version` field/path occurrence and every enclosing predicate, including binary `==`/`!=`, boolean nesting, `if`/`while`, `match` scrutinee/guards, parsed `matches!`, method-call, struct field/pattern, constant, and local-alias forms, and attributes each occurrence/predicate to its owning Rust item. Option A requires zero uses and predicates in presentation. Option B requires whole-file exactly one direct field use and one predicate, both in `CheckpointPresentation::render_console_block`, zero elsewhere, and the allowed predicate must be direct equality to literal `"v0.8"`. The same fail-closed production call-path inventory independently asserts exact owner-set equality `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` and exact raw direct `render_console_block` call-expression count `4`, partitioned as two replay, one live, and one adjudication. Any fifth direct call fails even inside an allowed owner; any unclassified occurrence, call owner, or macro form also fails. AST proof establishes syntax/ownership/count and call ownership only; it does not claim runtime data flow or behavior. |
| `crates/agent-drift-sentinel/tests/live_end_to_end.rs` | Replay/live/adjudication parity for diagnostics, headlines, turn context, archetype, progress, posture, session locality, and trigger/posture separation. Option A proves the public `None`/`Some` carrier survives to final rendering with no presentation predicate and the same public output. Option B proves legacy v0.2-v0.7/v0.8 facade output matches intended absence/presence output and separately exercises all three production consumers: `ReplayReport::to_console_text`, final live console rendering through `cli::run_live`'s call shape, and `adjudication::shape_request` `operator_summary` construction. Behavior witnesses lock exact pre/post `operator_summary` bytes and full `AdjudicationRequest` equality, proving no request eligibility, shaping, or decision-semantic change. Earlier core interpretation/projection/scheduling/construction remains predicate-free. These behavior tests, not the AST test, prove output and call-path parity. |

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
| `CTX-R8-05` | `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` is explicitly decided before R8-4 edits and authorizes the selected public-compatibility plus dev-only parser cost. Option A proves zero presentation `schema_version` uses/predicates, accepts the public-field/source-compatibility cost, and preserves the same public output. Option B proves exactly one direct field use and one literal-v0.8 predicate in `CheckpointPresentation::render_console_block`, zero elsewhere, and accepts the localized compatibility/presentation coupling and parity cost because unchanged public shapes erase internal presence before later rendering. AST/call-path tests prove exhaustive syntax/ownership policy, exact allowed owner-set equality `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`, and exact raw direct `render_console_block` call-expression count `4` — two replay, one live, and one adjudication — with any fifth call failing even within an allowed owner. Separate behavior tests prove facade/core output, all three consumer paths, and unchanged `operator_summary` bytes/full adjudication requests. `adjudication.rs` remains read-only; policy, request-shaping logic, and decision semantics stay unchanged. |
| `CTX-R8-06` | Failure produces no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance; pre-observe transport bookkeeping is permitted; success preserves existing scheduler/adjudication outputs, real-session closure, delivery order, and per-session cursor behavior. |

`CTX-R8-01` is `PROVEN`, and `CTX-R8-02` is `PROVEN` / `SATISFIED` by the complete R8 authority
family through `806e53740` receiving fresh independent built-in `default` `CLEAN` with no findings.
`CTX-R8-03` is `OPEN` / current at R8-IMPLEMENT entry; `CTX-R8-04..06` remain `BLOCKED` /
`UNPROVEN` in dependency order. All implementation tasks remain unchecked and unstarted. The four
future HIGH gates and unresolved `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` remain packet-local
prerequisites and authorize no edit.
