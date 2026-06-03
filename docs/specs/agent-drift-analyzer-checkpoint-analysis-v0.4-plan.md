# Plan: Agent Drift Analyzer Checkpoint Analysis v0.4

## Scope

This plan implements:

- `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-spec.md`

The goal is to deepen the analyzer so one internal module owns checkpoint time semantics while the
first implementation pass preserves the external checkpoint contract.

This slice should:

- add an internal `CheckpointAnalysis` seam
- route checkpoint diagnostics through that seam
- move `WrongPlanBranch` from cumulative-prefix semantics to interval-local semantics
- rename `IgnoringRepoTruth` to `TruthGroundingGap`
- make `TruthGroundingGap` hybrid: active from the latest interval, historical evidence preserved
- make `DeadEndThrash` explicit about recovery instead of staying active accidentally
- revalidate replay/live sentinel readers against the unchanged external checkpoint contract

This slice should not:

- redesign checkpoint segmentation itself
- widen the compactor bundle contract
- redesign sentinel scheduler/debounce policy
- broaden into shell/world/shim integration

## Why This Slice Comes Next

The current crate state makes this the right next analyzer follow-up:

- the analyzer already has richer checkpoint-local diagnostics, but its drift scorers still mostly
  consume cumulative prefix state
- the real-session live path now makes the late-checkpoint stickiness visible in a user-facing way
- the sticky live classification is rooted in analyzer semantics rather than sentinel scheduling
- the external checkpoint contract is still small enough that an internal deepening can happen
  before any downstream schema widening

That means the highest-leverage move is not threshold tuning. It is to deepen the checkpoint
analysis module so current-state claims and historical claims stop leaking into each other.

## Implementation Strategy

Build `v0.4` from the inside out:

1. lock the semantic contract in docs and add the internal `CheckpointAnalysis` seam
2. route diagnostics through that seam first with no intended behavior change
3. move `WrongPlanBranch` onto explicit interval-local inputs
4. rename `IgnoringRepoTruth` to `TruthGroundingGap` and shift it to hybrid semantics
5. move `DeadEndThrash` onto explicit repetition plus recovery inputs
6. revalidate analyzer export and sentinel consumers, then rerun the sticky-session proof

This keeps the risky semantic changes behind one new internal module instead of spreading them
incrementally across unrelated helpers.

Packet `v0.4A` execution lock:

- land only the docs lock, the internal `CheckpointAnalysis` seam, and diagnostics routing
- preserve the exported checkpoint schema and current scorer outputs
- defer `WrongPlanBranch`, `TruthGroundingGap`, and `DeadEndThrash` semantic changes to later
  packets exactly as listed below

## Major Components

### 1. Internal CheckpointAnalysis Seam

Deliver first:

- add `CheckpointAnalysis`
- add `CheckpointSlice`, `IntervalSlice`, `RepetitionSlice`, `TaskFrameDelta`, and
  `RecoveryState`
- keep it analyzer-internal for the first pass

Why first:

- every later scoring and diagnostics change depends on one shared time-semantics module

### 2. Diagnostics Migration With No Intended Behavior Change

Deliver second:

- derive `CheckpointDiagnostics` from `CheckpointAnalysis`
- preserve current exported diagnostics fields and summary behavior

Why second:

- diagnostics are already partly interval-aware and are the lowest-risk first consumer of the new
  seam

### 3. WrongPlanBranch Interval Migration

Deliver third:

- compute out-of-scope actions from the explicit interval slice
- stop letting historical prefix actions keep the latest checkpoint active

Why third:

- this is the most direct fix for the sticky late-session live issue

### 4. TruthGroundingGap Rename And Hybrid Migration

Deliver fourth:

- rename `IgnoringRepoTruth` to `TruthGroundingGap`
- move the class to latest-interval active semantics
- preserve earlier grounding gaps as historical evidence only

Why fourth:

- the rename sharpens the semantics before the hybrid behavior lands
- it keeps the analyzer honest about what it can actually prove

### 5. DeadEndThrash Recovery Migration

Deliver fifth:

- compute repeated loop evidence from the repetition slice
- compute recovery from the latest interval and latest failure cluster
- clear active thrash after one clean verification interval

