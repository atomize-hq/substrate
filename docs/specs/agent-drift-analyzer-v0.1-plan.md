# Plan: Agent Drift Analyzer v0.1

## Scope

This plan implements [agent-drift-analyzer-v0.1-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-v0.1-spec.md:1).

The goal is to deliver a library-first crate with a thin CLI that:

- consumes session-scoped compactor artifacts
- assembles objective, truth-artifact, and working-set context
- infers a current `task frame`
- scores three deterministic drift classes
- emits reviewable checkpoints

This module depends on the compactor artifact contract being stable enough to consume directly.
That contract is now concrete: analyzer work should assume the landed compactor bundle and row
shapes first, not the richer hypothetical event taxonomy from earlier sentinel ideation.

## Implementation Strategy

Build the analyzer around checkpoint semantics, not around ad hoc heuristics.

Implementation order:

1. lock the input contract against compactor artifacts
2. build deterministic context assembly
3. build task-frame inference
4. build scoring logic for the three drift classes
5. build checkpoint segmentation and export
6. add the thin CLI after the library path is stable

This keeps the logic centered on evidence and makes drift scoring reviewable before any live
integration exists.

## Major Components

### 1. Input Contract Loader

Deliver first:

- loading for `manifest.json`
- loading for `rows.archival.jsonl`
- loading for `rows.compact.jsonl`
- loading for `dedupe-audit.jsonl`
- session-scope validation and ordering checks
- row-contract validation against the landed `CompactionRow`, `RowRef`, and `DedupeGroup` shapes

Critical rule:

- if the compactor artifact surface is too distorted because of an upstream parser problem, pause and
  resolve that at the parser or compactor seam instead of compensating inside the analyzer

### 2. Context Assembly

Deliver second:

- objective extraction from user-originated rows
- candidate truth artifact ranking
- observed working-set assembly from the landed row kinds, dedupe identities, and payload text
- command-family and tool summaries

Why second:

- both task-frame inference and drift scoring depend on the same context pack

### 3. Task-Frame Inference

Deliver third:

- deterministic task-frame hypothesis assembly
- confidence shaping
- branch-shift detection
- counter-evidence capture

Why third:

- drift scoring should compare behavior against a task frame, not derive a new explanation in each
  scoring path

### 4. Drift Scoring

Deliver fourth:

- `wrong_plan_branch`
- `ignoring_repo_truth`
- `dead_end_thrash`

Critical rule:

- `dead_end_thrash` must read repetition-preserving evidence, not compacted-only rows

### 5. Checkpoint Segmentation

Deliver fifth:

- deterministic checkpoint boundaries for session analysis
- evidence refs attached to checkpoints
- summary material for operator review

### 6. Export Bundle

Deliver sixth:

- `checkpoints.jsonl`
- `summary.md`
- optional machine-readable evaluation artifact if justified during implementation

### 7. Thin CLI

Deliver last:

- input-dir selection
- output-dir selection
- session validation
- operator-facing exit errors

## Sequencing

Sequential work:

1. input contract loading
2. context assembly
3. task-frame inference
4. drift scoring
5. checkpoint segmentation
6. export
7. CLI

Parallel-safe work after context assembly stabilizes:

- `wrong_plan_branch` and `ignoring_repo_truth` scoring can be developed in parallel
- checkpoint export can proceed in parallel with summary rendering
- calibration fixtures can be assembled in parallel with final CLI wiring

## Risks And Mitigations

### Risk 1: Compactor rows omit needed evidence

Risk:

- the analyzer may need evidence not exposed cleanly in the row surface

Mitigation:

- validate analyzer needs at the compactor handoff checkpoint
- prefer improving the compactor contract over analyzer-side guessing
- do not assume first-class file or command event kinds that the compactor does not currently emit

### Risk 2: Task-frame inference is too fuzzy

Risk:

- deterministic inference may be unstable or overly sensitive to wording

Mitigation:

- prefer literal user language
- separate supporting evidence from counter-evidence
- make confidence drops explicit and test-covered

### Risk 3: Drift scoring collapses into opaque heuristics

Risk:

- the scoring layer could become difficult to audit or calibrate

Mitigation:

- explicit thresholds
- explicit evidence refs
- separate drift-class modules

### Risk 4: Thrash detection is broken by compaction

Risk:

- using compacted-only rows would erase repetition evidence

Mitigation:

- design thrash scoring around archival or repetition-preserving derived views from the start

## Verification Checkpoints

### Checkpoint A: Input Contract Validated

Verify:

- analyzer loads compactor artifacts without guessing
- session boundaries are explicit
- the landed row contract is sufficient for initial context assembly, or the gap is documented as a
  contract-widening decision

Evidence:

- fixture tests against expected compactor outputs

### Checkpoint B: Context Pack Stable

Verify:

- objective, truth artifacts, and working set are stable enough to review

Evidence:

- fixture-driven tests
- human review of sample derived context packs

### Checkpoint C: Drift Scoring Reviewable

Verify:

- all three drift classes emit explicit evidence refs and threshold behavior

Evidence:

- positive and negative scoring fixtures
- boundary tests around thresholds

### Checkpoint D: Checkpoint Bundle Complete

Verify:

- `checkpoints.jsonl` and summary output are stable and operator-readable
- CLI is only a thin wrapper

Evidence:

- integration tests
- end-to-end session run using the documented `cargo run` command

## Handoff To Next Module

This plan is complete enough for `agent-drift-sentinel` only when all of the following are true:

- checkpoints are deterministic and replayable
- evidence refs are stable enough for operator surfaces
- expected-next-step style output is well-defined enough for warning summaries
- ambiguity handling is explicit enough to define when optional adjudication is justified

## Exit Criteria

The plan is complete when:

1. the crate builds and tests cleanly
2. session-scoped analyzer output is reviewable without a live runtime
3. all three drift classes are deterministic and evidence-backed
4. the checkpoint contract is stable enough for replay-mode sentinel consumption
