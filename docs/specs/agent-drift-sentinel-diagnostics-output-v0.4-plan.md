# Plan: Agent Drift Sentinel Diagnostics Output v0.4

## Scope

This plan implements
[agent-drift-sentinel-diagnostics-output-v0.4-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-spec.md:1).

The goal is to expose analyzer checkpoint diagnostics through the sentinel’s existing replay and
live output surfaces without changing scheduler behavior.

This slice should:

- surface a compact diagnostics summary in replay output
- expose the same diagnostics contract in structured live sink events
- keep replay/live presentation aligned
- preserve all current scheduler semantics

This slice should not:

- change when warnings fire
- change warning fingerprinting
- enable live mode against real active session files
- widen analyzer or compactor contracts

## Why This Slice Comes Next

The current repo state makes this the cleanest next packet:

- analyzer diagnostics are already present in the shared checkpoint type and covered by analyzer
  tests
- sentinel compatibility is restored, but `diagnostics` remains mostly invisible downstream
- the operator-facing replay report still forces the user to infer cadence/context from drift
  scores and sparse evidence lines alone
- the live sink exists, but it does not yet provide a durable diagnostics contract for downstream
  consumers

This makes operator-facing diagnostics the highest-value, lowest-risk next step before any broader
real-session live work.

## Implementation Strategy

Implement this slice from shared data shape outward:

1. define a compact diagnostics summary contract shared by replay and live outputs
2. render that summary in replay checkpoint presentation
3. attach the same summary to live sink events
4. add alignment and formatting regression coverage
5. verify the full crate without any scheduler diffs

## Major Components

### 1. Shared Diagnostics Summary Contract

Deliver first:

- one sentinel-local diagnostics summary shape derived from `Checkpoint.diagnostics`
- explicit verification-density formatting from the interval counters
- one stable field set reused by replay and live outputs

Why first:

- this prevents replay/live drift and keeps later real-session work from inventing a second
  diagnostics contract

### 2. Replay Presentation

Deliver second:

- add a compact diagnostics line or block to `CheckpointPresentation`
- surface task-frame transition, working-set change, verification density/counts, and evidence-item
  count

Why second:

- replay output is the easiest operator surface to review manually and proves the wording before
  live events depend on it

### 3. Live Sink Exposure

Deliver third:

- carry the diagnostics summary through `OperatorEvent` payloads
- preserve the existing visible/silent/status split

Why third:

- live outputs should not invent a separate schema once replay wording is settled

### 4. Regression Coverage

Deliver fourth:

- replay formatting tests
- live sink event tests
- alignment tests that compare shared checkpoint facts across replay/live

Why fourth:

- the whole point of this slice is to freeze a durable output contract

## Risks And Mitigations

### Risk 1: Diagnostics output becomes noisy

Mitigation:

- keep the summary compact and numeric
- avoid duplicating evidence text or full task-frame detail

### Risk 2: Replay and live output drift apart

Mitigation:

- derive both from one shared diagnostics summary shape
- add alignment tests over the same checkpoint fixture

### Risk 3: Scheduler changes sneak into presentation work

Mitigation:

- keep `scheduler.rs` untouched in this slice
- explicitly verify that visible-warning counts do not change under existing fixtures

## Verification Wall

This slice is only complete when all of the following pass:

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Bounded manual proof:

- run the replay CLI against a current analyzer bundle and confirm the rendered output now explains
  why a warning fired without changing which warnings appear
