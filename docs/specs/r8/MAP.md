# R8 Map: Sentinel Interpretation Consolidation / Integration

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
complete-family `CLEAN`, or review result for this progress receipt is claimed. **No R8 code may begin until the
complete MAP/SPEC/PLAN/TASKS family is fresh-review-clean.**

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
- Compatibility facades delegate to centralized, non-validating compatibility projection and typed
  rendering. They do not duplicate or own the supported-version table, analyzer-state mapping, or
  delegation semantics. Neither core paths nor facades may bridge errors with `unwrap`, panic,
  error-swallowing fallback, or v0.2 inference after a v0.3-v0.8 failure.
- Delegation presence cannot be recovered from `CheckpointPresentation.checkpoint.delegation`
  alone: analyzer `Checkpoint.delegation` is non-optional, pre-v0.8 serialized absence becomes
  `DelegationContext::default()` during deserialization, and public `CheckpointPresentation` has no
  separate presence carrier. The future gate
  `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` must choose either an explicit optional public
  presence/projection field or a localized schema-presence check in the legacy compatibility
  facade while core interpretation/presentation uses an internal presence-carrying value. Until
  that decision, R8-4 may not edit any symbol and `CTX-R8-05` cannot be proven. Changing the
  upstream analyzer `Checkpoint` schema is outside R8.

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

## Gate Map

| Gate | R8 exit condition |
|---|---|
| `CTX-R8-01` | The closed R7 analyzer/delegation contract remains the unchanged semantic input. |
| `CTX-R8-02` | R8 MAP/SPEC/PLAN/TASKS are internally consistent and fresh-review-clean before code. |
| `CTX-R8-03` | Replay and live use the same typed interpretation function and parity matrix. |
| `CTX-R8-04` | One compatibility owner preserves v0.2 and v0.3-v0.8 contract behavior. |
| `CTX-R8-05` | After an explicit `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` decision, operator core code formats the internal optional delegation fact without analyzer/delegation inference; the chosen public compatibility representation and its exact parity/source-compatibility tests are green. |
| `CTX-R8-06` | Existing pre-observe transport bookkeeping is permitted; after interpretation failure there is no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance. |

`CTX-R8-01` is currently `PROVEN` by the stable R7 contract plus the clean MAP/SPEC freeze. The
remaining rows are future criteria, not completion claims. The control-pack ledger remains
authoritative until reviewed evidence updates it.

## Required Docs Before Implementation

1. this MAP — included in fresh-review-clean contract series `698c766f9` + `f5865fb7` +
   `95529809`;
2. the R8 specification — included in the same fresh-review-clean contract series;
3. an exact migration/verification PLAN — the family through `0ed3d8f04` + `cfcf65507` received
   `CHANGES_REQUIRED`; first fix `2b9565fb9` and the current later-three-finding Markdown fix await
   fresh re-review, and the current fix claims no review result;
4. packetized TASKS with the exact future HIGH-impact and delegation-presence decision gates — same
   review/fix state, with all implementation tasks unchecked and unstarted.

Stop at documentation authoring. Do not stage or implement source/test changes under this map until
all four artifacts are review-clean and `CTX-R8-02` is proven.