Why fifth:

- this change is semantically heavier and depends on the analysis seam already existing

### 6. Consumer Revalidation And Bounded Proof

Deliver sixth:

- rerun replay/live sentinel readers
- confirm the unchanged checkpoint contract remains consumable
- rerun the bounded sticky-session proof and validate late-checkpoint honesty

Why sixth:

- this slice is only complete if it resolves the live symptom without breaking checkpoint
  consumers

## Sequencing

Sequential work:

1. docs and contract lock
2. `CheckpointAnalysis` construction
3. diagnostics migration
4. `WrongPlanBranch` migration
5. `TruthGroundingGap` rename and hybrid migration
6. `DeadEndThrash` recovery migration
7. export/sentinel validation and bounded proof

Parallel-safe work after the contract is locked:

- test authoring for `CheckpointAnalysis` construction can proceed in parallel with diagnostics
  migration
- export compatibility test updates can proceed in parallel with scorer migrations
- continuity-doc refresh can proceed in parallel with the bounded proof once final semantics are
  stable

## Risks And Mitigations

### Risk 1: The new seam becomes another pass-through module

Risk:

- `CheckpointAnalysis` could become a struct dump that does not actually concentrate logic

Mitigation:

- make diagnostics and all three drift classes consume it
- do not allow scorers to keep reconstructing their own time model once the seam lands

### Risk 2: Active and historical evidence become muddled in export

Risk:

- the unchanged `evidence: Vec<EvidenceRef>` payload could blur current and historical reasons

Mitigation:

- preserve current evidence first
- append historical evidence only with explicit historical reason strings

### Risk 3: The rename creates downstream churn

Risk:

- sentinel consumers or tests may assume the old enum/class label

Mitigation:

- update analyzer and sentinel regression surfaces together
- keep the external checkpoint row shape stable even if the enum variant name changes

### Risk 4: Recovery semantics become too optimistic

Risk:

- one clean verification interval might clear `DeadEndThrash` too aggressively

Mitigation:

- make recovery depend on both verification-like work and absence of newly introduced repeated
  failure evidence
- validate against the bounded sticky-session proof and targeted thrash regressions

### Risk 5: The slice drifts into checkpoint segmentation redesign

Risk:

- maintainers may try to solve semantics by changing checkpoint windows themselves

Mitigation:

- hold segmentation constant in `v0.4`
- treat this slice as analysis deepening, not segmentation redesign

## Verification Checkpoints

### Checkpoint VA-A: Internal Seam Is Real

Must be true:

- `CheckpointAnalysis` exists
- diagnostics consume it
- no scorer-facing time semantics live only in ad hoc helper logic anymore

Verify:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Checkpoint VA-B: WrongPlanBranch Is Honest About Current Posture

Must be true:

- historical out-of-scope actions do not keep later checkpoints active after in-scope recovery

Verify:

```bash
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
```

### Checkpoint VA-C: TruthGroundingGap Semantics Are Renamed And Hybrid

Must be true:

- the class name changes to `TruthGroundingGap`
- active state depends on the latest interval
- historical grounding gaps remain visible without forcing the active flag

Verify:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

### Checkpoint VA-D: DeadEndThrash Recovers Explicitly

Must be true:

- active thrash clears after one clean verification interval
- historical thrash evidence remains available

Verify:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
```

### Checkpoint VA-E: Sentinel Consumers Still Read The Contract

Must be true:

- replay and live sentinel readers still consume exported checkpoints successfully

Verify:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

### Checkpoint VA-F: Sticky Live Proof Is Resolved

Must be true:

- the bounded sticky-session proof no longer leaves late checkpoints active solely because of
  earlier prefix behavior

Verify:

```bash
cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

## Exit Conditions

This slice is complete only when all of the following are true:

- `CheckpointAnalysis` is the internal source of truth for checkpoint time semantics
- `WrongPlanBranch` is interval-local
- `IgnoringRepoTruth` is renamed to `TruthGroundingGap`
- `TruthGroundingGap` is hybrid
- `DeadEndThrash` uses explicit recovery semantics
- sentinel consumers still read the checkpoint contract
- the bounded sticky-session proof demonstrates late-session recovery honestly
