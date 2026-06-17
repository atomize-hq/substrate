# Spec: Agent Drift Analyzer Checkpoint State v0.6

## Assumptions I'm Making

1. Live repo state is the authority here: analyzer `v0.4B/C` semantics and sentinel `v0.5A` /
   `v0.5B` posture/progress work are already landed in code, even if older task ledgers are stale.
2. This slice should deepen analyzer-owned checkpoint state first, then cut sentinel over to that
   exported state. It should not fold in the separate typed-outcome evidence follow-up.
3. The right downstream contract is additive checkpoint-schema widening, not another sentinel-local
   reconstruction layer. Sentinel should read explicit analyzer state when present.
4. Each checkpoint already exports one `DriftScore` per drift class, so per-class state belongs on
   `DriftScore` rather than in a second parallel map.
5. The exported state labels should be `active`, `recovered`, `historical_only`, and `cleared`.
   `cleared` is the analyzer-owned replacement for "no posture for this class at this checkpoint."
6. Widening the checkpoint contract should bump analyzer checkpoint `schema_version` from `v0.2`
   to `v0.3`, while sentinel remains backward-compatible with `v0.2` during the transition.

## Objective

Implementation status on `2026-06-04`:

- `v0.6A` lands the analyzer-owned `DriftState` contract, centralized state builder, and
  additive `checkpoints.jsonl` widening to checkpoint schema `v0.3`
- `v0.6B` remains the separate sentinel cutover / compatibility / proof packet
- typed outcome evidence remains a separate follow-on after `v0.6`

Deepen the checkpoint-state module in `agent-drift-analyzer` so the analyzer owns current versus
recovered versus historical-only versus cleared state for each drift class, exports that state in
`checkpoints.jsonl`, and lets `agent-drift-sentinel` become presentation-only for posture.

Primary users:

- the engineer or operator reading replay/live sentinel output and expecting checkpoint posture to
  reflect analyzer truth directly
- the maintainer changing analyzer semantics and needing one module to own checkpoint-state logic

Success means:

- analyzer constructs explicit per-drift-class state from `CheckpointAnalysis`
- exported checkpoints carry that state on every `DriftScore`
- sentinel stops using historical reason prefixes and previous-checkpoint heuristics when analyzer
  `v0.3` state is present
- replay/live posture remains `active`, `recovered`, or `historical-only` at the checkpoint level,
  but it is derived from analyzer-owned per-class state instead of sentinel reconstruction
- the `ToolOutput` evidence-classification fix remains a separate follow-on packet

## Packet Boundary

This packet is **checkpoint-state ownership and downstream cutover only**.

Execution split:

- `v0.6A`: analyzer-owned state contract, internal state builder, and additive checkpoint export
- `v0.6B`: sentinel cutover to exported state, compatibility fallback, and replay/live proof

In scope:

- add analyzer-owned per-class state semantics
- widen checkpoint export additively
- bump checkpoint schema version to `v0.3`
- preserve sentinel compatibility with existing `v0.2` checkpoints
- remove sentinel reliance on reason-prefix control flow when `v0.3` state is available
- refresh targeted docs and tests for the new contract

Out of scope:

- retyping `ToolOutput` / failure evidence
- changing analyzer drift classes or thresholds
- changing sentinel scheduler policy, cooldowns, or sink families
- reopening `real_session_live.rs` restart/progress work from `v0.5B`
- changing compactor bundle schema

## Tech Stack

- Language: Rust 2021
- Target crates:
  - `crates/agent-drift-analyzer`
  - `crates/agent-drift-sentinel`
- Upstream artifact contract:
  - compactor bundle `v0.2`
- Downstream checkpoint contract:
  - analyzer checkpoint schema `v0.2` today
  - analyzer checkpoint schema `v0.3` after this slice

Dependency posture:

- no compactor change is required first
- no sentinel scheduler change is required first
- no new crate or external dependency is required

## Commands

Targeted analyzer validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Targeted sentinel compatibility validation:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
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

That proof is only valid if the target rollout file is genuinely growing while the sentinel runs.

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Owns CheckpointAnalysis, interval/repetition slices, and should become the single owner of
  per-class checkpoint state construction.

crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Owns exported Checkpoint, DriftScore, and the new exported drift-state enum.

crates/agent-drift-analyzer/src/scoring/
  Produces drift scores that should now carry analyzer-owned state instead of leaving state
  reconstruction to sentinel.

