# Tasks: Agent Drift Analyzer Checkpoint State v0.6

This task list implements:

- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`

## Task List

## Packet v0.6A: Analyzer-Owned State Contract And Export

- [x] Task: Lock the checkpoint-state contract and packet boundary in repo docs
  - Acceptance: the docs explicitly define:
    - analyzer-owned `active`, `recovered`, `historical_only`, and `cleared` per-class state
    - additive checkpoint export on `DriftScore`
    - checkpoint schema bump from `v0.2` to `v0.3`
    - `v0.6A` as analyzer contract/export work
    - `v0.6B` as sentinel cutover/proof work
    - typed outcome evidence as a separate follow-on
  - Verify: doc review against the live analyzer checkpoint seam and sentinel posture seam
  - Files:
    - `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`

- [x] Task: Add one analyzer-owned `DriftState` contract and state builder
  - Acceptance: analyzer code has one centralized state seam that:
    - defines exported per-class `DriftState`
    - computes state from `CheckpointAnalysis` plus scorer semantics
    - keeps `WrongPlanBranch` narrowly current-state unless live code review justifies more
    - does not require sentinel-local previous-checkpoint logic for `v0.3`
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`

- [x] Task: Widen checkpoint export to `v0.3` with per-class state
  - Acceptance: exported checkpoints serialize `DriftScore.state`, checkpoint `schema_version`
    becomes `v0.3`, and analyzer export tests prove the new shape without changing compactor input.
  - Verify:
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

Packet `v0.6A` exit condition:

- analyzer owns one explicit per-class state seam
- checkpoint export is additively widened to `v0.3`
- analyzer tests prove state semantics and export shape before sentinel cutover begins
- `v0.6B` remains unstarted sentinel work; this task file intentionally leaves that packet pending

## Packet v0.6B: Sentinel Cutover, Compatibility, And Proof

- [ ] Task: Cut sentinel posture over to analyzer-exported state with `v0.2` fallback
  - Acceptance: sentinel derives checkpoint posture from current-checkpoint `DriftState` values
    when `v0.3` checkpoints are present, while legacy `v0.2` bundles still work through an
    isolated compatibility fallback.
  - Verify:
    - `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

- [ ] Task: Prove replay/live parity survives the checkpoint-state cutover
  - Acceptance: replay and live paths assign the same checkpoint posture after the analyzer-state
    cutover, and compatibility tests cover both `v0.2` fallback and `v0.3` preferred behavior.
  - Verify:
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
    - `cargo test -p agent-drift-sentinel operator_sink -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
    - `crates/agent-drift-sentinel/tests/operator_sink.rs`
    - `crates/agent-drift-sentinel/tests/fixtures/live/`

- [ ] Task: Refresh packet authority notes and bounded live proof guidance
  - Acceptance: implementation-order notes record `v0.6` as the analyzer follow-up after landed
    `v0.5A` / `v0.5B`, and the smoke/proof guidance reflects analyzer-owned state rather than
    sentinel-owned reason-prefix posture reconstruction.
  - Verify:
    - doc review against final analyzer/sentinel behavior
    - bounded live proof using the agreed command shape
  - Files:
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

Packet `v0.6B` exit condition:

- sentinel becomes presentation-only for `v0.3` checkpoint posture
- `v0.2` fallback remains green
- replay/live parity and bounded live proof stay green
- typed outcome evidence remains the next follow-on, not scope creep inside `v0.6`
