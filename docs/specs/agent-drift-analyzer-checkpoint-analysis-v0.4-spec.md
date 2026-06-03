# Spec: Agent Drift Analyzer Checkpoint Analysis v0.4

## Assumptions I'm Making

1. This is an analyzer-only redesign slice on top of the already-landed compactor `v0.2` bundle
   contract and analyzer `v0.3` diagnostics contract.
2. The immediate product problem is semantic honesty in late-session checkpoint classification,
   especially for real-session live monitoring, not a new sentinel scheduler policy.
3. The current analyzer architecture is still carrying proof-of-concept shape:
   - checkpoint windows are cumulative prefixes
   - checkpoint diagnostics are interval-aware
   - drift scorers still mostly consume cumulative prefix state
4. The live sticky-warning issue is rooted in analyzer time semantics, not in the sentinel runtime.
5. The repository is still effectively greenfield for drift-class naming, so changing
   `IgnoringRepoTruth` to `TruthGroundingGap` now is acceptable and preferable.
6. The first implementation pass should deepen the analyzer with a new internal module and preserve
   the exported `Checkpoint` schema unless a later slice explicitly widens it.

## Objective

Deepen the analyzer so one internal module owns the time semantics of a checkpoint.

Primary user:

- the engineer or operator reading analyzer output and expecting the latest checkpoint to describe
  current posture honestly while still preserving historical drift evidence

Success means:

- the analyzer constructs an internal `CheckpointAnalysis` value for each checkpoint ordinal
- drift classes read explicit slices from that analysis instead of reconstructing their own notion
  of time from cumulative prefix state
- `WrongPlanBranch` becomes a current-state claim
- `TruthGroundingGap` becomes a hybrid claim: active from the latest interval, historical evidence
  preserved separately
- `DeadEndThrash` becomes a historical claim with explicit recovery rules instead of accidental
  prefix stickiness
- the external `Checkpoint` contract can remain unchanged in the first implementation pass

## Problem Statement

The current analyzer mixes three different time models inside one checkpoint:

- cumulative prefix context from `checkpoint_windows(...)`
- interval-local diagnostics from `checkpoint_diagnostics(...)`
- repetition-preserving archival history for `dead_end_thrash`

That makes the checkpoint module shallow. A caller or test has to understand cumulative windows,
previous-window subtraction, and scorer-specific history rules spread across multiple modules.

The live sticky-warning issue is a direct result:

- late checkpoints can still carry `wrong_plan_branch` from earlier out-of-scope prefix actions
- `ignoring_repo_truth` currently overclaims intent and also consumes prefix semantics
- `dead_end_thrash` stays active because recovery is not first-class

The correct fix is to deepen the analyzer around one internal checkpoint-analysis seam rather than
to tune warning thresholds downstream.

## Tech Stack

- Language: Rust 2021
- Target crate: `crates/agent-drift-analyzer`
- Existing upstream input contract: compactor bundle schema `v0.2`
- Existing downstream consumers:
  - exported `checkpoints.jsonl`
  - exported `summary.md`
  - replay and live sentinel checkpoint readers

Dependency posture:

- no compactor bundle change is required first
- no sentinel scheduler change is required first
- no model-assisted adjudication is required
- no new crate is required

## Commands

Targeted analyzer validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Focused sentinel compatibility validation:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Bounded real-session proof shape after implementation:

```bash
export SESSION_ID="<active-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"

cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

## Project Structure

```text
crates/agent-drift-analyzer/src/lib.rs
  Threads loaded bundle sessions into checkpoint analysis, scoring, and export.

crates/agent-drift-analyzer/src/input.rs
  Loads and validates the compactor bundle and session-scoped row surfaces.

crates/agent-drift-analyzer/src/context/
  Assembles objective, truth-artifact, working-set, tool, and command-observation surfaces.

crates/agent-drift-analyzer/src/inference/mod.rs
  Builds `TaskFrame` and confidence from a `ContextPack`.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Defines checkpoint windows, boundaries, diagnostics, and the new internal checkpoint-analysis
  seam.

crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Defines the external checkpoint contract and drift-class names.

crates/agent-drift-analyzer/src/scoring/
  Produces drift scores from the checkpoint-analysis seam instead of ad hoc cumulative inputs.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Owns exported `checkpoints.jsonl` and `summary.md` rendering.

crates/agent-drift-analyzer/tests/
  Regression coverage for checkpoint semantics, class behavior, and export compatibility.
