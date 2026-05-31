# Plan: Agent Drift Analyzer Checkpoint Calibration v0.2

## Scope

This plan implements
[agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md:1).

The goal is to extend `summary.md` so it helps calibrate checkpoint usefulness rather than merely
listing checkpoint counts.

This slice should:

- quantify checkpoint density against turns and user prompts
- quantify checkpoint spacing against rows and elapsed time
- surface session diagnostics that explain whether checkpointing is clustering around real semantic
  shifts or around noise
- document the next-level metrics that remain blocked on richer normalization

This slice should not:

- widen the replay-facing checkpoint JSONL contract by default
- add new drift classes
- rework checkpoint segmentation logic itself
- block on richer compactor event kinds before landing the first calibration report

## Implementation Strategy

Build the calibration slice around explicit metric definitions before touching rendering.

Implementation order:

1. define metric semantics and denominator rules
2. compute per-session metrics from bundle sessions, context, and checkpoints
3. aggregate top-level metrics across analyzed sessions
4. render a concise operator summary block
5. document deferred next-level metrics and the exact contract gaps that block them

This keeps the work deterministic and avoids baking ambiguous semantics into the summary format.

## Major Components

### 1. Metric Contract

Deliver first:

- exact definitions for `turns observed` and `user prompts observed`
- role-aware definition of `user prompts observed` as `user_message_role = prompt`
- denominator behavior for ratios when zero prompts or zero turns exist
- interval semantics for `rows between checkpoints` and `seconds between checkpoints`
- flagged-streak semantics

Critical rule:

- ratios and intervals must measure gaps between checkpoints, not cumulative checkpoint-window size

### 2. Session Metric Assembly

Deliver second:

- turn count from distinct non-null `turn_id` values
- user-prompt count from `CompactionKind::UserMessage` rows classified as `UserMessageRole::Prompt`
- checkpoint density ratios
- interval metrics from adjacent checkpoint boundaries
- longest flagged streak

Why second:

- these metrics are the calibration floor and do not depend on richer summary presentation decisions

### 3. Context-Derived Diagnostics

Deliver third:

- distinct task-frame count
- truth-artifact count
- verification-command count
- reuse of existing prompt/steer/unknown supporting counts where they help interpret density ratios

Why third:

- these explain why checkpoint density may be high or low without changing the checkpoint contract

### 4. Summary Rendering

Deliver fourth:

- top-level calibration block
- per-session diagnostic block
- concise unavailable-metric formatting where timestamps or denominators are missing

Why fourth:

- rendering should follow settled semantics, not define them

### 5. Deferred-Metrics Ledger

Deliver fifth:

- explicit note in the docs about which next-level metrics need richer normalization
- exact contract gaps for:
  - phase transition counts
  - phase duration
  - first grounding before first write
  - first verification lag
  - work-area shift count
  - stronger repetition diagnostics

Why fifth:

- this prevents the follow-up slice from silently turning into a compactor redesign

## Sequencing

Sequential work:

1. metric contract
2. session metric assembly
3. context-derived diagnostics
4. summary rendering
5. deferred-metrics documentation

Parallel-safe work after metric semantics stabilize:

- export summary tests can proceed in parallel with bounded smoke verification
- docs updates can proceed in parallel with per-session diagnostic helpers

## Risks And Mitigations

### Risk 1: The summary overstates what a "turn" means

Risk:

- readers may interpret `turns observed` as identical to user prompts

Mitigation:

- report `turns observed` and `user prompts observed` separately
- define `user prompts observed` from the landed prompt-role classifier rather than from all user
  message rows
- keep the metric names literal and documented

### Risk 2: Checkpoint spacing is measured incorrectly

Risk:

- using cumulative checkpoint windows would inflate row and time spacing

Mitigation:

- compute interval metrics from adjacent checkpoint boundaries only
- add tests that fail if cumulative windows are treated as interval counts

### Risk 3: Timestamp coverage is too weak for time metrics

Risk:

- missing or sparse row timestamps could produce misleading averages

Mitigation:

- compute from checkpoint boundary rows only
- render unavailable metrics explicitly when timestamps are absent

### Risk 4: The summary becomes too verbose to scan

Risk:

- adding every diagnostic could turn `summary.md` into a wall of metrics

Mitigation:

- keep the first slice to the selected metric set
- defer richer calibration data to a later machine-readable artifact if needed

### Risk 5: The slice drifts into normalization redesign

Risk:

- next-level metric ideas could force compactor or analyzer contract widening mid-slice

Mitigation:

- land the first deterministic summary metrics from existing surfaces
- record blocked next-level metrics explicitly instead of widening the contract casually

## Verification Checkpoints

### Checkpoint AC-A: Metric Semantics Locked

Verify:

- summary definitions are explicit enough that two engineers would compute the same numbers

Evidence:

- spec + plan alignment
- tests naming the metric semantics directly

### Checkpoint AC-B: Session Metrics Are Deterministic

Verify:

- per-session counts and ratios are stable on repeated runs

Evidence:

- export-bundle tests
- bounded smoke session reruns

### Checkpoint AC-C: Summary Stays Operator-Readable

Verify:

- the top-level block surfaces the primary ratios and spacing metrics without forcing the reader to
  inspect checkpoint-by-checkpoint detail

Evidence:

- rendered summary review
- bounded smoke summary output

### Checkpoint AC-D: Deferred Metrics Are Explicit

Verify:

- blocked next-level metrics and their contract gaps are documented in repo docs rather than left
  implicit in chat

Evidence:

- follow-up note in the spec/plan/task chain

## Handoff To Later Slices

This plan is complete enough for a later analyzer or compactor follow-up only when all of the
following are true:

- the first calibration summary is stable and useful in operator review
- the team can name which remaining questions are about checkpoint cadence versus missing event
  normalization
- any requested schema or normalization widening is grounded in a concrete blocked metric

## Exit Criteria

The plan is complete when:

1. the analyzer summary reports the agreed top-level checkpoint-calibration metrics
2. each session block reports the agreed per-session diagnostics
3. the output stays deterministic and concise on bounded real-session reruns
4. the next-level blocked metrics are documented with exact contract gaps
