# Plan: Agent Drift Sentinel Checkpoint v0.2 Compatibility

## Scope

This plan implements
[agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md:1).

The goal is to restore the downstream sentinel seam after analyzer checkpoint `v0.2` landed by:

- cutting replay input over to the current analyzer checkpoint schema
- cutting live compatibility checks and live fixtures over to the current analyzer checkpoint schema
- updating sentinel test support so it constructs `CheckpointDiagnostics`
- proving the full sentinel crate is green again
- refreshing stale continuity docs only after the seam is restored

Status note:

- `2026-06-01`: `SC1` and `SC2` are green. Sentinel test support and the checked-in live fixtures
  now emit current-schema `v0.2` checkpoints with `diagnostics`, while scheduler and operator
  behavior remain compatibility-only and do not consume `diagnostics`.

This packet is intentionally compatibility-only.

It should:

- deserialize and carry `diagnostics`
- accept analyzer checkpoint `schema_version = "v0.2"`
- refresh fixtures and test support so they match the shared checkpoint type

It should not:

- change scheduler policy
- change warning fingerprinting
- change operator output semantics
- start using `diagnostics` to influence sentinel behavior
- widen analyzer output or broaden live runtime integration

## Why This Boundary Is Correct

The repo state that motivated this packet was a narrow compatibility-first break:

- analyzer already exports checkpoint `schema_version = "v0.2"` and includes a compact
  `diagnostics` payload
- replay input in `crates/agent-drift-sentinel/src/input.rs` hard-rejected anything except
  `v0.1`
- live compatibility in `crates/agent-drift-sentinel/src/live_input.rs` hard-rejected
  anything except `v0.1`
- sentinel test support in `crates/agent-drift-sentinel/tests/support/mod.rs` initialized
  `Checkpoint` without `diagnostics`
- both checked-in live fixtures serialized `schema_version = "v0.1"` checkpoints without
  `diagnostics`
- `cargo test -p agent-drift-sentinel -- --nocapture` failed at
  `crates/agent-drift-sentinel/tests/support/mod.rs:79` with `missing field diagnostics in
  initializer of Checkpoint`

That break is now repaired:

- replay and live compatibility accept current-schema analyzer checkpoints
- `tests/support/mod.rs` is the canonical minimal `v0.2` checkpoint builder with explicit
  `CheckpointDiagnostics`
- both checked-in live fixtures encode `diagnostics` on every `checkpoint_ready` payload
- `cargo test -p agent-drift-sentinel -- --nocapture` and
  `cargo test -p agent-drift-analyzer -- --nocapture` pass without widening sentinel behavior

The same repo state argues against consuming `diagnostics` in this packet:

- `crates/agent-drift-sentinel/src/operator_surface.rs` derives presentation from
  `drift_scores`, `flagged`, `expected_next_step`, and evidence refs
- `warning_fingerprint(checkpoint)` depends only on `session_id`, flagged drift classes, and
  `expected_next_step`
- `crates/agent-drift-sentinel/src/scheduler.rs` makes decisions from cursor progression,
  checkpoint flagged state, and warning-fingerprint dedupe
- `crates/agent-drift-sentinel/src/live_runtime.rs` only needs compatibility-checked checkpoints to
  feed the existing scheduler and operator surfaces

Because the hard break is contract drift and not missing `diagnostics`-driven behavior, the right
next packet is:

- direct current-schema compatibility cutover now
- intentional `diagnostics` consumption later, as a separately justified packet

## Implementation Strategy

Restore the seam from the outside in:

1. lock the compatibility-only boundary in docs
2. cut replay loader version checks to the current analyzer checkpoint schema
3. cut live compatibility checks and fixture loading to the current analyzer checkpoint schema
4. update test support and checked-in fixtures so every sentinel test path uses the current
   checkpoint shape
5. run targeted and full sentinel verification
6. refresh continuity docs and packet checklists after green proof

This keeps the packet scoped to the true failure mode and avoids mixing behavior work with contract
repair.

## Major Components

### 1. Replay Input Compatibility

Deliver first:

- update replay bundle validation in `src/input.rs` from `v0.1` to `v0.2`
- ensure loaded bundle metadata reports the current schema version
- keep mixed-schema and unsupported-schema failures explicit

Why first:

- replay input is the primary closed-bundle sentinel seam
- it is the most direct downstream consumer of the analyzer checkpoint export

### 2. Live Input Compatibility

Deliver second:

- update `verify_live_checkpoint_compatibility` in `src/live_input.rs` from `v0.1` to `v0.2`
- preserve existing cursor/order/objective/expected-next-step checks
- keep compatibility focused on acceptance of the current analyzer checkpoint shape, not new
  behavior derived from `diagnostics`

Why second:

- live compatibility is a separate current-schema gate from replay loading
- live tests and runtime code should stay aligned with the same checkpoint version

### 3. Sentinel Test Support And Fixture Refresh

Deliver third:

- update `tests/support/mod.rs` so the canonical test builder emits:
  - `schema_version = "v0.2"`
  - explicit `CheckpointDiagnostics`
