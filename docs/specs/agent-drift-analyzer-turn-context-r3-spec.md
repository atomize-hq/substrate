# Spec: Agent Drift Analyzer Turn Context R3

## Assumptions I'm Making

1. Live repo truth on `2026-06-06` is the authority: the `R1` family is landed through bounded
   replay re-proof, `R2` acceptance-fixture hardening is landed, and the next open packet is `R3`
   turn-context promotion rather than more replay, fixture, or scorer-honesty work.
2. `R3` is intentionally narrower than `R4+`. Its job is to add explicit checkpoint-local turn
   state, not session archetype, progress semantics, or drift-scorer retuning.
3. The current analyzer surfaces already expose enough deterministic evidence to compute useful
   turn-local context:
   - row-level `turn_id`
   - boundary timestamps
   - role-aware `user prompts observed`
   - checkpoint ordinals
   - command observations with read/write/verification flags
4. Because the repo already treats checkpoint-contract growth as explicit schema events, `R3`
   should widen analyzer checkpoints to schema `v0.4` instead of silently adding new fields under
   `v0.3`.
5. Sentinel replay/live consumers still need to load legacy `v0.3` checkpoints after `R3` lands,
   so `R3` must preserve `v0.3` compatibility while preferring `v0.4` when turn context is
   present.
6. Turn context must be analyzer-computed and deterministic. `R3` should not invoke a model or
   depend on prompt-based classification to determine turn-local state.
7. The useful lesson from external work such as AgentLens is history-aware interpretation, not
   wholesale adoption of task-level multi-trajectory scoring, PTA references, or per-action
   cognitive-stage labels as the canonical `R3` output.
8. The useful lesson from dialogue-state-tracking work is to maintain an explicit evolving state
   object, not to make free-form natural language the source of truth.
9. Replay/live consumers should be able to inspect turn-local context directly after `R3`, but
   that inspection can remain presentation-only in this packet; scheduler policy, posture
   classification, and warning visibility should not change.

## Objective

Add analyzer-owned per-checkpoint turn context so the hybrid-drift stack can distinguish one long
agent-side turn from many short conversational turns without yet imposing archetype or progress
meaning on that structure.

Primary users:

- the maintainer reading analyzer checkpoints or replay/live sentinel output and needing explicit
  turn-local state instead of inferring it from summary-only ratios
- the later `R4` / `R5` packet author who needs a stable machine-readable turn-context seam before
  adding archetype or progress semantics

Success means:

- every analyzer checkpoint exported under schema `v0.4` carries structured `turn_context`
- `turn_context` is derived from existing deterministic row and command evidence
- analyzer summary output exposes turn-local context in a compact operator-facing form
- sentinel replay/live consumers accept `v0.4` checkpoints, preserve `v0.3` compatibility, and
  surface the same compact turn-context view without changing posture or scheduler behavior
- the packet proves the difference between:
  - one long agent-side turn with many checkpoints
  - many short conversational turns with fewer checkpoints per turn
- `R3` does not widen into `R4` archetype, `R5` progress, or `R6` scorer cutover

## Packet Boundary

This packet is **checkpoint-local turn-context promotion only**.

In scope:

- define one analyzer-owned `TurnContext` contract for every checkpoint
- define one deterministic `TurnActivityMix` contract summarizing the current turn
- define one conservative `TurnExecutionMode` derived from current-turn evidence
- widen analyzer checkpoint export from schema `v0.3` to `v0.4`
- render compact turn-context information in analyzer `summary.md`
- preserve replay/live loading by extending sentinel compatibility from `v0.2 | v0.3` to
  `v0.2 | v0.3 | v0.4`
- surface compact turn-context output in replay/live operator presentation without changing
  posture, severity, or trigger logic
- add focused analyzer and sentinel tests that lock the `R3` contract

Out of scope:

- session archetype classification such as troubleshooting vs planning vs review / closeout
- progress semantics such as advancing failure frontier, narrowing candidate set, or
  verification-wall movement