crates/agent-drift-analyzer/tests/checkpoints.rs
crates/agent-drift-analyzer/tests/export_bundle.rs
crates/agent-drift-analyzer/tests/truth_grounding_gap.rs
crates/agent-drift-analyzer/tests/dead_end_thrash.rs
  Regression coverage for state semantics and checkpoint export.

crates/agent-drift-sentinel/src/operator_surface.rs
  Owns checkpoint presentation and should consume analyzer-exported state instead of historical
  reason prefixes plus previous-checkpoint heuristics.

crates/agent-drift-sentinel/src/input.rs
crates/agent-drift-sentinel/src/live_input.rs
  Own replay/live checkpoint loading and compatibility handling for `v0.2` and `v0.3`.

crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
  Regression coverage for posture cutover and replay/live parity.

docs/specs/hybrid-drift-sentinel-implementation-order.md
  Packet authority and continuity notes for where `v0.6` sits relative to `v0.5A` and `v0.5B`.
```

## Code Style

Make checkpoint state explicit in analyzer types instead of encoding it through a mix of
`flagged`, historical reason prefixes, and previous-checkpoint heuristics.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftState {
    Active,
    Recovered,
    HistoricalOnly,
    Cleared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftScore {
    pub class: DriftClass,
    pub state: DriftState,
    pub raw_score: u8,
    pub confidence: Confidence,
    pub flagged: bool,
    pub evidence: Vec<EvidenceRef>,
}
```

Conventions:

- keep checkpoint-state ownership in analyzer, not in sentinel presentation helpers
- use additive schema changes with explicit versioning; do not smuggle state through reason strings
- preserve sentinel fallback support for `v0.2`, but route new behavior through `v0.3` state
- keep checkpoint-level posture a downstream projection of per-class analyzer state
- keep typed outcome evidence as a later seam; do not mix state ownership and evidence-typing in
  one implementation packet

## Testing Strategy

Required test layers:

1. Analyzer checkpoint-state unit coverage
   - verify each drift class can emit `active`, `recovered`, `historical_only`, and `cleared`
     when the underlying semantics support those states
2. Analyzer export coverage
   - verify `checkpoints.jsonl` serializes `DriftScore.state`
   - verify checkpoint schema version moves to `v0.3`
3. Sentinel compatibility coverage
   - verify sentinel still accepts `v0.2` checkpoints
   - verify sentinel prefers explicit analyzer state when `v0.3` checkpoints are present
4. Replay/live parity coverage
   - verify matching checkpoints produce the same posture in replay and live flows after cutover
5. Bounded live proof
   - verify an actually active session still renders honest posture transitions after the cutover

## Boundaries

- Always:
  - treat `docs/specs/hybrid-drift-sentinel-implementation-order.md` as the packet authority
  - keep analyzer state construction and sentinel presentation responsibilities separate
  - update tests for both analyzer export and sentinel compatibility in the same packet
- Ask first:
  - widening the compactor artifact contract
  - adding new drift classes or changing scheduler policy
  - removing `v0.2` sentinel fallback instead of keeping compatibility during cutover
- Never:
  - mix the `ToolOutput` evidence redesign into this packet
  - reopen `v0.5B` coordinator persistence work from the checkpoint-state slice
  - make reason prefixes the control plane once explicit state exists

## Success Criteria

1. Analyzer exports explicit `DriftState` on every `DriftScore` and bumps checkpoint
   `schema_version` to `v0.3`.
2. Sentinel replay/live posture derivation becomes current-checkpoint-state-first and no longer
   depends on historical reason-prefix parsing when `v0.3` checkpoints are present.
3. Sentinel remains compatible with `v0.2` checkpoints during the transition.
4. Replay/live parity and the bounded live proof stay green after the cutover.
5. The follow-on typed-outcome evidence seam remains documented as separate work, not silently
   absorbed into this packet.

## Open Questions

1. `v0.2` fallback lifetime: should sentinel keep previous-checkpoint / reason-prefix fallback
   indefinitely for old bundles, or should a later cleanup packet remove it after all fixtures and
   proofs are refreshed?
   - Assumed default for this spec: keep compatibility in this packet and defer removal.
2. `WrongPlanBranch` recovery labeling: should it ever emit `historical_only`, or should it remain
   `active` / `cleared` only because it is meant to be a current-scope claim?
   - Assumed default for this spec: keep `WrongPlanBranch` as `active` / `cleared` only unless
     live code review proves a third state is semantically honest.