- rewrite or regenerate the two checked-in live fixtures so every `checkpoint_ready` payload
  includes `diagnostics`
- preserve their cursor and emission-ordinal intent

Why third:

- the initial red failure was caused by stale test-only construction
- test support should become the canonical local source for minimal valid checkpoints

### 4. Full Sentinel Regression

Deliver fourth:

- re-run the targeted replay, live-input, live-runtime, and fixture-backed suites
- re-run the full sentinel crate after targeted tests are green
- optionally re-run analyzer tests as upstream confirmation if needed during the packet

Why fourth:

- the seam is only restored when the crate-wide sentinel proof is green
- targeted tests alone are insufficient if some sentinel path still hardcodes `v0.1`

### 5. Continuity-Doc Refresh

Deliver last:

- update stale packet notes or checklists that still describe sentinel as lagging checkpoint
  `v0.2`
- record that this packet restored schema compatibility while deferring first
  `diagnostics`-consumption behavior

Why last:

- docs should describe the landed downstream state, not a planned repair
- the packet map should remain honest about what changed and what was deferred

## Sequencing

Sequential work:

1. compatibility boundary lock
2. replay input cutover
3. live input cutover
4. test-support checkpoint builder update
5. live fixture refresh
6. targeted sentinel verification
7. full sentinel verification
8. continuity-doc refresh

Parallel-safe work after the boundary is locked:

- replay and live test assertion updates can proceed in parallel once the target schema version is
  fixed
- fixture refresh can proceed in parallel with targeted test updates once the canonical test-only
  checkpoint shape is fixed
- continuity-doc refresh can be prepared in parallel but should not be finalized before green test
  proof

## Risks And Mitigations

### Risk 1: Accidental mixed-schema resting state

Risk:

- the implementation could keep partial `v0.1` fallback logic that becomes unowned long-term

Mitigation:

- cut replay and live compatibility directly to current schema `v0.2`
- keep unsupported or mixed schema failures explicit
- only preserve dual-schema support if a repo-local proof shows it is required

### Risk 2: Diagnostics consumption sneaks into the packet

Risk:

- once `diagnostics` is available, it becomes tempting to widen scheduler or operator behavior in
  the same packet

Mitigation:

- keep behavior boundaries explicit in the spec, plan, and tasks
- verify that operator and scheduler outputs remain driven by existing fields only

### Risk 3: Fixture drift hides the real contract

Risk:

- hand-edited fixture JSONL can drift away from the canonical checkpoint shape

Mitigation:

- make `tests/support/mod.rs` the canonical minimal checkpoint builder
- refresh checked-in fixtures only after the support builder shape is agreed
- keep fixture payloads minimal but complete, including `diagnostics`

### Risk 4: Green targeted tests but stale full-crate coverage

Risk:

- replay or live targeted suites may pass while another sentinel path still assumes `v0.1`

Mitigation:

- require `cargo test -p agent-drift-sentinel -- --nocapture` before calling the seam restored

### Risk 5: Docs remain stale after the repair

Risk:

- older continuity docs may keep telling future sessions that sentinel is behind analyzer `v0.2`

Mitigation:

- include a final continuity-doc refresh task once tests are green
- update packet notes to say the seam is restored and first `diagnostics` consumption remains
  deferred

## Verification Checkpoints

### Checkpoint VC-A: Boundary Locked

Verify:

- the spec/plan/tasks chain explicitly marks the packet compatibility-only
- the docs explicitly defer first sentinel behavior that consumes `diagnostics`

Evidence:

- the three new docs under `docs/specs/`

### Checkpoint VC-B: Replay And Live Inputs Accept Current Schema

Verify:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
```

Evidence:

- replay bundle tests report `schema_version = "v0.2"`
- live compatibility tests accept current-schema checkpoints with `diagnostics`

### Checkpoint VC-C: Fixtures And Support Match The Contract

Verify:

```bash
cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl
sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/cursor_regression_stream.jsonl
```

Evidence:

- checked-in live fixtures carry `diagnostics`
- ordering/cursor regression coverage still tests the intended behavior

### Checkpoint VC-D: Downstream Seam Restored

Verify:

```bash
cargo test -p agent-drift-sentinel -- --nocapture
```

Optional upstream confirmation:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Evidence:

- the full sentinel crate is green against the analyzer checkpoint contract
- no remaining sentinel path still hardcodes `v0.1`

### Checkpoint VC-E: Continuity Docs Updated

Verify:

- doc review against the final green state and the packet boundary

Evidence:

- stale packet notes or checklists no longer describe sentinel as lagging analyzer checkpoint
  `v0.2`
- docs explicitly record that `diagnostics` consumption remains a later packet

## Exit Criteria

This plan is complete when:

1. replay input, live input, test support, and live fixtures all use analyzer checkpoint
   `schema_version = "v0.2"`
2. every sentinel test checkpoint payload includes `CheckpointDiagnostics`
3. `cargo test -p agent-drift-sentinel -- --nocapture` passes
4. the packet ends without new behavior derived from `diagnostics`
5. continuity docs can state that the downstream sentinel seam is restored
