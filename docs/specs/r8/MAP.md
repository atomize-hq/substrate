# R8 Map: Sentinel Interpretation Consolidation / Integration

Status: **R8-IMPLEMENT ACTIVE / R8-1 `ec2c5da7d` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` /
R8-2 `08e0d0e2a` + `5669e1f6e` + `911dd49b` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` /
R8-3 `963a8202` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` / R8-4 DOCS-ONLY AUTHORITY
AMENDMENT `849393029` LANDED / FRESH INDEPENDENT REVIEW PENDING / NO R8-4 SOURCE OR TEST EDITS**.

R8-SPEC is `COMPLETE`, and the R8-SPEC -> R8-IMPLEMENT transition/entry gate remains review-clean.
R8-1 commit `ec2c5da7d`, R8-2 series `08e0d0e2a` + `5669e1f6e` + `911dd49b`, and R8-3 commit
`963a8202` each received fresh independent built-in `default` `CLEAN` with no actionable findings.
R8-IMPLEMENT remains the sole active phase. R8-4 is blocked only on fresh independent review of
landed bounded Markdown-only authority amendment `849393029`; no R8-4 source or test edit has
started.
The operator resolved every current R8-4 gate as
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`, and
`R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`. The last decision expands the R8-4 exact code
manifest from five files to six by adding
`crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`. Landed amendment candidate
`849393029` is pending fresh independent review, has no review result, makes no `CLEAN` claim, and
authorizes no R8-4 code until it is fresh-review-clean.

## Objective

Make replay and live sentinel paths consume one typed checkpoint interpretation while preserving
the analyzer as the owner of drift, progress, and delegation semantics. Centralize supported-schema
compatibility, then keep the operator surface a renderer of typed facts rather than a second
analyzer.

Detailed contract: [`agent-drift-sentinel-interpretation-consolidation-r8-spec.md`](agent-drift-sentinel-interpretation-consolidation-r8-spec.md).

## Authority Boundary

- R7 owns analyzer-exported checkpoint semantics, including typed `DelegationContext` in v0.8.
- R8 owns sentinel validation, compatibility normalization, and replay/live consumption of those
  exported facts.
- Parent orchestration is evidence of parent activity only. It never proves child implementation,
  drift, progress, or completion.
- Scheduling cadence/trigger decisions and adjudication request/response behavior do not change.
- R8 adds no drift class, scorer, linkage heuristic, scheduler policy, or adjudication policy.

## Current Topology

| Path | Current owners | Duplication or boundary |
|---|---|---|
| Replay | `input.rs` -> `operator_surface.rs` | Raw schema checks live in replay input; version/posture/evidence interpretation lives in presentation. |
| Fixture/live | `live_input.rs` -> `live_runtime.rs` -> `operator_surface.rs` | Raw schema checks repeat replay rules; typed compatibility and presentation interpretation are separate. |
| Real-session live | `real_session_live.rs` -> `live_runtime.rs` | Pipeline, per-session freshness, delivery, and persisted cursors surround the same presentation path. |
| Operator | `operator_surface.rs` | Diagnostics rendering is shared, but version-aware posture/evidence decisions remain embedded in presentation. |
| Adjudication summary | `adjudication.rs` -> `CheckpointPresentation::render_console_block` | `shape_request` preserves the rendered block, subject only to the existing truncation limit, as `operator_summary`; this is read-only dependency/call-path evidence and not an authorized edit seam. |

GitNexus joins the replay path at `load_replay_bundle -> render_replay_report ->
present_checkpoint_with_previous`, and the live path at `poll_once -> LiveRuntime::observe ->
verify_live_checkpoint_compatibility -> present_checkpoint_with_previous`.

## Target Topology

Both adapters must call one origin-neutral, fallible typed seam:

```text
serialized checkpoint -> centralized contract validation -> CheckpointInterpretation
                                                           |-> scheduler inputs
                                                           |-> presentation-only rendering
replay previous-by-session -------------------------------/
live previous-by-session --------------------------------/
```

The interpretation result carries the validated schema profile, cursor, warning fingerprint,
flagged/max-score inputs, normalized posture/evidence, and
`delegation: Option<DelegationContext>`. Replay and
live supply only the current checkpoint and the previous checkpoint from the same session. They do
not supply origin-specific semantic switches.

The new module and its needed items remain crate-private, with its v0.2-v0.8 RED/GREEN matrix under
an in-module `#[cfg(test)]`; R8 adds no public re-export/API and no external interpretation test
target. Existing public-flow integration tests begin at R8-2/
R8-3 and exercise public facades.