```

## Code Style

Keep the analyzer library-first and make time semantics explicit in types instead of reconstructing
them ad hoc inside scorers.

```rust
pub struct CheckpointAnalysis {
    pub session_id: String,
    pub ordinal: usize,
    pub current: CheckpointSlice,
    pub previous: Option<CheckpointSlice>,
    pub interval: IntervalSlice,
    pub repetition: RepetitionSlice,
    pub task_frame_delta: TaskFrameDelta,
    pub recovery: RecoveryState,
}
```

Conventions:

- prefer explicit structs over tuple-like helper return values when carrying checkpoint semantics
- keep scorer interfaces narrow and semantic; pass `CheckpointAnalysis` or its dedicated slices
  instead of raw `BundleSession` plus loosely related helpers
- keep exported checkpoint changes additive or deferred; deepen the internal module before widening
  the external contract
- name drift classes after what the analyzer can actually prove, not stronger intent claims
- keep historical versus current evidence explicit in reason strings when both must coexist in the
  exported payload

## Design Summary

Add one internal module value:

```rust
pub struct CheckpointAnalysis {
    pub session_id: String,
    pub ordinal: usize,
    pub current: CheckpointSlice,
    pub previous: Option<CheckpointSlice>,
    pub interval: IntervalSlice,
    pub repetition: RepetitionSlice,
    pub task_frame_delta: TaskFrameDelta,
    pub recovery: RecoveryState,
}
```

The analyzer flow becomes:

1. build cumulative checkpoint windows exactly as today
2. construct `CheckpointAnalysis` for each checkpoint ordinal
3. score drift classes from `CheckpointAnalysis`
4. derive diagnostics from `CheckpointAnalysis`
5. export the existing `Checkpoint` value

The deepening is not “remove cumulative windows.” The deepening is “make one module own how a
checkpoint relates to the prior checkpoint and to repetition-preserving history.”

## Internal CheckpointAnalysis Module

### Core Shape

```rust
pub struct CheckpointSlice {
    pub window: BundleSession,
    pub context: ContextPack,
    pub task_frame: TaskFrame,
}

pub struct IntervalSlice {
    pub compact_rows: Vec<CompactionRow>,
    pub command_observations: Vec<CommandObservation>,

    pub truth_artifacts: Vec<String>,
    pub grounded_reads: Vec<EvidenceRef>,
    pub ungrounded_actions: Vec<EvidenceRef>,
    pub in_scope_actions: Vec<EvidenceRef>,
    pub out_of_scope_actions: Vec<EvidenceRef>,
    pub verification_actions: Vec<EvidenceRef>,
}

pub struct RepetitionSlice {
    pub archival_rows: Vec<CompactionRow>,
    pub repeated_verification_loops: Vec<RepeatedCommandLoop>,
    pub repeated_failure_loops: Vec<RepeatedFailureLoop>,
    pub last_failure_cluster_end: Option<RowRef>,
}

pub struct TaskFrameDelta {
    pub transitioned: bool,
    pub working_set_changed: bool,
    pub objective_changed: bool,
}

