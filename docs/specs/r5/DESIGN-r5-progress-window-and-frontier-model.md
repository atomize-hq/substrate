# Design: R5 Progress Window And Frontier Model

Status: canonical design authority locked in Packet R5-0 on 2026-06-09.

## Why This Doc Exists

A checkpoint should not compare the current verification result against the entire session history
as one undifferentiated blob. DoVer's trial segmentation and TRAJEVAL's destructive-regression
findings both point to the same local rule: progress should be evaluated inside a bounded window
where the objective, archetype, and working frontier are still comparable.

This doc defines the R5 analyzer-local equivalent of DoVer-style trials: `ProgressWindow`.

## Current Repo Reality

The analyzer already has natural window inputs:

1. `checkpoint_analyses(session)` constructs an ordered list of `CheckpointAnalysis` values.
2. Each `CheckpointAnalysis` has:
   - current and previous `CheckpointSlice`
   - `DelegationContext`
   - `IntervalSlice`
   - `TurnContext`
   - `RepetitionSlice`
   - `TaskFrameDelta`
   - `RecoveryState`
3. `checkpoint_windows(session)` already creates checkpoint windows by phase boundaries and max rows
   per checkpoint.
4. `TaskFrameDelta` currently records only:
   - `task_frame_transitioned`
   - `working_set_changed`
5. R4 currently builds one `SessionArchetype` per checkpoint and may change label across adjacent
   checkpoints.

R5 should build on this existing checkpoint sequence. It should not introduce a new upstream
compactor phase model.

## Core Concept

A `ProgressWindow` is a comparable prefix region where the analyzer can honestly ask: compared to
recent relevant work, is this checkpoint advancing, stalled, regressing, mixed, or too sparse?

