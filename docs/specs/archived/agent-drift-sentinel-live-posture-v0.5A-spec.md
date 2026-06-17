# Spec: Agent Drift Sentinel Live Posture v0.5A

## Assumptions I'm Making

1. This is a sentinel-only presentation slice on top of the already-landed analyzer `v0.4`
   semantics and the bounded real-session live `v0.5` coordinator seam.
2. `v0.5A` must not widen the analyzer `Checkpoint` schema first; it should derive posture from:
   - existing checkpoint-level `flagged` state
   - existing drift-score `flagged` state
   - existing historical evidence reason prefixes already emitted by the analyzer
   - the previous checkpoint for the same session as seen by the sentinel
3. Posture is a sentinel-owned semantic layer and must remain separate from visibility:
   `WarningDisposition` answers "was this surfaced visibly right now?" while posture answers
   "what is the session's drift state right now?"
4. The posture contract should remain sequence-aware but checkpoint-scoped:
   `active`, `recovered`, and `historical-only` are derived for one presented checkpoint using that
   checkpoint plus the immediately previous checkpoint in the same ordered session stream.
5. The next packet, `v0.5B`, remains separate and continues to own real-session progress,
   restart continuity, persisted cursor behavior, and coordinator shape in
   `crates/agent-drift-sentinel/src/real_session_live.rs`.
6. This slice should preserve the existing event families in `operator_sink.rs` and add posture
   as structured data on those existing events rather than introducing a larger new event taxonomy.
7. For execution focus, `v0.5A` should land as two reviewable sub-packets:
   - `v0.5A.1`: posture contract plus replay presentation
   - `v0.5A.2`: sink event wiring, replay/live parity, and real-session regression refresh

## Objective

Deepen the sentinel operator surface so replay and live presentation can describe checkpoint
posture honestly.

Primary user:

- the engineer or operator reading replay or live sentinel output and needing to distinguish:
  - a currently active drift warning
  - a checkpoint that just recovered from earlier drift
  - a checkpoint that still carries historical drift evidence but is no longer the recovery
    transition

Success means:

- the sentinel derives a checkpoint posture independently from visibility
- replay and live surfaces can represent `active`, `recovered`, and `historical-only`
- the same checkpoint can still be visually silent while carrying meaningful posture
- analyzer semantics remain unchanged in this slice
- real-session restart continuity remains unchanged in this slice

## Packet Boundary

This packet is **sentinel posture presentation only**.

Execution split:

- `v0.5A.1`: lock the shared posture classifier and replay presentation surface
- `v0.5A.2`: carry posture through live sink events and complete replay/live/regression proof

In scope:

- add a sentinel-owned posture type
- derive posture from existing analyzer checkpoint output plus prior checkpoint context
- thread posture through `operator_surface.rs` and `operator_sink.rs`
- preserve replay/live parity for posture output
- update targeted docs and tests for the new posture contract

Out of scope:

- analyzer scoring or schema changes
- scheduler cooldown or debounce redesign
- new sink families or transport protocols
- restart continuity, persisted cursor, or coordinator refactors in `real_session_live.rs`
- shell/world/shim integration

## Problem Statement

The current sentinel public model is semantically shallow:

- `WarningDisposition` is binary: `Visible` or `Silent`
- console output labels are binary: `warning` or `checkpoint`
- sink events are binary at the checkpoint layer: `VisibleWarning` or `SilentCheckpoint`

That binary model was acceptable before the analyzer could preserve historical evidence while
clearing active flags. It is now misleading.

Today, the analyzer already emits enough signal to distinguish more honest states:

- `checkpoint.flagged` and `drift_score.flagged` identify active drift
- `TruthGroundingGap` can preserve historical evidence using
  `historical truth-grounding gap: ...`
- `DeadEndThrash` can preserve historical evidence using
  `historical repeated failure evidence: ...` and
  `historical repeated verification evidence: ...`

What the sentinel still lacks is a downstream contract for presenting that state honestly.

## Tech Stack

- Language: Rust 2021
- Target crate: `crates/agent-drift-sentinel`
- Upstream semantic input: `crates/agent-drift-analyzer`
- Shared runtime modules already in place:
  - `src/operator_surface.rs`
  - `src/operator_sink.rs`
  - `src/live_runtime.rs`
  - `src/live_input.rs`
  - `src/real_session_live.rs`

Dependency posture:

- no analyzer crate change is required first
- no scheduler change is required first
- no new crate or external dependency is required

## Commands

Targeted sentinel validation:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Analyzer compatibility validation:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
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
crates/agent-drift-sentinel/src/operator_surface.rs
  Owns checkpoint presentation, replay report rendering, and checkpoint classification.

crates/agent-drift-sentinel/src/operator_sink.rs
  Owns structured operator event types and mapping from live observations into sink events.

crates/agent-drift-sentinel/src/live_runtime.rs
  Owns ordered live checkpoint observation and scheduler state used before sink emission.

crates/agent-drift-sentinel/src/live_input.rs
  Supplies live checkpoint events and compatibility helpers for replay/live parity tests.

crates/agent-drift-sentinel/src/real_session_live.rs
  Owns the real-session coordinator; remains structurally unchanged in `v0.5A`.

crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/operator_sink.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/real_session_live.rs
  Regression coverage for replay/live posture presentation and structured sink output.

docs/internals/testing/hybrid-drift-stack-smoke-guide.md
  Bounded replay/live proof guidance to refresh after posture lands.
```

## Code Style

Keep posture separate from visibility and make posture precedence explicit in the sentinel layer.

```rust
pub enum CheckpointPosture {
    Active,
    Recovered,
    HistoricalOnly,
}