- any `dead_end_thrash`, `wrong_plan_branch`, or `truth_grounding_gap` scoring changes
- AgentLens-style per-action intent-stage labeling as a first-class artifact
- task-level PTA references or multi-trajectory comparison
- LLM- or prompt-generated turn summaries
- scheduler-policy changes, warning-threshold changes, or live-runtime coordinator redesign
- sentinel interpretation consolidation beyond the narrow compatibility/presentation work needed to
  expose `v0.4` turn context

## Interpretation Of External Patterns

### AgentLens

Adopt:

- history-aware interpretation instead of tool-identity-only interpretation
- the insight that read/write/verification behavior can mean different things depending on the
  surrounding trajectory

Reject for `R3`:

- per-action `Exploration` / `Implementation` / `Verification` / `Orchestration` label streams as
  the canonical artifact
- PTA references built from multiple passing solutions
- Lucky / Solid / Ideal quality tiers
- task-level divergence localization

`R3` interpretation:

- use AgentLens only as support for a coarse checkpoint-local view such as `turn_activity_mix` and
  `turn_execution_mode`
- keep the output observational and local to one checkpoint, not evaluative across many
  trajectories

### Function-Calling Dialogue State Tracking

Adopt:

- keep an explicit evolving state object rather than reconstructing turn meaning from scratch in
  every downstream consumer

Reject for `R3`:

- function-schema modeling
- ontology-driven slot/value state
- LLM-generated state extraction
- prompt engineering as the primary state-construction mechanism

`R3` interpretation:

- `TurnContext` is a deterministic analyzer-built state object over existing telemetry, not a model
  inference product

### Natural-Language Dialogue State Tracking

Adopt:

- optional interpretability value from a compact human-readable state summary derived from
  structured fields

Reject for `R3`:

- natural-language state descriptions as the canonical checkpoint contract
- free-form text as the source of truth for replay/live consumers

`R3` interpretation:

- if a prose turn-context summary exists, it is derived from structured fields and remains
  presentation-only

## Tech Stack

- Language: Rust 2021
- Target crates:
  - `crates/agent-drift-analyzer`
  - `crates/agent-drift-sentinel`
- Existing analyzer input contract:
  - compactor bundle `v0.2`
- Existing analyzer checkpoint contract:
  - `v0.3`
- New analyzer checkpoint contract owned by `R3`:
  - `v0.4`
- Existing sentinel compatibility:
  - `v0.2 | v0.3`
- Required sentinel compatibility after `R3`:
  - `v0.2 | v0.3 | v0.4`

No new crate or external dependency is required by default.

## Commands

Formatting gate:

```bash
cargo fmt --all -- --check
```

Analyzer-focused validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Sentinel replay/live compatibility validation:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Optional targeted wall while iterating on the checkpoint contract:

```bash
cargo test -p agent-drift-analyzer checkpoints_are_deterministic_and_session_scoped -- --nocapture
cargo test -p agent-drift-analyzer export_bundle_writes_checkpoints_and_summary -- --nocapture
cargo test -p agent-drift-sentinel replay_input_loads_and_sorts_v0_3_checkpoints -- --nocapture
```

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Owns checkpoint DTOs and the exported checkpoint schema contract. R3 adds `TurnContext`,
  `TurnActivityMix`, and `TurnExecutionMode` here.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Owns checkpoint analysis and checkpoint construction. R3 should compute current-turn slices and
  attach turn context during checkpoint assembly.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Owns `summary.md`. R3 should render compact turn-context information here without replacing the
  existing top-level/session metrics.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Deterministic checkpoint-contract coverage. R3 should add turn-context field assertions here.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary-contract coverage. R3 should lock the operator-facing turn-context rendering here.

crates/agent-drift-analyzer/tests/end_to_end.rs
  End-to-end artifact stability. R3 should prove reruns preserve `v0.4` checkpoints and summary.

