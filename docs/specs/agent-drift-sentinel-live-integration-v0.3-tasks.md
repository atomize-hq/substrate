# Tasks: Agent Drift Sentinel Live Integration v0.3

This task list implements:

- [agent-drift-sentinel-live-integration-v0.3-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-live-integration-v0.3-spec.md:1)
- [agent-drift-sentinel-live-integration-v0.3-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-live-integration-v0.3-plan.md:1)

Continuity note:

- `2026-06-01`: the underlying live fixtures now carry analyzer checkpoint `schema_version =
  "v0.2"` with `diagnostics` as part of `SC2`. The live-integration tasks below remain bounded to
  runtime/sink behavior and do not authorize diagnostics-driven policy changes.

## Task List

- [x] Task: Define the incremental live checkpoint event contract
  - Acceptance: `agent-drift-sentinel` has a library-owned event contract for `CheckpointReady`, `Heartbeat`, `RepeatedFailure`, and `ManualReview`, with stable cursor and ordering rules documented in code and tests.
  - Verify: `cargo test -p agent-drift-sentinel live_input -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/live_input.rs`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/tests/live_input.rs`

- [x] Task: Implement a bounded live input adapter or fixture loader
  - Acceptance: the sentinel can consume append-only live checkpoint events from a fixture-backed source or equivalent adapter seam without depending on shell/world runtime code.
  - Verify: `cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/live_input.rs`, `crates/agent-drift-sentinel/tests/live_input_adapter.rs`, `crates/agent-drift-sentinel/tests/fixtures/live/`

- [x] Task: Gate analyzer compatibility for incremental live consumption
  - Acceptance: the implementation proves that existing analyzer checkpoints are sufficient for live warning evaluation, or stops and documents the exact upstream analyzer contract gap without papering over it inside the sentinel.
  - Verify: `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/live_input.rs`, `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`, `docs/specs/agent-drift-sentinel-live-integration-v0.3-plan.md`
  - Gate: `raise-to-user-if-failed`

- [x] Task: Implement the library-owned live runtime on top of shared scheduler state
  - Acceptance: the sentinel can accept incremental live events, update runtime state, and decide whether to evaluate without forking replay scheduler semantics.
  - Verify: `cargo test -p agent-drift-sentinel live_runtime -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/live_runtime.rs`, `crates/agent-drift-sentinel/src/scheduler.rs`, `crates/agent-drift-sentinel/src/lib.rs`

- [x] Task: Implement the live operator sink surface
  - Acceptance: visible warnings, silent checkpoints, and heartbeat/status events can be emitted through a structured sink surface instead of being hard-coded to direct console output.
  - Verify: `cargo test -p agent-drift-sentinel operator_sink -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/operator_sink.rs`, `crates/agent-drift-sentinel/src/operator_surface.rs`, `crates/agent-drift-sentinel/tests/operator_sink.rs`

- [x] Task: Preserve replay behavior while exposing the bounded live seam
  - Acceptance: replay mode still passes unchanged while the library exports the new live runtime seam; any binary changes remain thin and fixture-oriented only.
  - Verify: `cargo test -p agent-drift-sentinel replay_input -- --nocapture` and `cargo test -p agent-drift-sentinel warning_policy -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/src/cli.rs`, `crates/agent-drift-sentinel/tests/replay_input.rs`, `crates/agent-drift-sentinel/tests/warning_policy.rs`

- [x] Task: Run a bounded live end-to-end proof over fixture streams
  - Acceptance: a synthetic or file-backed incremental stream produces stable live warning output without touching shell/world integration, and the proof is documented as sentinel-local only.
  - Verify: `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files: `crates/agent-drift-sentinel/tests/live_end_to_end.rs`, `crates/agent-drift-sentinel/tests/fixtures/live/`, `docs/specs/agent-drift-sentinel-live-integration-v0.3-plan.md`
  - Gate: `always-check-with-user`

- [x] Task: Hold the post-slice runtime gate after the bounded live proof
  - Acceptance: the docs and implementation stop short of shell/world or host runtime wiring, and the next broader integration step is explicitly framed as a separate approval gate.
  - Verify: human review gate recorded in the plan/task docs before any runtime-wiring task starts
  - Files: `docs/specs/agent-drift-sentinel-live-integration-v0.3-spec.md`, `docs/specs/agent-drift-sentinel-live-integration-v0.3-plan.md`, `docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Gate: `always-check-with-user`