pub struct CheckpointPresentation {
    pub posture: Option<CheckpointPosture>,
    pub disposition: WarningDisposition,
    // existing fields continue here
}
```

Conventions:

- `WarningDisposition` remains visibility-only; do not overload it with posture semantics
- derive posture from checkpoint sequence plus analyzer evidence, not from raw rollout rows
- keep sink event families stable and add posture fields to those existing event payloads
- use checkpoint-level posture precedence:
  - `active` if any drift class is currently flagged
  - else `recovered` if a drift class was active in the immediately previous checkpoint and is now
    present only as historical evidence
  - else `historical-only` if historical evidence is present without an active-to-recovered
    transition at this checkpoint
- represent no drift posture as `None`, not as a fourth public posture label

## Testing Strategy

Required test layers:

1. Operator-surface unit tests
   - verify checkpoint-level posture precedence
   - verify console rendering includes posture without collapsing back to `Visible` vs `Silent`
2. Operator-sink unit tests
   - verify `VisibleWarning` and `SilentCheckpoint` carry posture where applicable
   - verify heartbeat/status events preserve current behavior unless posture is intentionally added
3. Replay/live parity tests
   - verify matching checkpoints render the same posture in replay and live flows
4. Real-session live regression tests
   - verify the coordinator path can emit recovered or historical-only posture without changing
     cursor or restart semantics
5. Bounded live proof
   - verify an actually active session can show posture transitions using the existing real-session
     live command

## Design Summary

Add a new sentinel-owned posture layer:

```rust
pub enum CheckpointPosture {
    Active,
    Recovered,
    HistoricalOnly,
}
```

The sentinel flow becomes:

1. receive an ordered checkpoint stream exactly as today
2. classify visibility exactly as today
3. classify posture from:
   - the current checkpoint
   - the immediately previous checkpoint for the same session
   - existing analyzer historical-evidence reason prefixes
4. render replay and sink events with both visibility and posture

The key rule is that posture is not a synonym for visibility:

- a checkpoint can be `Visible + Active`
- a checkpoint can be `Silent + Recovered`
- a checkpoint can be `Silent + HistoricalOnly`
- a checkpoint with no active or historical drift posture remains `Silent + None`

## Posture Contract

### Active

`active` means the current checkpoint is making a current-state drift claim.

Rules:

- if `checkpoint.flagged` is true, posture is `Active`
- if any drift score is flagged, checkpoint posture is `Active`
- `Active` takes precedence over every other posture

### Recovered

`recovered` means the current checkpoint is the first non-active checkpoint after an active drift
state for the same session, and the current checkpoint still carries historical evidence for that
same class.

Rules:

- the current checkpoint is not active
- at least one current drift score contains historical evidence
- the immediately previous checkpoint in the sentinel-ordered stream had that drift class active

This is intentionally transition-aware. It marks "the system just recovered" rather than
"there exists historical evidence somewhere."

### HistoricalOnly

`historical-only` means the current checkpoint carries historical drift evidence, but this
checkpoint is not the active-to-recovered transition.

Rules:

- the current checkpoint is not active
- at least one current drift score contains historical evidence
- the `Recovered` rule does not apply

Typical examples:

- later checkpoints after the initial recovery checkpoint still preserve historical evidence
- replaying a checkpoint with historical evidence outside the precise recovery transition context

## Historical Evidence Detection

`v0.5A` should not widen analyzer schema first. The sentinel should detect historical evidence from
the existing analyzer output it already consumes.

Initial required prefixes:

- `historical truth-grounding gap:`
- `historical repeated failure evidence:`
- `historical repeated verification evidence:`

Design rule:

- historical-evidence detection must be centralized in sentinel code, not duplicated ad hoc in
  multiple tests or renderers
- expanding the supported prefix set later is additive and belongs in the sentinel compatibility
  seam unless analyzer schema is widened in a future packet

## Boundaries

- Always:
  - derive posture from analyzer checkpoints plus prior-checkpoint context only
  - preserve replay/live posture parity for the same ordered checkpoint sequence
  - keep `v0.5A` scoped to `operator_surface`, `operator_sink`, and directly related tests/docs
  - preserve scheduler behavior and existing coordinator persistence semantics
- Ask first:
  - widening analyzer checkpoint schema
  - renaming public sink event families
  - changing CLI contract or adding new top-level output modes
  - broadening the slice into `real_session_live.rs` coordinator refactors
- Never:
  - rescore raw rollout rows in the sentinel
  - hide historical evidence by stripping analyzer-provided reasons
  - combine posture presentation work with restart continuity hardening in the same packet

## Success Criteria

This slice is complete only when all of the following are true:

1. The sentinel has a dedicated posture contract for `active`, `recovered`, and
   `historical-only`.
2. `WarningDisposition` remains visibility-only and does not absorb posture semantics.
3. A checkpoint with any flagged drift score surfaces `Active`.
4. A checkpoint that follows an active checkpoint, is no longer flagged, and carries historical
   evidence surfaces `Recovered`.
5. A later checkpoint that still carries historical evidence without being the recovery transition
   surfaces `HistoricalOnly`.
6. A silent checkpoint with no historical evidence does not claim posture.
7. Replay and live tests prove parity for the same checkpoint sequences.
8. No analyzer schema change is required.
9. No `real_session_live.rs` restart/persistence change is required.

## Open Questions

1. Console rendering default: should posture appear as a dedicated line such as
   `- Posture: recovered`, or should the headline label itself change?
   - Assumed default for this spec: keep the existing `warning` / `checkpoint` label families and
     add posture as separate structured/rendered output.
2. Status and heartbeat events: should they mention posture text explicitly, or should posture stay
   only on checkpoint-bearing events?
   - Assumed default for this spec: checkpoint-bearing events must carry posture; heartbeat/status
     wording can remain unchanged unless tests show a clarity gap.
