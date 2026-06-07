# Tasks: Agent Drift Analyzer Turn Context R3

This task list implements:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`

## Task List

Completion status on `2026-06-07`:

- Packets `R3-1` through `R3-4` are landed on this worktree.
- The focused analyzer and sentinel verification wall for `R3` is green.
- `R4`, `R5`, and `R6` remain outside the landed `R3` boundary.

## Packet R3-1: Repo Doc Contract Lock

- [x] Task: Lock the `R3` packet boundary and `v0.4` turn-context contract in repo docs
  - Acceptance:
    - docs define `R3` as checkpoint-local turn-context promotion only
    - docs explicitly keep `R4` archetype, `R5` progress, and `R6` scorer cutover out of scope
    - docs define `TurnContext`, `TurnActivityMix`, and `TurnExecutionMode`
    - docs state that analyzer checkpoints widen explicitly from `v0.3` to `v0.4`
    - docs encode the deliberate differences from AgentLens and DST-inspired work
  - Verify:
    - doc review against `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - doc review against `docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
    - `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
    - `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Packet `R3-1` historical exit condition before `R3-2+` landed:

- repo docs lock `R3` as checkpoint-local turn context only
- repo docs lock the explicit checkpoint-contract decision `v0.3 -> v0.4`
- repo docs lock the deliberate differences from AgentLens and DST-inspired patterns
- `R3-2`, `R3-3`, `R3-4`, `R4`, `R5`, and `R6` remain unstarted here

## Packet R3-2: Analyzer Turn-Context Contract And Export

- [x] Task: Add analyzer-owned turn-context schema types and exports
  - Acceptance:
    - analyzer checkpoint schema gains `TurnContext`, `TurnActivityMix`, and `TurnExecutionMode`
    - `agent_drift_analyzer::Checkpoint` exports `turn_context`
    - new checkpoints emit `schema_version = "v0.4"`
    - legacy `v0.3` fixtures remain loadable where required by downstream tests
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/lib.rs`

## Packet R3-3: Analyzer Derivation, Summary, And Regression Walls

- [x] Task: Derive deterministic current-turn slices during checkpoint analysis
  - Acceptance:
    - analyzer computes current-turn boundaries from existing row order and `turn_id`
    - `turn_ordinal`, `rows_since_turn_start`, `seconds_since_turn_start`, and
      `checkpoints_in_turn` are populated deterministically
    - `prompts_observed_in_session` reuses the landed role-aware semantics instead of redefining
      prompt counting
    - ambiguous/no-`turn_id` cases degrade conservatively rather than inventing structure
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task: Add deterministic turn activity mix and coarse execution-mode derivation
  - Acceptance:
    - analyzer computes `directive_row_count`, `assistant_message_count`, `tool_call_count`,
      `read_like_command_count`, `write_like_command_count`,
      `verification_like_command_count`, and `tool_output_count` for the current turn
    - analyzer derives only the coarse execution modes:
      - `conversational`
      - `autonomous`
      - `verification_heavy`
      - `mixed`
    - ambiguous cases bias to `mixed`
    - no archetype/progress semantics are introduced
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/context/mod.rs`
    - `crates/agent-drift-analyzer/src/context/working_set.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task: Extend analyzer summary output with compact turn-context inspection
  - Acceptance:
    - `summary.md` renders compact turn-context information for each session or checkpoint summary
      block
    - existing calibration metrics remain present and unchanged in meaning
    - summary output helps distinguish:
      - one long autonomous turn
      - many short conversational turns
    - rendering remains concise and operator-readable
  - Verify:
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`

- [x] Task: Add analyzer regression coverage for long-turn and many-short-turn shapes
  - Acceptance:
    - one deterministic test case proves multiple checkpoints inside one turn are recognized as a
      long agent-side/autonomous shape
    - one deterministic test case proves checkpoints across multiple turn ids are recognized as a
      many-short-turn/conversational shape
    - reruns on the same bundle preserve identical `turn_context`
    - checkpoint artifacts remain session-scoped and ordinal-stable
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`

## Packet R3-4: Sentinel Compatibility And Presentation

- [x] Task: Extend sentinel replay input and live compatibility to `v0.4`
  - Acceptance:
    - replay input accepts and sorts `v0.4` checkpoints
    - live compatibility accepts `v0.4` checkpoints
    - `v0.3` fallback remains intact
    - contract validation remains explicit about which fields are required for each schema version
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

- [x] Task: Surface compact turn-context inspection in replay/live operator presentation
  - Acceptance:
    - replay and live surfaces expose the same compact turn-context inspection view for matching
      checkpoints
    - posture classification remains unchanged
    - diagnostics and turn-context presentation stay readable without becoming a verbose dump
    - no scheduler, warning-policy, or trigger semantics change
  - Verify:
    - `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

Packet `R3` exit condition:

- analyzer emits `v0.4` checkpoints with structured `turn_context`
- analyzer summary exposes compact turn-context inspection
- sentinel replay/live consumers accept `v0.4` and preserve `v0.3`
- replay/live surfaces expose the same compact turn-context view
- `R3` lands without archetype, progress, scorer, scheduler, or warning-policy drift

Packet `R3` completion note on `2026-06-07`:

- all `R3` packet tasks above are landed
- the focused verification wall passed:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
