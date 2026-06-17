# Plan: Agent Drift Sentinel Real Session Progress v0.5B

## Scope

This plan implements
[agent-drift-sentinel-real-session-progress-v0.5B-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-spec.md:1).

The goal is to deepen the real-session live coordinator so restart continuity is owned by an
explicit progress seam rather than mixed coordinator fields.

This slice should:

- add one internal progress/state seam for `real_session_live.rs`
- persist and restore enough progress to resume deterministically after restart
- keep checkpoint delta delivery stable across restarts
- preserve the existing single-session CLI path and posture/analyzer semantics

This slice should not:

- widen analyzer checkpoint schema
- change posture presentation behavior
- redesign scheduler cooldown/debounce behavior
- add multi-session live coordination
- broaden into shell/world/shim integration

Execution split:

- `v0.5B.1`: extract and persist the live-progress seam
- `v0.5B.2`: restart behavior hardening, regression coverage, and continuity-doc refresh

## Why This Slice Comes Next

The repo is at the right seam for this packet:

- `v0.5` established the real-session live path
- `v0.5A` handled downstream posture honesty separately
- the remaining live-specific gap is now coordinator shape and restart continuity
- `real_session_live.rs` still owns source progress, delivery progress, and persistence inside one
  type
- persisted continuity still restores only the cursor, while other progress facts reset on
  restart

That makes `v0.5B` the clean follow-up packet after posture.

## Implementation Strategy

Build `v0.5B` in two focused sub-packets:

1. `v0.5B.1`: define one internal live-progress contract and persisted-state shape, then route
   coordinator persistence through that seam, including the unchanged-rollout restore decision
   that depends on persisted source progress
2. `v0.5B.2`: preserve monotonic delivery ordering across appended restart and lock the broader
   regressions plus continuity-proof guidance

## Major Components

### 1. Progress-State Contract

Deliver first:

- add `LiveSessionProgress`
- add a persisted wrapper for schema/session-scoped live progress
- define which progress facts are continuity-critical

Why first:

- everything else in this packet depends on a durable notion of “how far the live session has
  already progressed”

### 2. Restart-Aware Poll Decisions

Deliver second:

- use persisted source progress to decide whether an unchanged rollout needs a rerun
- keep interrupted-poll state explicit so unchanged restart cannot skip still-undelivered
  checkpoints from previously observed growth
- keep shrink detection and sparse-startup handling intact

Why second:

- restart continuity is not only about duplicate checkpoint suppression; it also governs when the
  coordinator should act at all

### 3. Delta Delivery And Emission Continuity

Deliver third:

- route fresh-checkpoint filtering through the progress seam
- persist `next_emission_ordinal` continuity
- keep already-delivered checkpoints from reappearing after restart

Why third:

- the delivery stream should feel resumed, not freshly re-created, after a process restart

### 4. Regression Coverage And Proof Refresh

Deliver fourth:

- extend restart tests to prove unchanged-restart and appended-growth behavior explicitly
- preserve fail-closed state-mismatch coverage
- refresh the smoke guide if it still underspecifies restart expectations

Why fourth:

- this packet is about continuity guarantees, so the tests and proof guidance are part of the
  contract

## Sequencing

Sub-packet `v0.5B.1`:

1. add the progress/state contract
2. route persisted-state loading/saving through that seam
3. use restored source progress for unchanged-rollout idle decisions without treating partially
   drained growth as idle

Sub-packet `v0.5B.2`:

4. route checkpoint delta and appended-restart emission-order continuity through that seam
5. land restart regressions and refresh docs

Parallel-safe work after the contract is locked:

- restart-specific test authoring can proceed in parallel with smoke-guide wording updates

## Risks And Mitigations

### Risk 1: Progress extraction becomes a cosmetic refactor only

Mitigation:

- make rerun decisions, checkpoint filtering, and emission ordering all consume the same progress
  seam

### Risk 2: Restart continuity changes accidentally alter posture behavior

Mitigation:

- keep posture and sink semantics out of scope for this packet
- validate via existing live-end-to-end and real-session tests

### Risk 3: Persisted progress becomes under-specified again

Mitigation:

- explicitly persist source progress and delivery progress, not just one cursor field
- test restart against unchanged and appended rollout states separately

### Risk 4: Invalid persisted state silently resets progress

Mitigation:

- preserve current fail-closed behavior for session/schema mismatches
- extend progress-state parsing/validation through the same error posture

## Verification Wall

`v0.5B.1` minimum wall:

```bash
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
```

`v0.5B.2` completion wall:

```bash
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Required bounded live proof:

1. identify an actually active `SESSION_ID`
2. run the bounded live sentinel command against that session
3. confirm the source rollout is genuinely growing while the sentinel runs
4. confirm the restart-continuity seam still behaves correctly without reopening posture or
   analyzer semantics