crates/agent-drift-sentinel/src/input.rs
  Replay checkpoint loading and schema-version compatibility.

crates/agent-drift-sentinel/src/live_input.rs
  Live checkpoint compatibility and fixture validation.

crates/agent-drift-sentinel/src/operator_surface.rs
  Replay/live operator presentation. R3 should expose compact turn-context inspection here without
  changing posture classification.

crates/agent-drift-sentinel/tests/replay_input.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
  Sentinel regression walls that lock schema compatibility and replay/live presentation parity.

docs/specs/agent-drift-analyzer-turn-context-r3-*.md
  Spec, plan, and task authority for Packet `R3`.
```

## Code Style

Keep the source of truth structured and deterministic. If a field is interpretive, make the
interpretation explicit and conservative instead of hiding it in prose.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnContext {
    pub turn_id: Option<String>,
    pub turn_ordinal: usize,
    pub rows_since_turn_start: usize,
    pub seconds_since_turn_start: Option<i64>,
    pub checkpoints_in_turn: usize,
    pub prompts_observed_in_session: usize,
    pub execution_mode: TurnExecutionMode,
    pub activity_mix: TurnActivityMix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnActivityMix {
    pub directive_row_count: usize,
    pub assistant_message_count: usize,
    pub tool_call_count: usize,
    pub read_like_command_count: usize,
    pub write_like_command_count: usize,
    pub verification_like_command_count: usize,
    pub tool_output_count: usize,
}
```

Conventions:

- keep `TurnContext` analyzer-owned
- make field names literal and non-metaphorical
- prefer explicit `Option` for unavailable timing data instead of fake zeroes
- preserve the already-landed `turns observed` and `user prompts observed` definitions
- derive presentation strings from structured fields, never the reverse
- bias ambiguous turn-execution interpretation toward a conservative `mixed`/non-committal mode

## Locked Turn-Context Definitions

### Current Turn Slice

For each emitted checkpoint, define the current turn as the maximal suffix of rows ending at the
checkpoint boundary where:

- rows belong to the latest observed non-null `turn_id` at or before the checkpoint boundary
- trailing rows with null `turn_id` remain part of that suffix until a different non-null
  historical `turn_id` is encountered

If the checkpoint has never observed a non-null `turn_id`:

- `turn_id = null`
- `turn_ordinal = 0`
- turn-local counts still compute from the full current checkpoint interval/window as applicable

### Field Semantics

- `turn_id`
  - latest observed non-null `turn_id` at or before the checkpoint boundary, if present
- `turn_ordinal`
  - 1-based count of distinct non-null turn ids observed in session order through the current turn
- `rows_since_turn_start`
  - count of archival rows in the current turn slice
- `seconds_since_turn_start`
  - non-negative whole-second difference between the first timestamped row and the boundary-end
    row inside the current turn slice
  - unavailable when the current turn slice lacks either timestamp
- `checkpoints_in_turn`
  - count of emitted checkpoints whose boundary-end row falls within the current turn slice's
    `turn_id`
- `prompts_observed_in_session`
  - existing role-aware session metric already defined by checkpoint-calibration `v0.2`
- `activity_mix.directive_row_count`
  - focusable directive rows in the current turn slice
- `activity_mix.assistant_message_count`
  - assistant-message rows in the current turn slice
- `activity_mix.tool_call_count`
  - tool-call rows in the current turn slice
- `activity_mix.read_like_command_count`
  - command observations in the current turn slice where `read_like = true`
- `activity_mix.write_like_command_count`
  - command observations in the current turn slice where `write_like = true`
- `activity_mix.verification_like_command_count`
  - command observations in the current turn slice where `verification_like = true`
- `activity_mix.tool_output_count`
  - tool-output rows in the current turn slice

### `TurnExecutionMode`

`TurnExecutionMode` is a coarse checkpoint-level interpretation of the current turn, not a
per-action stage label stream.

Initial modes:

