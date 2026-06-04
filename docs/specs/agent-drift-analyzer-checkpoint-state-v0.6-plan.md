# Plan: Agent Drift Analyzer Checkpoint State v0.6

## Scope

Implementation status on `2026-06-04`:

- Packet `v0.6A` is the analyzer-only landing for exported `DriftState`, the centralized
  analyzer-owned state builder, and checkpoint schema `v0.3`
- Packet `v0.6B` stays reserved for sentinel posture cutover, `v0.2` compatibility handling, and
  replay/live proof refresh
- typed outcome evidence remains outside this packet family

This plan implements `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`.

The goal is to move checkpoint-state ownership into `agent-drift-analyzer`, widen checkpoint
export additively, and cut sentinel posture over to that exported state without mixing in the
separate typed-outcome evidence redesign.

This slice should:

- add analyzer-owned per-class `DriftState`
- construct that state from `CheckpointAnalysis`
- export `DriftState` on every `DriftScore`
- bump checkpoint schema version from `v0.2` to `v0.3`
- make sentinel consume exported state first, with `v0.2` fallback preserved
- revalidate replay/live parity and the bounded live proof

This slice should not:

- retune drift thresholds or rename drift classes
- change scheduler behavior or sink families
- reopen real-session progress persistence work
- redefine failure evidence typing

Execution split:

- `v0.6A`: analyzer contract and checkpoint export
- `v0.6B`: sentinel cutover, compatibility, and proof

## Why This Slice Comes Next

The live stack now makes the seam problem explicit:

- analyzer semantics and sentinel posture both landed, but the ownership boundary stayed shallow
- `operator_surface.rs` still reconstructs state from `flagged`, historical reason prefixes, and
  the previous checkpoint
- that means analyzer truth is still partly encoded in sentinel control flow instead of one module
  owning it
- the architecture review's top recommendation is to deepen checkpoint-state first and only then
  tackle typed outcome evidence

That makes checkpoint-state ownership the highest-leverage next packet. It removes the current
cross-module semantic leak without broadening into the distinct `ToolOutput` evidence seam.

## Implementation Strategy

Build `v0.6` in two focused sub-packets:

1. `v0.6A`: define the analyzer-owned state contract, compute it from `CheckpointAnalysis`, and
   export it additively on `DriftScore`
2. `v0.6B`: teach sentinel to consume exported state, preserve `v0.2` fallback, and rerun the
   replay/live proof wall

This sequence keeps the risky behavior change behind one analyzer contract instead of spreading it
incrementally across sentinel heuristics.

## Major Components

### 1. Analyzer-Owned DriftState Contract

Deliver first:

- add a new exported enum for per-class state
- decide which drift classes support which states
- define one analyzer-internal builder that maps `CheckpointAnalysis` plus scorer outputs into
  `DriftState`

Why first:

- everything else depends on having one semantic owner for checkpoint state

### 2. Analyzer Export Widening

Deliver second:

- add `state` to exported `DriftScore`
- bump checkpoint `schema_version` to `v0.3`
- keep serialized compatibility explicit and testable

Why second:

- the whole point of this packet is to stop encoding state through sentinel-local inference

### 3. Sentinel Current-State Cutover

Deliver third:

- derive `CheckpointPosture` from current-checkpoint `DriftState` values
- keep `v0.2` fallback isolated behind compatibility helpers
- remove reason-prefix parsing from the `v0.3` path

Why third:

- once analyzer exports explicit state, sentinel should become presentation-only for posture

### 4. Replay/Live Proof Refresh

Deliver fourth:

- rerun replay/live parity tests
- rerun real-session compatibility coverage
- refresh continuity notes in the packet authority doc

Why fourth:

- this slice is only durable if the cutover works on both replay and live code paths

## Sequencing

Sub-packet `v0.6A`:

1. lock the docs and packet boundary
2. add `DriftState` plus analyzer-owned state builder
3. widen checkpoint export and analyzer tests

Sub-packet `v0.6B`:

4. cut sentinel posture to explicit analyzer state
5. keep `v0.2` fallback green
6. rerun replay/live proof wall and refresh continuity docs

Parallel-safe work after the contract is locked:

- analyzer export tests can be authored in parallel with sentinel compatibility tests
- continuity-note updates can be drafted while the proof wall runs, once the final schema version
  and fallback posture are settled

## Risks And Mitigations

### Risk 1: DriftState becomes a thin alias for `flagged`

Mitigation:

- define state per class from analyzer semantics, not just score thresholds
- require tests for `recovered` and `historical_only`, not only `active`

### Risk 2: Schema widening breaks downstream readers

Mitigation:

- bump `schema_version` explicitly
- keep sentinel dual-read support for `v0.2` and `v0.3`
- refresh replay/live compatibility tests in the same packet

### Risk 3: Sentinel still reconstructs state in hidden fallback branches

Mitigation:

- isolate `v0.2` fallback into compatibility helpers
- make the `v0.3` path state-first and current-checkpoint-only

### Risk 4: The packet drifts into the typed-outcome evidence seam

Mitigation:

- keep `ToolOutput` classification unchanged in this packet
- record typed outcome evidence as the next planned follow-on, not implementation scope

### Risk 5: WrongPlanBranch semantics become muddled

Mitigation:

- keep `WrongPlanBranch` recovery semantics narrow
- prefer `active` / `cleared` only unless a concrete analyzer rule supports historical labeling

## Verification Wall

`v0.6A` minimum wall:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
```

`v0.6B` completion wall:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Required bounded live proof:

1. identify an actually active `SESSION_ID`
2. run the bounded live sentinel command against that session
3. confirm the source rollout is genuinely growing while the sentinel runs
4. confirm posture transitions are still rendered honestly after the analyzer-state cutover

## Verification Checkpoints

### Checkpoint VS-A: Analyzer Owns State

Must be true:

- `DriftState` exists in analyzer schema
- one analyzer-owned builder maps checkpoint analysis into exported state
- `schema_version` is `v0.3`

Verify:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
```

### Checkpoint VS-B: Sentinel Is Presentation-Only For v0.3

Must be true:

- sentinel derives posture from current-checkpoint state when `v0.3` checkpoints are present
- the `v0.3` path does not depend on historical reason-prefix parsing

Verify:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
```

### Checkpoint VS-C: Compatibility And Proof Stay Green

Must be true:

- sentinel still accepts `v0.2`
- replay/live parity stays aligned
- bounded live proof remains honest

Verify:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```