Validation and interpretation complete before either core path calls the scheduler or presentation.
Replay validates/interprets the entire selected set before constructing any report decision; live
validates/interprets one event before mutating accepted-checkpoint/runtime state. Existing
real-session monitor-closure tracking, pending-poll bookkeeping, and emission-ordinal allocation may
already occur before `runtime.observe` and are preserved/out of scope. After interpretation fails,
there is no scheduler decision, presentation, adjudication, operator sink emission,
`record_delivery`, persisted cursor/delivery, or checkpoint acceptance; no rollback production edit
is authorized.

## Fallible Core And Compatibility Facades

- `interpret_checkpoint` is the shared fallible semantic entry point. Replay adds an internal
  fallible report entry point; live keeps the existing fallible `LiveRuntime::observe` entry point
  and calls interpretation before state mutation or scheduling.
- Replay maps `CheckpointContractError` deterministically into `InputError`, then through the
  existing `SentinelError::Input` variant returned by `execute`. Live maps the same central error
  into `LiveInputError`, then through the existing `LiveRuntimeError::Input` conversion.
- The current public signatures of `present_checkpoint`, `present_checkpoint_with_previous`,
  `render_replay_report`, and `execute` remain source-compatible. The first three infallible APIs are
  compatibility facades with their current behavior; core replay/live do not use them as validation
  boundaries.
- Compatibility facades delegate to a crate-private, total, non-validating compatibility projection
  owned by `checkpoint_interpretation.rs`, then feed the returned `CheckpointInterpretation` to the
  same `present_interpretation` renderer used by validated replay/live interpretations. The central
  module owns one shared posture/evidence/delegation normalization implementation. Its explicit
  `CheckpointProjectionProfile` distinguishes supported `Schema(CheckpointSchemaVersion)` input
  from `CompatibilityOnly`; an unsupported typed facade input selects `CompatibilityOnly` and
  preserves the current total facade behavior without being relabeled as v0.2. The compatibility
  entry point performs no serialized, typed, or history validation and does not call or catch
  `interpret_checkpoint`. Neither core paths nor facades may bridge errors with `unwrap`, panic,
  error swallowing, default-acceptance fallback, or v0.2 inference after a v0.3-v0.8 failure. Core
  replay/live remain on the unchanged fallible `interpret_checkpoint` contract and cannot select
  `CompatibilityOnly`.
- Delegation presence cannot be recovered from `CheckpointPresentation.checkpoint.delegation`
  alone: analyzer `Checkpoint.delegation` is non-optional, pre-v0.8 serialized absence becomes
  `DelegationContext::default()` during deserialization, and public `CheckpointPresentation` has no
  separate presence carrier. Under an unchanged public shape, construction of
  `CheckpointPresentation`, storage in `ReplayReport`, and storage in `LiveObservation` therefore
  erase the internal `Option` presence before the later console-rendering call. The resolved gate
  `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` preserves the public shape and permits exactly one
  `self.checkpoint.schema_version == "v0.8"` predicate in the named legacy public facade
  `CheckpointPresentation::render_console_block`. Option B explicitly permits that unchanged facade
  to serve the exact allowed downstream production owner set
  `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. Fail-closed
  AST/call-path proof must independently assert both that exact owner-set equality and exact raw
  direct `render_console_block` call-expression count `4`: two in
  `ReplayReport::to_console_text`, one in `cli::run_live`, and one in
  `adjudication::shape_request` while constructing `operator_summary`. Any fifth direct call fails,
  even when added inside one of those three allowed owners. Contract validation, typed interpretation,
  posture/evidence/delegation projection, scheduling, presentation construction, and adjudication
  decision/request-shaping logic must contain zero R8-4 schema-presence predicates. On the
  adjudication path, the sole predicate may affect only the preserved bytes rendered into the
  operator summary; it may not affect request eligibility, fields other than rendered
  `operator_summary` content, or decision semantics. `adjudication.rs` remains read-only evidence
  and is not an R8-4 edit-manifest file.
  This is a localized compatibility/presentation exception with an explicit parity-test cost, not
  a claim that the erased internal `Option` reaches later rendering and not an adjudication-policy
  exception. The operator rejected Option A's public-field/source-compatibility cost by selecting B.
  R8-4 still may not edit any source/test symbol until this six-file manifest amendment is committed
  and fresh-review-clean. Changing the upstream analyzer `Checkpoint` schema is outside R8.

## Compatibility Contract

- v0.2 retains its bounded legacy posture/evidence behavior.
- v0.3-v0.8 use explicit analyzer `DriftState`; malformed explicit-state checkpoints fail closed
  rather than falling back to v0.2 inference.
- v0.4 requires `turn_context`; v0.5 adds `session_archetype`; v0.6-v0.8 add
  `session_progress`; v0.8 adds serialized analyzer-owned `delegation`.
- Supported versions remain the exact literal set v0.2 through v0.8. Generalized version parsing is
  outside R8.
- Sentinel-owned typed non-empty validation is limited to exactly `session_id`, `checkpoint_id`,
  `task_frame.objective`, and `expected_next_step`, in addition to the existing serialized
  version/required-shape rules.
- Validation and semantic interpretation of fields inside analyzer-owned `DelegationContext` remain
  analyzer responsibilities. Sentinel may consume/project the typed delegation facts and reject only
  the already-defined whole-checkpoint gaps: a missing/null v0.8 `delegation` value or a conflicting
  schema/same-session contract state. Analyzer-declared `DelegationTopology::MixedOrAmbiguous` and
  `ChildWorkVisibility::{Partial, Opaque}` states are valid typed input, not a prompt for sentinel
  field-level revalidation or inference.

## R8-4 Amended Boundary

Decision `R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A` expands only R8-4's code manifest from five
files to these exact six:

- `crates/agent-drift-sentinel/Cargo.toml`;
- `Cargo.lock`;
- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`;
- `crates/agent-drift-sentinel/src/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`.