- `conversational`
  - no tool calls in the current turn, or directive/assistant rows dominate with at most one
    checkpoint in the turn
- `autonomous`
  - tool calls are present and the turn has emitted multiple checkpoints while remaining in the
    same turn id
- `verification_heavy`
  - verification-like commands are the strict plurality of turn-local command observations
- `mixed`
  - deterministic fallback when none of the above conditions holds cleanly

`R3` must keep this conservative:

- do not encode troubleshooting/planning/review semantics here
- do not treat `TurnExecutionMode` as a progress or quality judgment

## Testing Strategy

Frameworks:

- unit tests beside turn-context helpers when useful
- integration tests in `crates/agent-drift-analyzer/tests/` and
  `crates/agent-drift-sentinel/tests/`

Required test layers:

1. Analyzer checkpoint-contract tests
   - prove `v0.4` checkpoints serialize deterministic `turn_context`
   - prove `turn_ordinal`, `rows_since_turn_start`, and `checkpoints_in_turn` distinguish:
     - one long autonomous turn
     - many short conversational turns
2. Analyzer summary-contract tests
   - prove `summary.md` renders compact turn-context information without regressing existing
     summary metrics
3. Analyzer rerun stability tests
   - prove repeated analysis of the same bundle produces identical `v0.4` checkpoint and summary
     artifacts
4. Sentinel replay/live compatibility tests
   - prove replay and live loaders accept `v0.4`
   - prove `v0.3` compatibility remains intact
5. Sentinel presentation parity tests
   - prove replay and live surfaces expose the same compact turn-context view for matching
     checkpoints
   - prove posture and visibility stay unchanged when only turn-context fields are added

## Boundaries

- Always:
  - keep `R3` observational and checkpoint-local
  - widen the checkpoint schema explicitly instead of silently extending `v0.3`
  - preserve existing `turns observed` and `user prompts observed` semantics
  - derive turn context only from landed analyzer inputs and deterministic helpers
  - preserve `v0.3` replay/live compatibility while preferring `v0.4`
  - keep replay/live presentation changes narrow and non-behavioral
- Ask first:
  - using a model or prompt to infer `TurnExecutionMode`
  - adding any field that sounds like archetype or progress rather than turn structure
  - changing scheduler policy, warning thresholds, or checkpoint posture rules
  - dropping `v0.3` compatibility instead of carrying a narrow fallback
- Never:
  - ship AgentLens-style PTA scoring or per-action stage labels as the canonical `R3` output
  - let `R3` change drift scores or warning posture
  - make a prose summary authoritative over structured `turn_context`
  - broaden into `R4`, `R5`, or `R6` work under the `R3` label

## Success Criteria

1. Analyzer checkpoints widen from `v0.3` to `v0.4` and every `v0.4` checkpoint carries
   structured `turn_context`.
2. `turn_context` fields distinguish one long agent-side turn from many short conversational
   turns on deterministic analyzer tests.
3. Analyzer `summary.md` exposes compact turn-context information without redefining the landed
   summary metrics from checkpoint-calibration `v0.2`.
4. Sentinel replay and live inputs accept `v0.4` checkpoints while preserving legacy `v0.3`
   compatibility.
5. Replay/live operator surfaces expose the same compact turn-context inspection view for matching
   checkpoints.
6. `R3` lands without changing drift-score calculation, checkpoint posture classification,
   scheduler triggers, or visibility thresholds.

## Open Questions

1. Should replay/live operator presentation render turn context as an extension of
   `diagnostics_summary`, or as a separate compact line beneath diagnostics?
2. Should the first `R3` presentation include a derived human-readable turn summary string, or
   should it render only structured field excerpts until `R4` clarifies downstream semantics?
3. If a checkpoint has no non-null `turn_id`, should `checkpoints_in_turn` count checkpoints in
   the current checkpoint interval/window only, or all checkpoints since session start with
   `turn_id = null`? The conservative default in this spec is the current turn/window only unless
   implementation evidence shows a better deterministic rule.
