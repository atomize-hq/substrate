# Tasks: Agent Drift Analyzer Session Archetype R4

This task list implements:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`

## Task List

Completion status on `2026-06-08`:

- The analyzer outcome-evidence `R1` family is landed on this worktree.
- `R2`, `R3`, and `R3.5` are landed on this worktree.
- `R3.75-1` through `R3.75-3` are landed on this worktree.
- `R3.75-4` remains the immediate next packet before `R4`.
- `R4` remains the next session-archetype packet family after the remaining `R3.75` packet lands.
- `R5`, `R6`, `R7`, and `R8` remain outside the `R4` boundary.

Validation gate for this phase:

- this task list is the `TASKS` phase artifact for review
- do not begin `R4-2+` implementation until the spec and plan are reviewed and accepted
- if scope or packet boundaries change, update the spec first, then the plan, then this task list

## Packet R4-1: Repo Doc Contract Lock

- [x] Task: Lock the `R4` packet boundary and `v0.5` session-archetype contract in repo docs
  - Acceptance:
    - docs define `R4` as checkpoint-local session-archetype classification only
    - docs explicitly keep `R5` progress, `R6` scorer cutover, `R7` full delegated-session
      support, and `R8` sentinel consolidation out of scope
    - docs define `SessionArchetype` and `SessionArchetypeLabel`
    - docs state that analyzer checkpoints widen explicitly from `v0.4` to `v0.5`
    - docs define the initial archetypes:
      - `troubleshooting`
      - `planning`
      - `autonomous_implementation`
      - `verification_closeout`
    - docs state that classification is deterministic, confidence-bearing, and derived from
      observable evidence rather than commentary alone
    - docs state that the exported label is session type at checkpoint scope, not only turn
      archetype
  - Verify:
    - doc review against `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - doc review against `docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Packet `R4-1` historical exit condition before `R4-2+` lands:

- repo docs lock `R4` as checkpoint-local session-archetype classification only
- repo docs lock the explicit checkpoint-contract decision `v0.4 -> v0.5`
- repo docs lock the four initial archetype labels and the evidence-backed classification rule
- repo docs lock the AgentLens-style lower intent-evidence layer as the preferred implementation
  direction
- `R4-2`, `R4-3`, `R4-4`, `R5`, `R6`, `R7`, and `R8` remain unstarted here

## Packet R4-2: Analyzer Session-Archetype Contract And Export

- [ ] Task: Add analyzer-owned session-archetype schema types and exports
  - Acceptance:
    - analyzer checkpoint schema gains `SessionArchetype` and `SessionArchetypeLabel`
    - `agent_drift_analyzer::Checkpoint` exports `session_archetype`
    - new checkpoints emit `schema_version = "v0.5"`
    - legacy `v0.4` fixtures remain loadable where required by downstream tests
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/lib.rs`

## Packet R4-3: Analyzer Derivation, Summary, And Regression Walls

- [ ] Task: Derive deterministic checkpoint archetype during checkpoint analysis
  - Acceptance:
    - analyzer computes session type at checkpoint scope from existing task-frame, turn-context,
      diagnostics, and command-observation evidence
    - a lower deterministic intent-evidence layer exists or is explicitly introduced as the input
      seam for archetype classification
    - `supporting_evidence` and `counter_evidence` are populated deterministically
    - ambiguous cases degrade confidence instead of inventing a fifth archetype
    - no `R5` progress semantics are introduced
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task: Extend analyzer summary output with compact archetype inspection
  - Acceptance:
    - `summary.md` renders compact archetype and confidence information for each checkpoint or
      session summary block
    - existing turn-context and calibration metrics remain present and unchanged in meaning
    - rendering remains concise and operator-readable
  - Verify:
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`

- [ ] Task: Add analyzer regression coverage for the four initial archetype shapes
  - Acceptance:
    - one deterministic case proves troubleshooting classification with expected verification churn
    - one deterministic case proves planning classification with directive-heavy synthesis
    - one deterministic case proves autonomous implementation classification with stable
      write/test cadence
    - one deterministic case proves verification-closeout classification with proof-oriented
      narrowing
    - reruns on the same bundle preserve identical `session_archetype`
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`

## Packet R4-4: Sentinel Compatibility And Presentation

- [ ] Task: Extend sentinel replay input and live compatibility to `v0.5`
  - Acceptance:
    - replay input accepts and sorts `v0.5` checkpoints
    - live compatibility accepts `v0.5` checkpoints
    - `v0.4` fallback remains intact
    - contract validation remains explicit about which fields are required for each schema version
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

- [ ] Task: Surface compact archetype inspection in replay/live operator presentation
  - Acceptance:
    - replay and live surfaces expose the same compact archetype inspection view for matching
      checkpoints
    - posture, trigger labeling, and diagnostics rendering remain unchanged
    - turn-context output stays available alongside the new archetype output
  - Verify:
    - `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

Packet `R4` exit condition:

- analyzer emits `v0.5` checkpoints with structured `session_archetype`
- analyzer summary exposes compact archetype inspection
- sentinel replay/live consumers accept `v0.5` and preserve earlier schemas
- replay/live surfaces expose the same compact archetype view
- `R4` lands without progress, scorer, scheduler, or warning-policy drift
