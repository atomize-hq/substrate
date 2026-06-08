# Tasks: Agent Drift Analyzer Session Archetype R4

This task list implements:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`

## Task List

Completion status on `2026-06-08`:

- The analyzer outcome-evidence `R1` family is landed on this worktree.
- `R2`, `R3`, and `R3.5` are landed on this worktree.
- `R3.75` delegation-aware analyzer boundary is landed on this worktree.
- `R4` is now the next session-archetype packet family and should consume the landed delegation
  boundary rather than blocking on it.
- the `SPECIFY`, `PLAN`, and `TASKS` artifacts now exist for `R4`, but implementation packets have
  not started yet
- `R5`, `R6`, `R7`, and `R8` remain outside the `R4` boundary.

Validation gate for this phase:

- this task list is the `TASKS` phase artifact for review
- do not begin `R4-1+` implementation until the spec and plan are reviewed and accepted
- `SPECIFY` / `PLAN` / `TASKS` are pre-implementation gates and are not counted as implementation
  packets
- if scope or packet boundaries change, update the spec first, then the plan, then this task list

## Pre-Implementation Gates

- [x] Task: Lock the `R4` packet boundary and `v0.5` session-archetype contract in repo docs
  - Acceptance:
    - docs state that `R3.75` is landed and that `R4` consumes the existing delegation boundary
      instead of treating it as an open blocker
    - docs define `R4` as checkpoint-local session-archetype classification only
    - docs explicitly keep `R5` progress, `R6` scorer cutover, `R7` full delegated-session
      support, and `R8` sentinel consolidation out of scope
    - docs define `SessionArchetype` and `SessionArchetypeLabel`
    - docs state that analyzer checkpoints widen explicitly from `v0.4` to `v0.5`
    - docs define the shared DTO as legacy-safe for `v0.2` through `v0.4` while making
      `session_archetype` required by contract for `v0.5`
    - docs define the initial archetypes:
      - `troubleshooting`
      - `planning`
      - `autonomous_implementation`
      - `verification_closeout`
    - docs state that classification is deterministic, confidence-bearing, and derived from
      observable evidence rather than commentary alone
    - docs state that the first code landing is behavior-first, with kickoff priors deferred,
      disabled, or weak-only behind an internal gate
    - docs state that the exported label is session type at checkpoint scope, not only turn
      archetype
  - Verify:
    - doc review against `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - doc review against `docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Pre-implementation gate status:

- the spec/plan/tasks stack exists and is reviewable
- packet numbering below starts at the first code-bearing implementation slice

## Packet R4-1: Analyzer Session-Archetype Contract And Export

- [ ] Task: Add analyzer-owned session-archetype schema types and exports
  - Acceptance:
    - analyzer checkpoint schema gains `SessionArchetype` and `SessionArchetypeLabel`
    - `agent_drift_analyzer::Checkpoint` exports `session_archetype` with legacy-safe serde
      behavior for `v0.2` through `v0.4`
    - new checkpoints emit `schema_version = "v0.5"`
    - `v0.5` validation requires `session_archetype`, while legacy fixtures remain loadable where
      required by downstream tests
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/lib.rs`

## Packet R4-2: Analyzer Derivation

- [ ] Task: Derive deterministic checkpoint archetype during checkpoint analysis
  - Acceptance:
    - analyzer computes session type at checkpoint scope from existing task-frame, turn-context,
      diagnostics, and command-observation evidence
    - a lower deterministic intent-evidence layer exists or is explicitly introduced as the input
      seam for archetype classification
    - landed delegation topology / child-visibility state is consumed to cap confidence or add
      counter-evidence for delegated-parent plus opaque-child cases
    - low-hanging command-role and file-role interpretation exists so broad command families do not
      become direct label proxies
    - `supporting_evidence` and `counter_evidence` are populated deterministically
    - ambiguous cases degrade confidence instead of inventing a fifth archetype
    - kickoff priors stay deferred, disabled, or weak-only behind an internal gate until
      behavior-first regressions are stable
    - no `R5` progress semantics are introduced
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## Packet R4-3: Analyzer Summary And Regression Walls

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
    - one ambiguous mixed case stays `low` or `medium` confidence
    - one legitimate transition case proves simple hysteresis / no flapping
    - one delegated-parent plus opaque-child case proves confidence capping
    - one PR-response loop defaults to `autonomous_implementation` while targeted edits plus local
      verification remain dominant
    - one proof-oriented closeout case proves verification-closeout can outweigh residual
      implementation
    - one failing verification loop proves troubleshooting beats verification-closeout
    - reruns on the same bundle preserve identical `session_archetype`
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`

## Packet R4-4: Sentinel Compatibility

- [ ] Task: Extend sentinel replay input and live compatibility to `v0.5`
  - Acceptance:
    - replay input accepts and sorts `v0.5` checkpoints
    - live compatibility accepts `v0.5` checkpoints
    - `v0.2` through `v0.4` fallback remains intact
    - contract validation remains explicit about which fields are required for each schema version
    - operator-surface explicit-state helpers recognize `v0.5` as analyzer-state-backed rather than
      hard-coding only `v0.3 | v0.4`
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

## Packet R4-5: Replay/Live Presentation

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
- delegated-parent plus opaque-child cases no longer overclaim high-confidence autonomous
  implementation from parent-visible behavior alone
- `R4` lands without progress, scorer, scheduler, or warning-policy drift
