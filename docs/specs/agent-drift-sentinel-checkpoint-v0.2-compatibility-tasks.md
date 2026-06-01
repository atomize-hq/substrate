# Tasks: Agent Drift Sentinel Checkpoint v0.2 Compatibility

This task list implements:

- [agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md:1)
- [agent-drift-sentinel-checkpoint-v0.2-compatibility-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-plan.md:1)

## Task List

## Packet SC1: Boundary And Current-Schema Compatibility

- [ ] Task: Lock the packet as compatibility-only in repo docs
  - Acceptance: the spec/plan/tasks chain explicitly says this packet restores sentinel
    compatibility with analyzer checkpoint `v0.2`, requires `diagnostics` in the checkpoint shape,
    and defers any new scheduler/operator behavior that would consume `diagnostics`.
  - Verify: doc review against
    `crates/agent-drift-sentinel/src/input.rs`,
    `crates/agent-drift-sentinel/src/live_input.rs`,
    `crates/agent-drift-sentinel/src/operator_surface.rs`,
    `crates/agent-drift-sentinel/src/scheduler.rs`, and
    `crates/agent-drift-sentinel/src/live_runtime.rs`
  - Files:
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md`
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-plan.md`
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-tasks.md`

- [ ] Task: Cut replay bundle loading to analyzer checkpoint schema `v0.2`
  - Acceptance: `load_replay_bundle` accepts non-empty replay bundles whose checkpoint rows all use
    `schema_version = "v0.2"`, reports `bundle.schema_version = "v0.2"`, preserves stable sorting,
    and still rejects mixed or unsupported schema versions explicitly.
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`

- [ ] Task: Cut live checkpoint compatibility to analyzer checkpoint schema `v0.2`
  - Acceptance: `verify_live_checkpoint_compatibility` accepts checkpoints with
    `schema_version = "v0.2"` and `diagnostics`, preserves the existing validation for cursor
    identity, non-empty `session_id`, non-empty `checkpoint_id`, non-empty
    `task_frame.objective`, and non-empty `expected_next_step`, and does not add new behavior that
    consumes `diagnostics`.
  - Verify:
    - `cargo test -p agent-drift-sentinel live_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_runtime -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/live_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
    - `crates/agent-drift-sentinel/tests/live_runtime.rs`

Packet `SC1` exit condition:

- replay and live current-schema checks both accept analyzer checkpoint `v0.2`
- this packet still only restores compatibility; it does not change sentinel behavior from
  `diagnostics`

## Packet SC2: Test Support, Fixtures, And Downstream Proof

- [ ] Task: Update sentinel test support to build current-schema checkpoints with
  `CheckpointDiagnostics`
  - Acceptance: `crates/agent-drift-sentinel/tests/support/mod.rs` becomes the canonical minimal
    sentinel checkpoint builder for the current analyzer contract by emitting:
    - `schema_version = "v0.2"`
    - explicit `CheckpointDiagnostics`
    - unchanged replay/live test intent for checkpoint ids, evidence, flagged scores, and expected
      next steps
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/support/mod.rs`

- [ ] Task: Rewrite or regenerate the checked-in live fixtures to include `diagnostics`
  - Acceptance: both checked-in live fixture files encode `schema_version = "v0.2"` and a
    `diagnostics` object on every `checkpoint_ready` payload, while preserving:
    - append-only emission ordinals in `append_only_stream.jsonl`
    - the cursor-regression failure shape in `cursor_regression_stream.jsonl`
  - Verify:
    - `cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
    - `sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl`
    - `sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/cursor_regression_stream.jsonl`
  - Files:
    - `crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl`
    - `crates/agent-drift-sentinel/tests/fixtures/live/cursor_regression_stream.jsonl`
    - `crates/agent-drift-sentinel/tests/live_input_adapter.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

- [ ] Task: Run the full sentinel regression and prove the downstream seam is restored
  - Acceptance: the sentinel crate passes against the current analyzer checkpoint contract, no test
    or runtime path still hardcodes checkpoint schema `v0.1`, and `diagnostics` is successfully
    deserialized and carried through replay and bounded live paths without new behavior changes.
  - Verify:
    - `cargo test -p agent-drift-sentinel -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/support/mod.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`
    - `crates/agent-drift-sentinel/tests/live_input.rs`
    - `crates/agent-drift-sentinel/tests/live_input_adapter.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
    - `crates/agent-drift-sentinel/tests/live_runtime.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

- [ ] Task: Refresh stale continuity docs and task checklists after the green proof
  - Acceptance: repo docs that currently imply sentinel is still behind analyzer checkpoint `v0.2`
    are updated to say the downstream seam is restored and that first sentinel behavior using
    `diagnostics` remains deferred to a later packet.
  - Verify: doc review against the final green test state and the packet boundary
  - Files:
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `docs/specs/agent-drift-sentinel-live-integration-v0.3-spec.md`
    - `docs/specs/agent-drift-sentinel-live-integration-v0.3-plan.md`
    - `docs/specs/agent-drift-sentinel-live-integration-v0.3-tasks.md`
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-spec.md`
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-plan.md`
    - `docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-tasks.md`

Packet `SC2` exit condition:

- sentinel test support and checked-in fixtures match analyzer checkpoint `v0.2`
- the full sentinel crate is green
- continuity docs no longer describe the downstream seam as broken

## Final Packet Exit Condition

- `agent-drift-sentinel` accepts analyzer checkpoint `schema_version = "v0.2"` in replay and live
  compatibility paths
- sentinel fixtures and test support include `CheckpointDiagnostics`
- `cargo test -p agent-drift-sentinel -- --nocapture` passes
- this packet stops at compatibility restoration; first user-visible sentinel behavior that uses
  `diagnostics` is intentionally deferred
