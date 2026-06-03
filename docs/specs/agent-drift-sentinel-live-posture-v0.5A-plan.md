# Plan: Agent Drift Sentinel Live Posture v0.5A

## Scope

This plan implements
[agent-drift-sentinel-live-posture-v0.5A-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-live-posture-v0.5A-spec.md:1).

The goal is to deepen the sentinel operator surface so replay and live outputs can distinguish
current drift, newly recovered drift, and retained historical drift without changing analyzer
semantics or real-session coordinator behavior.

This slice should:

- add a sentinel-owned checkpoint posture contract
- derive `active`, `recovered`, and `historical-only` from existing analyzer checkpoint output
- thread posture through replay presentation and checkpoint-bearing sink events
- preserve replay/live parity for the same ordered checkpoint sequence
- keep scheduler behavior and `real_session_live.rs` restart semantics unchanged

This slice should not:

- widen analyzer checkpoint schema
- redesign scheduler cooldown, debounce, or fingerprinting
- rename existing sink event families
- refactor `real_session_live.rs` coordinator persistence or restart continuity
- broaden into shell/world/shim integration

Execution split:

- `v0.5A.1`: posture contract plus replay presentation
- `v0.5A.2`: sink event wiring, replay/live parity, and real-session regression refresh

## Why This Slice Comes Next

The repo is at a clean seam for this packet:

- analyzer `v0.4` already distinguishes active versus historical evidence downstream, but sentinel
  presentation still collapses that into `Visible` vs `Silent`
- `operator_surface.rs` and `operator_sink.rs` are intentionally downstream-only and are the
  narrowest place to fix operator honesty
- the live coordinator already emits ordered checkpoints; the next missing contract is presentation
  semantics, not pipeline or cursor behavior
- keeping posture separate from `v0.5B` preserves review clarity by isolating presentation bugs
  from restart/cursor bugs

That makes sentinel posture the highest-leverage next packet before deeper coordinator work.

## Implementation Strategy

Build `v0.5A` in two focused sub-packets:

1. `v0.5A.1`: add one sentinel-local posture contract and detection helpers, then teach replay
   presentation to compute and render posture separately from visibility
2. `v0.5A.2`: carry that posture through checkpoint-bearing live sink events, lock replay/live
   parity tests, and prove the bounded live path can surface posture transitions without changing
   restart semantics

## Major Components

### 1. Shared Posture Contract And Detection

Deliver first:

- add a sentinel-local `CheckpointPosture` enum
- add one centralized historical-evidence detector for the currently supported analyzer prefixes
- add one centralized classifier that uses:
  - the current checkpoint
  - the immediately previous checkpoint for the same session
  - checkpoint/drift-score flagged state

Why first:

- this prevents replay and live code paths from inventing competing posture rules

### 2. Replay Presentation Deepening

Deliver second:

- extend `CheckpointPresentation` to carry posture
- compute posture inside replay rendering without altering warning visibility decisions
- expose posture in replay console output in a way that keeps `warning` / `checkpoint` labels
  stable

Why second:

- replay output is the easiest place to validate wording and precedence before live event payloads
  depend on it

### 3. Live Sink Exposure

Deliver third:

- add posture to `VisibleWarningEvent` and `SilentCheckpointEvent`
- keep `OperatorEvent` families stable
- preserve existing heartbeat/status behavior unless a targeted clarity gap requires additional
  wording

Why third:

- once replay semantics are locked, live checkpoint-bearing events should reuse them rather than
  inventing a second contract

### 4. Replay/Live Parity Coverage

Deliver fourth:

- add unit coverage for posture precedence and historical-evidence detection
- add sink tests proving visible and silent checkpoint events carry posture correctly
- extend replay/live parity tests so the same checkpoint sequences produce the same posture

Why fourth:

- this slice is only durable if posture is frozen as a shared downstream contract

### 5. Bounded Live Proof Refresh

Deliver fifth:

- run the existing bounded live path against an actually active session
- verify posture transitions can appear in live output without any restart-continuity changes
- refresh the smoke guide wording if it still describes the old binary presentation

Why fifth:

- the slice is user-facing and needs proof on the real live path, not only on fixtures

## Sequencing

Sub-packet `v0.5A.1`:

1. add posture type and detection helpers
2. wire posture into replay presentation

Sub-packet `v0.5A.2`:

3. wire posture into checkpoint-bearing sink events
4. land unit and parity regressions
5. run the bounded live proof and refresh docs

Parallel-safe work after the contract is locked:

- replay rendering tests can be authored in parallel with sink payload tests
- smoke-guide wording can be prepared in parallel once the rendered posture text is settled

## Risks And Mitigations

### Risk 1: Posture drifts back into visibility semantics

Mitigation:

- keep `WarningDisposition` unchanged and visibility-only
- add posture as separate structured data on presentation and event payloads

### Risk 2: Historical detection becomes brittle or duplicated

Mitigation:

- centralize historical-prefix detection in one sentinel helper
- keep prefix expansion additive and local to the posture seam

### Risk 3: `Recovered` and `HistoricalOnly` become ambiguous

Mitigation:

- make `Recovered` explicitly transition-aware using the immediately previous checkpoint
- reserve `HistoricalOnly` for non-active checkpoints with historical evidence where that
  transition rule does not apply

### Risk 4: Replay and live outputs diverge

Mitigation:

- classify posture once from shared ordered-checkpoint semantics
- extend parity tests across replay and live fixtures before considering the packet done

### Risk 5: The slice expands into `v0.5B`

Mitigation:

- keep `real_session_live.rs` behaviorally unchanged except where posture-bearing outputs need to
  pass through existing observations
- reject restart persistence, cursor-state, and coordinator-shape changes from this packet

## Verification Wall

`v0.5A.1` minimum wall:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
```

`v0.5A.2` completion wall:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Required bounded live proof:

1. identify an actually active `SESSION_ID`
2. run the bounded live sentinel command against that session
3. confirm the source rollout is genuinely growing while the sentinel runs
4. record that live checkpoint output can show `active`, `recovered`, or `historical-only`
   posture without any restart-continuity changes