pub struct RecoveryState {
    pub clean_verification_interval: bool,
    pub verified_after_last_failure_cluster: bool,
    pub goal_completion_observed: bool,
    pub recovered_from_thrash: bool,
    pub recovered_from_truth_gap: bool,
}
```

### Design Rule

The checkpoint-analysis module owns time semantics. Scorers no longer derive their own time model
from raw `BundleSession` plus helper calls.

## Time Semantics Contract

Each `CheckpointAnalysis` exposes four explicit time surfaces.

### 1. Current Slice

The current cumulative checkpoint window.

Use for:

- current checkpoint boundary
- current task frame
- current cumulative context when historical evidence still matters

### 2. Previous Slice

The immediately prior checkpoint window in the same session, if present.

Use for:

- task-frame churn
- working-set churn
- interval start computation

### 3. Interval Slice

The newly covered compact-row slice since the previous checkpoint.

Use for:

- current-state scope drift
- current-state grounding gaps
- interval diagnostics counts
- recovery signals driven by the latest verification behavior

### 4. Repetition Slice

The repetition-preserving archival slice up to the current checkpoint end.

Use for:

- repeated verification loops
- repeated failure loops
- locating the most recent failure cluster for recovery semantics

## Drift Class Contract

The analyzer class set after this redesign is:

- `WrongPlanBranch`
- `TruthGroundingGap`
- `DeadEndThrash`

### Rename Contract

Rename `IgnoringRepoTruth` to `TruthGroundingGap`.

Rationale:

- the current heuristic does not prove intent
- it detects missing grounding before action, not deliberate disregard of known truth
- the new name matches what the analyzer can actually support

## Drift Class Semantics

### WrongPlanBranch

Meaning:

- the latest interval contains write-like or verification-like work outside the expected working
  scope of the current task frame

Input surface:

- `analysis.interval.out_of_scope_actions`

Active semantics:

- interval-local only

Historical semantics:

- none required to keep the active flag set

Rule:

- if the latest interval returns in scope, the active flag clears

### TruthGroundingGap

Meaning:

- the latest interval acts without adequate grounding on likely truth artifacts

Input surface:

- `analysis.interval.truth_artifacts`
- `analysis.interval.grounded_reads`
- `analysis.interval.ungrounded_actions`
- `analysis.recovery`

Active semantics:

- current interval only

Historical semantics:

- preserve earlier grounding gaps as historical evidence
- historical grounding gaps do not keep the active flag set by themselves

Rule:

- if the latest interval performs write-like or verification-like work without adequate grounding,
  the class is active
- if the latest interval re-grounds before acting, the active flag clears
- historical evidence remains attached with explicitly historical reasons

### DeadEndThrash

Meaning:

- the session is in a repeated failure or repeated verification loop and has not yet recovered

Input surface:

- `analysis.repetition.repeated_verification_loops`
- `analysis.repetition.repeated_failure_loops`
- `analysis.recovery`

Active semantics:

- historical loop evidence plus explicit recovery state

Historical semantics:

- full historical evidence preserved

Rule:

- repetition loops can create the active state
- a recovered session clears the active flag even though the historical incident remains visible

## Recovery Semantics

### TruthGroundingGap Recovery

Lock the first implementation to:

- `recovered_from_truth_gap = true` after one clean verification interval following a grounding gap

Why:

- this keeps the live monitor honest and responsive
- it avoids requiring goal completion for routine re-grounding recovery

### DeadEndThrash Recovery

Lock the first implementation to:

- `recovered_from_thrash = true` after one clean verification interval after the last failure
  cluster

Supporting evidence:

- `goal_completion_observed = true` strengthens the recovery conclusion
- goal completion is not required to clear the active flag

### Clean Verification Interval

In this spec, a clean verification interval means:

- the interval includes verification-like work
- the interval does not introduce new repeated-failure evidence
- the interval does not preserve active out-of-scope thrash behavior

This remains heuristic and should be framed that way in code comments and operator text.

## Diagnostics Contract

`CheckpointDiagnostics` should be derived from `CheckpointAnalysis`, not from an ad hoc second path.

This means:

- `task_frame_transitioned` comes from `analysis.task_frame_delta.transitioned`
- `working_set_changed` comes from `analysis.task_frame_delta.working_set_changed`
- `interval_command_count` comes from `analysis.interval.command_observations`
- `interval_verification_command_count` comes from the verification-like subset of the interval
- `evidence_item_count` still comes from the deduped evidence across task frame and drift scores

The goal is one source of truth for time semantics.

## Exported Checkpoint Compatibility

First implementation pass:

- preserve the external `Checkpoint` struct shape if possible
- preserve `checkpoints.jsonl` as the downstream contract
- let the new internal analysis seam drive:
  - `drift_scores[*].raw_score`
  - `drift_scores[*].flagged`
  - `drift_scores[*].evidence`
  - `diagnostics`

### Historical Evidence Policy

For the first slice:

- keep the current external `evidence: Vec<EvidenceRef>` shape
- allow historical evidence items to remain in the exported payload
- historical evidence reasons must be explicit, for example:
  - `historical truth-grounding gap`
  - `historical repeated failure evidence`

Do not widen the external schema to split `active_evidence` and `historical_evidence` in this
slice.

## Migration Plan

### Phase 1: Add CheckpointAnalysis With No Behavior Change

- add internal `CheckpointAnalysis` construction
- route diagnostics through it first
- preserve existing exported behavior

### Phase 2: Move WrongPlanBranch To Interval Semantics

- switch `WrongPlanBranch` to `analysis.interval.out_of_scope_actions`
- update tests so late checkpoints clear after in-scope recovery

### Phase 3: Rename IgnoringRepoTruth To TruthGroundingGap

- rename the enum variant
- rename scorer/module/test names
- update operator and summary text to avoid the word `ignoring`

### Phase 4: Move TruthGroundingGap To Hybrid Semantics

- compute active state from the latest interval
- preserve historical evidence explicitly
- add recovery-driven tests

### Phase 5: Move DeadEndThrash To Explicit Recovery Semantics

- compute loop evidence from `analysis.repetition`
- compute recovery from `analysis.recovery`
- clear active thrash after one clean verification interval

### Phase 6: Revalidate Sentinel Consumers

- rerun replay and live sentinel regression surfaces
- prove the late-session sticky-warning issue is resolved on the bounded real-session proof

## Testing Strategy

Required test layers:

1. `CheckpointAnalysis` construction tests
   - previous/current/interval/repetition slices are deterministic
2. `WrongPlanBranch` regression tests
   - historical out-of-scope actions do not keep later checkpoints active
3. `TruthGroundingGap` regression tests
   - active latest-interval grounding gaps still flag
   - historical grounding gaps stay visible without keeping the flag active
4. `DeadEndThrash` regression tests
   - repeated failure clusters still flag
   - one clean verification interval clears the active thrash state
5. export compatibility tests
   - `checkpoints.jsonl` remains readable by sentinel consumers
   - evidence reasons clearly distinguish historical evidence
6. bounded real-session proof
   - the known sticky late-session case no longer reports active `WrongPlanBranch` after recovery

Framework and coverage expectations:

- keep unit-style helpers beside the checkpoint-analysis module where the semantics are easiest to
  isolate
- keep integration coverage in `crates/agent-drift-analyzer/tests/`
- rerun sentinel reader/regression surfaces whenever checkpoint semantics or class names change
- require one bounded real-session proof for the sticky late-session scenario before calling the
  slice complete

## Boundaries

- Always:
  - keep the spec, plan, and tasks docs aligned before implementation starts
  - preserve deterministic checkpoint ordering and session ordering
  - route checkpoint diagnostics and drift scoring through the same `CheckpointAnalysis` time
    semantics
  - preserve explicit historical evidence when active flags clear
  - rerun analyzer and sentinel validation before closing the slice
- Ask first:
  - widening the external `Checkpoint` schema beyond additive/internal-first changes
  - adding new drift classes beyond `WrongPlanBranch`, `TruthGroundingGap`, and `DeadEndThrash`
  - changing checkpoint segmentation rules instead of only changing checkpoint analysis semantics
  - changing sentinel scheduler/debounce policy as part of this analyzer slice
- Never:
  - reintroduce cumulative-prefix semantics into current-state drift classes after this slice lands
  - claim intent-based `ignored known truth` behavior from the current heuristic evidence
  - hide historical incidents by deleting evidence instead of marking it historical
  - broaden into shell/world/shim integration in this slice

## Success Criteria

- `CheckpointAnalysis` exists as the internal source of truth for checkpoint time semantics.
- `WrongPlanBranch` reads interval-local scope evidence and clears when the latest interval is back
  in scope.
- `IgnoringRepoTruth` is renamed to `TruthGroundingGap` and the renamed class reflects grounding
  heuristics rather than intent claims.
- `TruthGroundingGap` becomes hybrid: active from the latest interval, historical evidence
  preserved explicitly.
- `DeadEndThrash` uses repetition history plus explicit recovery semantics and clears after one
  clean verification interval.
- The external `Checkpoint` contract remains readable by sentinel consumers in the first
  implementation pass.
- The known sticky late-session real-session proof no longer reports active late checkpoints solely
  because of earlier prefix behavior.

## Non-Goals

This slice does not:

- redesign checkpoint segmentation itself
- redesign sentinel scheduler debounce or visibility thresholds
- add a new exported `active` versus `historical` state machine to the checkpoint schema
- infer a true intent-based `ignored known truth` class
- broaden into shell/world/shim integration

## Final Design Decisions Locked By This Spec

- add an internal `CheckpointAnalysis` module
- preserve the external checkpoint contract in the first pass
- make `WrongPlanBranch` interval-local
- rename `IgnoringRepoTruth` to `TruthGroundingGap`
- make `TruthGroundingGap` a hybrid class: current active state plus historical evidence
- make `DeadEndThrash` a historical class with explicit recovery semantics
- clear active `TruthGroundingGap` and `DeadEndThrash` after one clean verification interval
- preserve historical evidence with explicit historical reasons instead of hiding it

## Open Questions

- Should the renamed class land in one packet with the internal `CheckpointAnalysis` seam, or after
  the seam is first proven with no behavior change?
  - Current recommendation: allow the docs to lock both now, but implement the seam first.
- Should exported drift-score evidence preserve current-first ordering when historical reasons are
  appended?
  - Current recommendation: yes, current evidence first and historical evidence second.
- Does the bounded sticky-session proof need a dedicated analyzer fixture in addition to the real
  session artifact already cited in the handoff chain?
  - Current recommendation: yes for regression stability, but it is not required to validate the
    spec itself.