`input.rs`, `live_input.rs`, and `live_runtime.rs` remain outside R8-4: their review-clean fallible
calls do not change. The central module adds the total compatibility projection and explicit
projection profile, while `interpret_checkpoint` continues to validate and fail closed before
calling the same private normalization. The new projection profile/helper and any new private
normalization adapter have no pre-existing GitNexus target. Before code, run upstream impact on each
existing central item actually edited, including at minimum `CheckpointInterpretation`,
`CheckpointSchemaVersion::from_literal`, `interpret_checkpoint`, `checkpoint_posture`, and
`checkpoint_evidence`; a HIGH/CRITICAL result not covered by the recorded decisions requires its
own gate.

Central in-module tests must prove exact supported v0.2-v0.8 parity between the validated and total
projection for valid typed inputs; explicit `CompatibilityOnly` behavior for unsupported typed
facade input without v0.2 relabeling; total behavior for typed shapes the public facades currently
accept; and unchanged fail-closed unsupported-schema, required-field, explicit-state, and
cross-session behavior in `interpret_checkpoint`. Edit-manifest integration tests lock the existing
public signatures and exact bytes for supported and unsupported typed facade inputs. None of those
tests may claim the compatibility projection validates input or converts a core error into success.

## Gate Map

| Gate | R8 exit condition |
|---|---|
| `CTX-R8-01` | The closed R7 analyzer/delegation contract remains the unchanged semantic input. |
| `CTX-R8-02` | R8 MAP/SPEC/PLAN/TASKS are internally consistent and fresh-review-clean before code. |
| `CTX-R8-03` | Replay and live use the same typed interpretation function and parity matrix. |
| `CTX-R8-04` | One compatibility owner preserves v0.2 and v0.3-v0.8 contract behavior. |
| `CTX-R8-05` | Recorded decision `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` requires exactly one direct field use and one equality predicate with literal `"v0.8"` in `CheckpointPresentation::render_console_block`, zero in every other item, and permits that unchanged facade to serve the exact allowed owner set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. A `syn`-based AST/call-path visitor independently proves that exact owner-set equality and exact raw direct `render_console_block` call-expression count `4` — two replay, one live, and one adjudication — and fails on any fifth call even within an allowed owner. Separate behavior tests prove facade/core output, the three actual consumer paths, and unchanged `operator_summary` bytes/full adjudication requests. On the adjudication path, the predicate reaches only preserved presentation content, never adjudication policy/request shaping or decision semantics; `adjudication.rs` is read-only evidence, not an edit-manifest file. |
| `CTX-R8-06` | Existing pre-observe transport bookkeeping is permitted; after interpretation failure there is no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance. |

`CTX-R8-01` is `PROVEN`, and `CTX-R8-02` is `PROVEN` / `SATISFIED` by the complete review-clean
authority family through `806e53740`. `CTX-R8-03` is `OPEN` / current at R8-IMPLEMENT entry;
`CTX-R8-04..06` remain `BLOCKED` / `UNPROVEN` in dependency order. The control-pack ledger remains
authoritative until reviewed implementation evidence updates those rows.

## Required Docs Before Implementation

1. this MAP;
2. the R8 specification;
3. the exact migration/verification PLAN;
4. packetized TASKS with the recorded R8-2/R8-3 decisions, all five recorded R8-4 decisions, and
   the amended six-file R8-4 boundary.

All four artifacts are included in the complete authoring/review-fix series through `806e53740`,
which received fresh independent built-in `default` `CLEAN` with no findings. This satisfies
`CTX-R8-02` and the R8-IMPLEMENT entry gate. Later decisions and review-clean R8-1..R8-3 commits do
not authorize R8-4 code before this amendment itself is committed and fresh-review-clean.
