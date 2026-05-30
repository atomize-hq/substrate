# Plan: Agent Drift Sentinel Live Integration v0.3

## Scope

This plan implements
[agent-drift-sentinel-live-integration-v0.3-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-live-integration-v0.3-spec.md:1).

The goal is to deliver the next bounded post-`S10` slice for `agent-drift-sentinel`:

- add a live checkpoint intake seam
- reuse the existing scheduler and operator warning logic for incremental events
- emit operator-facing live warning events through a thin sink
- prove the slice with fixture-driven live streams

This plan is explicitly **not** broader runtime integration.
It does not authorize or require edits in `shell`, `world`, `shim`, `broker`, or host-orchestrator
code.

## Fixed Seam From Replay Mode

The following surfaces stay fixed unless a concrete defect forces redesign:

- `input.rs` remains the closed-bundle replay loader
- analyzer checkpoint generation remains owned by `agent-drift-analyzer`
- replay warning presentation remains owned by `operator_surface.rs`
- scheduler thresholds, cooldowns, and repeated-failure semantics remain shared rather than split
  into replay-vs-live forks
- `Live` stays conceptually downstream of the replay/adjudication slice rather than reopening it

In other words: the live slice extends the sentinel around the replay core; it does not replace the
replay core.

## Implementation Strategy

Build the live slice from the inside out:

1. define the live checkpoint event contract
2. prove the contract is satisfied by existing analyzer checkpoints
3. add a library-owned live runtime that reuses scheduler and presentation logic
4. add a thin operator sink surface
5. validate the whole slice with fixture-driven streams
6. stop at the post-slice runtime gate

This keeps the implementation library-first and thin-binary-friendly.

## Major Components

### 1. Live Checkpoint Event Contract

Deliver first:

- a `LiveCheckpointEvent` surface for incremental checkpoint-ready, heartbeat, repeated-failure,
  and manual-review events
- stable cursor and ordering expectations
- explicit duplicate/out-of-order handling rules

Why first:

- without a stable incremental contract, any live loop will leak transport assumptions

### 2. Analyzer Compatibility Gate

Deliver second:

- proof that the existing `agent-drift-analyzer::Checkpoint` surface carries enough information for
  live warning evaluation
- explicit stop condition if live mode would need analyzer-owned data that is not already present

Why second:

- this is the seam most likely to reveal the wrong contract

### 3. Library-Owned Live Runtime

Deliver third:

- a runtime that consumes `LiveCheckpointEvent`s
- reuse of `SchedulerPolicy`, `WarningPolicy`, and checkpoint presentation logic
- stable runtime state independent of the CLI

Why third:

- this is the smallest useful live behavior and preserves thin-binary discipline

### 4. Operator Sink

Deliver fourth:

- a structured sink or sink trait for visible warnings, silent checkpoints, and heartbeat/status
  events
- local reviewable outputs only

Why fourth:

- operator delivery is the product value, but it should sit on top of the stable runtime seam

### 5. Bounded Live Proof

Deliver fifth:

- an end-to-end proof using append-only fixture streams or an in-memory harness
- verification that replay behavior still works unchanged

Why fifth:

- it proves the live contract without pretending the broader runtime is integrated

## Sequencing

Sequential work:

1. live contract
2. analyzer compatibility gate
3. live runtime
4. operator sink
5. bounded live proof
6. post-slice runtime gate

Parallel-safe work after the contract stabilizes:

- operator sink serialization can proceed in parallel with runtime-state tests
- fixture authoring can proceed in parallel with live runtime implementation
- CLI proof wiring, if approved inside this slice, can proceed only after the library runtime and
  sink behavior are stable

## Risks And Mitigations

### Risk 1: Live mode re-owns analyzer logic

Risk:

- the sentinel may start reconstructing drift judgments from raw events or partial rows

Mitigation:

- accept analyzer checkpoints as the live semantic unit
- stop and escalate if live needs new analyzer output fields

### Risk 2: Replay and live semantics drift apart

Risk:

- live mode could fork scheduler or warning rules and make replay no longer representative

Mitigation:

- reuse `SchedulerPolicy`, `WarningPolicy`, and checkpoint presentation
- add regression coverage that compares replay and live output for equivalent checkpoint sequences

### Risk 3: Hidden runtime coupling sneaks in

Risk:

- "just enough" shell/world assumptions could leak into the sentinel crate and widen scope

Mitigation:

- keep live input adapter-driven and fixture-testable
- reject direct dependencies on execution-runtime modules in this slice

### Risk 4: Operator output becomes noisy again

Risk:

- incremental triggering could surface more warnings than the replay review justified

Mitigation:

- preserve cooldown and debounce semantics
- require a human review gate after bounded live proof

### Risk 5: The live proof looks more integrated than it is

Risk:

- a file-backed or harness-driven live demo may be mistaken for full runtime readiness

Mitigation:

- document the proof as sentinel-local only
- end the slice with a separate gate before any shell/world integration

## Verification Checkpoints

### Checkpoint L-A: Live Contract Is Stable

Verify:

- live event ordering, cursor behavior, and duplicate handling are deterministic

Evidence:

- `live_input` tests
- fixture or harness coverage for append-only and out-of-order cases

### Checkpoint L-B: Analyzer Contract Is Sufficient

Verify:

- the live runtime can produce warnings using existing analyzer checkpoints without adding new
  analyzer-owned semantics

Evidence:

- targeted compatibility tests
- explicit failure escalation if the checkpoint contract is insufficient

### Checkpoint L-C: Live Runtime Reuses Replay Core

Verify:

- the runtime uses shared scheduler and checkpoint presentation logic rather than live-only forks

Evidence:

- unit tests on shared state transitions
- review of public library seams

### Checkpoint L-D: Bounded Live Proof Is Useful

Verify:

- a fixture-driven live stream produces stable visible warnings, silent checkpoints, and heartbeat
  behavior without widening into runtime wiring

Evidence:

- `live_end_to_end` tests
- if a thin binary proof exists, a single documented command against repo-local fixtures

### Checkpoint L-E: Runtime Integration Stays Deferred

Verify:

- no shell/world integration begins after the live proof without another explicit approval

Evidence:

- task gate recorded as `always-check-with-user`
- no touched files outside the sentinel crate and the relevant spec docs

## Verification Wall Before Broader Runtime Integration

All of the following must pass before any later runtime-integration slice is even proposed:

1. the live contract proves sufficient without analyzer redesign
2. live runtime output is stable under fixture-driven incremental streams
3. replay behavior still passes unchanged
4. operator noise is reviewed again by a human
5. the post-slice gate is explicitly re-approved

If any of those fail, stop at the sentinel-local seam and revise the docs before widening scope.

## Exit Criteria

The plan is complete when:

1. the sentinel has a bounded live runtime seam at the library level
2. the live seam is verified by fixture-driven or harness-driven tests
3. the replay seam stays fixed
4. operator output remains evidence-backed and conservative
5. shell/world integration is still deferred behind a later approval gate
