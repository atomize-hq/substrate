# Spec: Agent Drift Sentinel Real Session Progress v0.5B

## Assumptions I'm Making

1. The bounded real-session live packet in
   `docs/specs/agent-drift-sentinel-real-session-live-v0.5-*.md` is already landed and remains
   the correct outer contract for single-session live monitoring.
2. The sentinel posture packet `v0.5A` is already landed, so `v0.5B` must not reopen posture
   semantics in `operator_surface.rs` or `operator_sink.rs`.
3. `v0.5B` is a coordinator-only deepening slice inside
   `crates/agent-drift-sentinel/src/real_session_live.rs`.
4. Analyzer semantics, replay presentation semantics, and single-session live CLI semantics should
   remain unchanged in this slice.
5. The current continuity seam is still shallow:
   - `last_delivered_cursor` is persisted
   - `last_observed_size_bytes` is in-memory only
   - `next_emission_ordinal` is in-memory only
   - poll, rerun, delta filtering, emission, and persistence all live inside one coordinator type
6. The correct next move is to deepen progress/state ownership first rather than changing analyzer
   output, scheduler policy, or posture presentation.
7. For execution focus, `v0.5B` should land as two reviewable sub-packets:
   - `v0.5B.1`: extract and persist the live-progress seam
   - `v0.5B.2`: restart behavior hardening, regression coverage, and continuity-doc refresh

## Objective

Deepen the real-session live coordinator so one internal progress module owns restart continuity.

Primary user:

- the engineer or operator running `agent-drift-sentinel --mode live` and expecting a process
  restart to resume cleanly without replaying already delivered checkpoints or regressing progress
  state

Success means:

- the coordinator has one explicit progress/state seam
- restart continuity is restored from persisted progress, not reconstructed ad hoc from mixed
  coordinator fields
- unchanged live sources do not look like a fresh session after process restart
- appended growth after restart emits only genuinely newer checkpoints with stable progress
  ordering
- posture and analyzer semantics remain unchanged

## Packet Boundary

This packet is **real-session progress ownership and restart continuity hardening**.

Execution split:

- `v0.5B.1`: lock the shared progress/state contract, route persistence through it, and restore
  unchanged-rollout source progress without skipping partially drained growth after an interrupted
  poll
- `v0.5B.2`: harden appended-growth/emission-order restart behavior and complete
  regression/proof refresh

In scope:

- add a sentinel-owned internal progress/state seam for the live coordinator
- persist and restore enough coordinator progress to resume deterministically after restart
- make restart continuity fail closed when persisted state is invalid or mismatched
- keep the single-session live path library-first and bounded
- add regression coverage for unchanged-restart and appended-growth behavior

Out of scope:

- analyzer scoring or checkpoint schema changes
- replay-mode changes
- posture model changes
- persisting full `LiveRuntime` scheduler state across restarts
- multi-session orchestration
- shell/world/shim integration

## Problem Statement

The current `real_session_live.rs` seam works, but it is still structurally shallow.

Today `LiveSessionCoordinator` directly owns:

- rollout discovery
- persisted-cursor loading
- rollout size checks
- sparse-startup retry rules
- compactor/analyzer reruns
- checkpoint delta filtering
- emission ordinal assignment
- persisted-state writes

That keeps the live path functioning, but it blurs two different responsibilities:

1. **coordination**
   - find the live source
   - rerun upstream pipeline work
   - feed the runtime
2. **progress ownership**
   - decide whether rerun is needed
   - remember how far the source and delivery stream have advanced
   - resume that state after restart

The current persisted state also restores only part of continuity:

- `last_delivered_cursor` survives restart
- `last_observed_size_bytes` does not
- `next_emission_ordinal` does not

That means restart behavior is correct enough to prevent replay, but still incomplete as a progress
model. A fresh coordinator instance still behaves like a partial reset.

## Tech Stack

- Language: Rust 2021
- Target crate: `crates/agent-drift-sentinel`
- Upstream library crates already in use:
  - `crates/agent-session-compactor`
  - `crates/agent-drift-analyzer`
- Existing live seam:
  - `src/real_session_live.rs`
  - `src/cli.rs`
  - `tests/real_session_live.rs`

Dependency posture:

- no new crate is required
- no analyzer change is required
- no scheduler redesign is required

## Commands

Targeted sentinel validation:

```bash
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Bounded live proof shape after implementation:

```bash
export SESSION_ID="<active-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"

sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```

That proof is only valid if the targeted rollout file is actively growing while the sentinel is
running.

## Project Structure

```text
crates/agent-drift-sentinel/src/real_session_live.rs
  Current coordinator seam for live polling, pipeline reruns, delta delivery, and persisted state.