Recommended internal shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProgressWindow {
    pub id: usize,
    pub checkpoint_ordinals: Vec<usize>,
    pub start_ordinal: usize,
    pub end_ordinal: usize,
    pub archetype_label: SessionArchetypeLabel,
    pub dimension: ProgressDimension,
    pub reset_reasons: Vec<ProgressWindowResetReason>,
    pub best_frontier: Option<FrontierSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ProgressWindowResetReason {
    NewSession,
    ArchetypeChanged,
    TaskFrameChanged,
    WorkingSetShifted,
    UserReplanned,
    DelegationVisibilityChanged,
    LongGapOrSparseEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrontierSnapshot {
    pub attempt_ordinal: usize,
    pub status_rank: FrontierRank,
    pub signature: Option<DiagnosticSignature>,
    pub clean_verification: bool,
    pub failing_count: Option<u32>,
    pub paths: Vec<String>,
}
```

## Window Boundaries

Start a new progress window when any high-confidence comparability break occurs:

1. new session,
2. strong `session_archetype.label` change,
3. `task_frame_delta.task_frame_transitioned` with changed objective/truth artifacts,
4. working set shifts into mostly unrelated paths,
5. user steer or prompt explicitly replans the task,
6. delegation topology or child-work visibility changes,
7. evidence becomes too sparse after a long gap to compare confidently.

Do not reset on every weak signal. For example:

1. one new verification command in the same source/test scope should stay in the same window,
2. adding a test file for a source edit should stay in the same implementation/troubleshooting
   window,
3. closeout proof that broadens from focused test to full suite should stay in the same closeout
   window.

## Dimension Selection

Dimension should generally derive from `session_archetype.label`:

| Archetype | Default dimension |
|---|---|
| `troubleshooting` | `TroubleshootingFrontier` |
| `planning` | `PlanningConvergence` |
| `autonomous_implementation` | `ImplementationVerificationWall` |
| `verification_closeout` | `VerificationCloseoutNarrowing` |

Delegation may override dimension to `ParentVisibleOrchestration` when child work is opaque or only
parent-visible orchestration evidence is available.

## Best-So-Far Frontier

R5 should track a best-so-far frontier inside each progress window. This is the mechanism that lets
the analyzer distinguish:

1. not moving (`Stalled`),
2. moving forward (`Advancing`),
3. moving backward after prior progress (`Regressing`),
4. mixed evidence.

Recommended order for troubleshooting and implementation verification frontiers:

```text
no verification evidence
  < env/dependency/setup failure
  < compile/build failure
  < test discovery failure
  < test execution failure
  < assertion/golden/replay mismatch
  < clean focused verification
  < clean broader verification
```

A checkpoint regresses when it previously reached a higher comparable frontier and later falls back
to a lower one on the same or overlapping target after fresh edits.

Examples:

1. `cargo check` compile failure -> `cargo test foo` assertion failure = advancing.
2. `cargo test foo` assertion failure -> `cargo test foo` same failure, no edits = stalled.
3. `cargo test foo` clean -> later `cargo check` compile failure in same touched module = regressing.
4. `cargo test foo` failure -> `pnpm test unrelated` failure = insufficient/mixed unless target
   overlap is established.

## Status Decision Algorithm

The first implementation should use an interpretable signal aggregation rather than a numeric
reward.

Pseudo-shape:

```rust
fn assess_progress(analysis: &CheckpointAnalysis, prior: &ProgressWindowState) -> SessionProgress {
    let attempts = build_command_attempts(analysis);
    let verifications = build_verification_attempts(&attempts);
    let window = derive_progress_window(analysis, prior);
    let signals = collect_progress_signals(analysis, &window, &verifications);
    let capped = apply_delegation_caps(analysis, signals);
    aggregate_progress_status(capped)
}
```

Aggregation rules:

```text
Regressing wins when:
  there is strong direct evidence that a previously cleaner/later frontier is now worse,
  especially when the same or overlapping scope is broken.

Advancing wins when:
  direct diagnostic/frontier evidence improves,
  or the archetype-specific structural frontier narrows with limited contradiction.

Stalled wins when:
  comparable attempts repeat the same signature or structural state,
  no overlapping edits or narrowing occur,
  and evidence is direct enough to avoid insufficient_evidence.

Mixed wins when:
  positive and negative signals are both material,
  or comparable attempts show improvement in one dimension and deterioration in another.

InsufficientEvidence wins when:
  no comparable attempts exist,
  verifier target is not exercised,
  parser confidence is too low,
  or delegation opacity prevents an honest archetype-native claim.
```

## Sparse Evidence

Sparse evidence is common. The first implementation should not force confident labels.

Guidelines:

1. One read command and no artifact or verifier usually means `insufficient_evidence` for progress,
   even if R4 labels the checkpoint as `planning`.
2. One edit command and no verification may support low-confidence `mixed` or
   `insufficient_evidence` for implementation progress, not `advancing` by default.
3. Repeated exact verifier failures can support `stalled` even without perfect parsing when command
   and output hashes are stable and no edits intervened.
4. Planning convergence should rarely be `high` in v0.6 because it uses structural proxies rather
   than direct machine diagnostics.

## Delegation Caps

R5 must consume the R3.75 delegation boundary but not implement R7.

Rules:

```text
single_agent:
  normal progress rules apply.

delegating_parent + partial visibility:
  allow low/medium progress only for visible evidence;
  attach DelegationVisibilityLimited when visibility affects confidence.

delegating_parent + opaque visibility:
  emit ParentVisibleOrchestration or InsufficientEvidence;
  never high confidence;
  never claim child troubleshooting frontier movement or child implementation progress.

mixed_or_ambiguous + opaque visibility:
  prefer InsufficientEvidence unless direct parent-visible orchestration is clear.
```

## Relationship To R6

R5 status is not the final posture. R6 may later decide that:

1. `TroubleshootingFrontier + Advancing` suppresses or lowers `dead_end_thrash`,
2. `TroubleshootingFrontier + Stalled` strengthens it,
3. `Regressing` is more severe than `Stalled`,
4. `InsufficientEvidence` should not be over-scored.

R5 should not pre-bake those scoring decisions.

## Validation Implications

Every fixture in the R5 acceptance matrix should identify:

1. progress window id or expected reset reason,
2. expected dimension,
3. expected status,
4. confidence ceiling/floor,
5. decisive before/after frontier evidence,
6. why unrelated attempts do not compare.

## Non-Goals

This design does not:

1. require per-action stage labeling,
2. require global full-session causality inference,
3. stitch child sessions to parent sessions,
4. use reference patches or gold solutions at runtime,
5. retune drift scores.

## Locked Decisions After Packet R5-0

1. Progress-window ids stay debug-only in R5; they are not part of the public `SessionProgress`
   contract.
2. A task-frame transition resets a planning/progress window only when objective, truth-artifact,
   or working-set comparability changes materially; small refinements inside the same narrowed plan
   stay in-window.
3. `Clean broader verification` outranks `clean focused verification` inside the same comparable
   window in the first landing.
