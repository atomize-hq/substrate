# Plan: Agent Drift Analyzer Turn Context R3

## Scope

This plan implements:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`

`R3` is the first context-deepening packet after the landed `R1` scorer-honesty family and the
landed `R2` analyzer acceptance wall.

Its job is to promote turn shape from summary-only reporting into analyzer-owned checkpoint state
that replay/live consumers can inspect directly.

This packet should:

- define one structured `TurnContext` contract on analyzer checkpoints
- compute deterministic current-turn slices from existing bundle rows
- export current-turn counts, timing, checkpoint density, activity mix, and coarse execution mode
- widen analyzer checkpoints from `v0.3` to `v0.4`
- preserve replay/live compatibility by extending sentinel support to `v0.4` while keeping `v0.3`
- render compact turn-context inspection in summary and replay/live operator surfaces

This packet should not:

- add archetype classification
- add progress semantics
- retune `dead_end_thrash` or any other drift scorer
- adopt task-level multi-trajectory references or per-action intent labels
- change scheduler triggers, warning thresholds, or live-runtime coordinator behavior

## Why This Packet Comes Next

The current analyzer already exports useful session-shape metrics:

- `turns observed`
- `user prompts observed`
- `checkpoints per turn`
- `avg rows between checkpoints`
- `avg seconds between checkpoints`
- working-set churn and verification density

But those metrics currently live only in summary-level reporting. The checkpoint contract still
lacks any turn-local state, so downstream consumers cannot directly tell whether a suspicious or
busy-looking checkpoint belongs to:

- one long autonomous implementation turn
- a rapid conversational back-and-forth
- a verification-heavy suffix inside one stable turn

`R3` is therefore the smallest honest next step:

- richer structure than `R2`
- still narrower than `R4` archetype or `R5` progress
- no scorer changes yet

## Implementation Strategy

### Workstream 1: Lock The Turn-Context Contract

Define the `R3` contract in code and docs first:

- `TurnContext`
- `TurnActivityMix`
- `TurnExecutionMode`
- schema `v0.4`

Why first:

- later analyzer, export, and sentinel work all depend on a stable contract
- this is the place to encode the narrow interpretation of AgentLens/DST inspiration before code
  drifts into `R4+`

### Workstream 2: Add Analyzer Turn-Slice Derivation

Implement one analyzer-local helper that derives the current turn slice for each checkpoint using:

- checkpoint boundary rows
- latest observed `turn_id`
- archival ordering
- timestamps
- command observations

The helper should compute:

- `turn_id`
- `turn_ordinal`
- `rows_since_turn_start`
- `seconds_since_turn_start`
- `checkpoints_in_turn`
- `prompts_observed_in_session`
- `activity_mix`
- `execution_mode`

Why second:

- the analyzer must own the raw turn-context truth before export or sentinel compatibility changes

### Workstream 3: Widen Analyzer Export To `v0.4`

Once analyzer computation is stable:

- attach `turn_context` to every checkpoint
- switch new exports to `schema_version = "v0.4"`
- update summary rendering with compact turn-context inspection

Why third:

- widening the public contract before deterministic analyzer derivation is stable would create a
  moving downstream target

### Workstream 4: Extend Sentinel Compatibility

Update replay/live checkpoint loaders to:

- accept `v0.4`
- preserve `v0.2`/`v0.3` fallback behavior
- treat missing `turn_context` as acceptable for legacy schemas only

Why fourth:

- once analyzer emits `v0.4`, replay/live compatibility becomes the primary downstream risk

### Workstream 5: Surface Turn Context In Replay/Live Presentation

Add a compact presentation seam so replay/live consumers can inspect turn context directly without
opening raw JSON artifacts.

This should:

- render a compact summary of current-turn shape
- stay presentation-only
- preserve current posture/severity behavior
- keep replay/live parity tests tight

Why fifth:

- consumer visibility is part of the packet objective, but it should ride on top of the stabilized
  analyzer contract rather than shape it

## Sequencing

Sequential work:

1. lock the `R3` contract in docs
2. add analyzer-local turn-context types and helper seams
3. populate `turn_context` during checkpoint construction
4. widen analyzer checkpoint export to `v0.4`
5. update analyzer summary rendering and analyzer tests
6. extend sentinel replay/live compatibility to `v0.4`
7. expose compact turn-context output in replay/live presentation
8. run focused analyzer and sentinel walls

Parallel-safe work after the contract is locked:

- analyzer fixture/test drafting for long-turn vs many-short-turn cases
- summary rendering test drafting while turn derivation is being coded
- sentinel presentation wording once the structured fields are stable

Not parallel-safe:

- finalizing sentinel compatibility before the `v0.4` analyzer contract is stable
- adding archetype/progress-like fields while turn-context definitions are still moving
- folding turn context into scorer logic during the same packet

## Major Risks And Mitigations

### Risk 1: `R3` Quietly Becomes `R4`

Mitigation:

- keep `TurnExecutionMode` coarse and observational
- reject any field that classifies troubleshooting/planning/review semantics in this packet
- keep success criteria focused on structural turn-state exposure

### Risk 2: Additive `v0.3` Growth Creates Contract Ambiguity

Mitigation:

- widen to explicit schema `v0.4`
- keep `v0.3` compatibility isolated to sentinel loaders
- test legacy `v0.3` fallback and `v0.4` preferred behavior side by side

### Risk 3: Execution-Mode Rules Become Hidden Quality Judgments

Mitigation:

- derive execution mode from raw current-turn counts only
- bias ambiguous cases to `mixed`
- defer value-laden interpretation to `R4` and `R5`

### Risk 4: Replay/Live Surfaces Drift Apart

Mitigation:

- keep one compact presentation shape for turn context
- prove replay/live parity with existing `live_end_to_end` style tests
- avoid custom formatting branches that exist only in one path

### Risk 5: Turn Metrics Re-define Landed Summary Semantics

Mitigation:

- explicitly preserve existing definitions for `turns observed` and `user prompts observed`
- render turn context as an addition, not a replacement, in `summary.md`
- reuse the landed calibration semantics by reference instead of inventing new ratios

## Verification Checkpoints

### Checkpoint 1: Contract Boundary Locked

Confirm the spec, plan, and tasks all agree that:

- `R3` is checkpoint-local turn context only
- schema widening is explicit and versioned
- archetype, progress, and scorer changes stay out of scope

### Checkpoint 2: Analyzer `v0.4` Contract Passes

Run:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

This should prove deterministic `turn_context` derivation and stable `v0.4` export.

### Checkpoint 3: Sentinel Compatibility Holds

Run:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
```

This should prove `v0.4` acceptance and `v0.3` fallback.

### Checkpoint 4: Replay/Live Presentation Parity Holds

Run:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

This should prove replay/live consumers inspect the same compact turn-context view without posture
drift.

## Exit Condition

`R3` is complete when:

- analyzer emits deterministic `v0.4` checkpoints with structured `turn_context`
- analyzer summary exposes compact turn-context inspection
- sentinel replay/live paths accept `v0.4` while keeping `v0.3` fallback
- replay/live operator surfaces expose turn-local context directly
- no archetype, progress, scorer, scheduler, or warning-policy semantics moved during the packet