crates/agent-drift-sentinel/src/cli.rs
  Bounded `--mode live` entrypoint that exercises the coordinator seam.

crates/agent-drift-sentinel/tests/real_session_live.rs
  Current regression coverage for append-only growth, restart continuity, sparse startup, and
  invalid persisted state.

docs/specs/agent-drift-sentinel-real-session-live-v0.5-*.md
  Existing outer contract for single-session real-session live behavior.

docs/specs/agent-drift-sentinel-real-session-progress-v0.5B-*.md
  New follow-up spec, plan, and task chain for coordinator deepening.

docs/internals/testing/hybrid-drift-stack-smoke-guide.md
  Bounded replay/live proof guidance that should reflect final restart-continuity expectations.
```

## Code Style

Keep progress ownership explicit and separate from pipeline orchestration.

```rust
pub struct LiveSessionProgress {
    pub last_observed_size_bytes: Option<u64>,
    pub pending_observed_size_bytes: Option<u64>,
    pub last_delivered_cursor: Option<CheckpointCursor>,
    pub next_emission_ordinal: usize,
}
```

Conventions:

- coordinator methods should ask a progress/state seam what to do rather than recomputing progress
  rules ad hoc
- persist progress atomically and fail closed on mismatched session or schema state
- keep progress state single-session and bounded to the current `state_dir`
- preserve analyzer checkpoints as the only semantic input to the live runtime
- keep restart continuity deterministic before adding broader orchestration

## Testing Strategy

Required test layers:

1. Progress-state restoration tests
   - prove persisted progress restores the same session boundary and continuity state
2. Restart continuity tests
   - restart with unchanged rollout should not emit already-delivered checkpoints
   - appended growth after restart should emit only newer ordinals
3. Fail-closed persisted-state tests
   - mismatched session or schema should still stop immediately
4. Bounded live proof
   - a real active session should still run through the existing live CLI path with the hardened
     continuity seam

## Design Summary

Add one internal live-progress seam that owns restart continuity:

```rust
pub struct LiveSessionProgress {
    pub last_observed_size_bytes: Option<u64>,
    pub pending_observed_size_bytes: Option<u64>,
    pub last_delivered_cursor: Option<CheckpointCursor>,
    pub next_emission_ordinal: usize,
}

pub struct PersistedLiveSessionProgress {
    pub schema_version: u32,
    pub session_id: String,
    pub progress: LiveSessionProgress,
}
```

The coordinator flow becomes:

1. resolve the target rollout artifact
2. load persisted progress for the session
3. decide from progress plus current source size whether a rerun is needed, treating an
   interrupted poll as pending until that observed size is fully drained
4. rerun compactor/analyzer only when required
5. filter checkpoints through the progress seam
6. emit observations using persisted `next_emission_ordinal` continuity
7. persist the advanced progress state atomically

The deepening is not “change the live contract.” The deepening is “make one module own progress
and restart continuity so the coordinator stops behaving like a partial reset after process
restart.”

## Boundaries

- Always:
  - keep `v0.5B` scoped to `real_session_live.rs`, its direct tests, and any continuity-proof docs
  - preserve single-session live semantics and existing CLI flags
  - preserve posture and analyzer semantics
  - persist enough progress to resume deterministically after restart
- Ask first:
  - widening persisted state beyond progress ownership into scheduler-state snapshots
  - changing the live CLI contract
  - broadening the slice into multi-session coordination
- Never:
  - fold `v0.5B` back into posture work
  - re-score raw rollout rows in the sentinel
  - silently treat mismatched persisted state as a clean reset

## Success Criteria

This slice is complete only when all of the following are true:

1. The live coordinator has a dedicated internal progress/state seam.
2. Persisted restart continuity includes more than just `last_delivered_cursor`.
3. Restart against an unchanged rollout does not emit previously delivered checkpoints.
4. Appended growth after restart emits only checkpoints newer than the persisted cursor.
5. Emission ordering remains monotonic across restart rather than resetting to a fresh local
   sequence.
6. Invalid or mismatched persisted state still fails closed.
7. No posture or analyzer contract change is required.
8. The bounded real-session live proof still works through the existing CLI path.

## Open Questions

1. Should unchanged restart avoid only duplicate emission, or also avoid an unnecessary immediate
   pipeline rerun when the rollout size has not changed?
   - Assumed default for this spec: avoid the rerun when persisted source progress proves the
     rollout is unchanged.
2. Should `LiveRuntime` scheduler state itself be persisted in this packet?
   - Assumed default for this spec: no; `v0.5B` owns coordinator progress continuity, not full
     scheduler-state snapshotting.
