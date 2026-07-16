# R8 Map: Sentinel Interpretation Consolidation / Integration

Status: **R8-SPEC ACTIVE / IN PROGRESS; CANDIDATE FIX NOT YET REVIEWED**. R7 is closed with the
stable analyzer/delegation contract recorded in [`../r7/MAP.md`](../r7/MAP.md). Candidate MAP/SPEC
commit `698c766f9` landed, and fresh independent built-in `default` review returned
`CHANGES_REQUIRED` with three scoped documentation findings. This docs-only follow-up addresses
those findings but makes no claim of fresh independent review. The R8 PLAN and TASKS remain
unauthored/unreviewed; `CTX-R8-01` remains `OPEN` / in progress, `CTX-R8-02` remains `BLOCKED`, and
R8-IMPLEMENT remains blocked/boundary-only. **No R8 code may begin until the complete
MAP/SPEC/PLAN/TASKS family is fresh-review-clean.**

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
flagged/max-score inputs, normalized posture/evidence, and v0.8 typed delegation facts. Replay and
live supply only the current checkpoint and the previous checkpoint from the same session. They do
not supply origin-specific semantic switches.

Validation and interpretation complete before either core path calls the scheduler or presentation.
Replay validates/interprets the entire selected set before constructing any report decision; live
validates/interprets one event before mutating runtime state. A contract failure therefore produces
no scheduler decision, presentation, adjudication, sink emission, cursor advancement, or checkpoint
acceptance.

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
  schema/same-session contract state. Analyzer-declared `Conflicting`, `Partial`, or `Opaque`
  delegation is valid typed input, not a prompt for sentinel field-level revalidation or inference.

## Gate Map

| Gate | R8 exit condition |
|---|---|
| `CTX-R8-01` | The closed R7 analyzer/delegation contract remains the unchanged semantic input. |
| `CTX-R8-02` | R8 MAP/SPEC/PLAN/TASKS are internally consistent and fresh-review-clean before code. |
| `CTX-R8-03` | Replay and live use the same typed interpretation function and parity matrix. |
| `CTX-R8-04` | One compatibility owner preserves v0.2 and v0.3-v0.8 contract behavior. |
| `CTX-R8-05` | Operator code formats typed facts and performs no analyzer or delegation inference. |
| `CTX-R8-06` | Scheduler/adjudication behavior and real-session delivery/cursor rules remain unchanged. |

These are criteria, not current completion claims. The control-pack ledger remains authoritative
until a reviewed phase transition updates it.

## Required Docs Before Implementation

1. this MAP — candidate commit `698c766f9` landed; three review findings addressed by this
   not-yet-reviewed follow-up;
2. the R8 specification — same candidate/fix status;
3. an exact migration/verification PLAN — unauthored/unreviewed;
4. packetized TASKS with review gates — unauthored/unreviewed.

Stop at documentation authoring. Do not stage or implement source/test changes under this map until
all four artifacts are review-clean.
