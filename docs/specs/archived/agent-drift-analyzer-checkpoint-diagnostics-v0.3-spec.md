# Spec: Agent Drift Analyzer Checkpoint Diagnostics v0.3

## Assumptions I'm Making

1. This is still an analyzer-only follow-up slice on top of the already-landed compactor `v0.2`
   bundle contract and analyzer loader; no compactor contract widening is required first.
2. `agent-drift-analyzer` should widen the replay-facing `checkpoints.jsonl` contract in this slice
   so replay or sentinel consumers can read the new diagnostics directly without scraping
   `summary.md`.
3. Most of the requested diagnostics are already derivable from the current checkpoint surface:
   - `Checkpoint.flagged`
   - `Checkpoint.drift_scores`
   - `Checkpoint.task_frame`
   - `TaskFrame.confidence`
   - `TaskFrame.working_set_paths`
   - `TaskFrame.supporting_evidence` and `counter_evidence`
4. `verification density` is the main requested metric that is not safely derivable from the
   current cumulative checkpoint snapshots alone; it needs compact per-checkpoint interval counters
   in the exported checkpoint schema.
5. `working_set_paths` and truth-artifact hints are heuristic path extractions from directive text
   and tool-call commands; metrics built on them must be framed as checkpoint-snapshot churn, not as
   a true semantic phase or work-area taxonomy.
6. `verification_like` is currently a command-family heuristic derived from observed tool-call
   commands, not proof of command success, test pass, or human acceptance.
7. If a proposed metric still needs stronger event normalization or stronger phase semantics after
   the checkpoint-schema widening below, that metric should stay blocked or render `unavailable`
   instead of being guessed.

## Objective

Extend `agent-drift-analyzer` so it exports compact checkpoint diagnostics through both:

- `checkpoints.jsonl` for machine-readable downstream use
- `summary.md` for operator-readable review

Primary user:

- the operator or engineer reviewing analyzer output and deciding whether current checkpoint
  warnings, cadence, and task-frame evolution are meaningful enough to trust downstream replay or
  sentinel behavior

Success means:

- the summary keeps the existing checkpoint-calibration metrics and adds a compact diagnostics block
  that explains why checkpoints are being flagged or changing
- the checkpoint JSONL rows carry the new diagnostics in a compact machine-readable form
- the new diagnostics stay honest about current heuristics and do not imply semantic phase/timeline
  guarantees the live crate does not have
- the analyzer does not need any compactor bundle changes to produce the first `v0.3` report

## Tech Stack

- Language: Rust 2021
- Target crate: `crates/agent-drift-analyzer`
- Existing upstream input contract: `agent-session-compactor` export schema `v0.2`
- Analyzer output contracts after this slice:
  - `checkpoints.jsonl`
  - `summary.md`
- Existing dependencies already sufficient for this slice:
  - `serde`
  - `serde_json`
  - `camino`
  - `time`
  - `thiserror`

Dependency posture:

- no new crate is required
- no compactor normalization redesign is required
- no compactor bundle schema change is required
- no sentinel-side prerequisite change is required before this analyzer slice can land

## Commands

Targeted crate validation:

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer context_assembly -- --nocapture
cargo test -p agent-drift-analyzer input_contract -- --nocapture
```

Focused analyzer verification:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Bounded current-schema smoke:

```bash
export SESSION_ID=019e767c-e64b-7b93-a540-7a33a90f780f
export SMOKE_ROOT=target/hybrid-drift-smoke/$SESSION_ID
export COMPACTOR_OUT=$SMOKE_ROOT/compactor
export ANALYZER_OUT=$SMOKE_ROOT/analyzer

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

sed -n '1,140p' "$ANALYZER_OUT/summary.md"
sed -n '1,20p' "$ANALYZER_OUT/checkpoints.jsonl"
```

## Project Structure

```text
crates/agent-session-compactor/src/export/mod.rs
  Defines the analyzer-facing v0.2 export DTOs and five-file bundle contract.

crates/agent-session-compactor/src/export/files.rs
  Builds the manifest-owned file registry and writes the compactor bundle.

crates/agent-drift-analyzer/src/input.rs
  Loads and validates the v0.2 compactor bundle, resolves file/turn ids, and guards analyzer
  surface sufficiency.

crates/agent-drift-analyzer/src/context/mod.rs
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/src/context/working_set.rs
  Assemble objective, truth-artifact, working-set, tool, and command-family heuristics.

crates/agent-drift-analyzer/src/inference/mod.rs
  Builds `TaskFrame`, including confidence and counter-evidence.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/checkpoint/schema.rs
crates/agent-drift-analyzer/src/checkpoint/export.rs
  Define checkpoint windows/schema and render both `checkpoints.jsonl` and `summary.md`.

crates/agent-drift-analyzer/src/scoring/
  Produces the three landed drift classes and per-checkpoint flagged state.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary/export contract regression coverage.

crates/agent-drift-analyzer/tests/context_assembly.rs
crates/agent-drift-analyzer/tests/input_contract.rs
crates/agent-drift-analyzer/tests/support/mod.rs
  Guard the context and input surfaces that this summary slice depends on.

docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-*.md
  Spec, plan, and task chain for this slice.
```

## Code Style

Keep every metric explicit about its numerator, denominator, and aggregation mode. The output
schema should carry only the fields that downstream consumers cannot already derive safely from the
current checkpoint row.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointDiagnostics {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
    pub interval_command_count: usize,
    pub interval_verification_command_count: usize,
    pub evidence_item_count: usize,
}
```

Conventions:

- preserve deterministic session order and stable drift-class order
- use `Option<f64>` for zero-denominator summary metrics and render them as `unavailable`
- use literal zero only when the metric is well-defined and the true result is zero
- keep task-frame identity aligned with the existing exporter identity tuple unless this spec is
  amended first
- keep the checkpoint diagnostics payload compact and additive
- keep the operator summary compact; add a diagnostics slice, not a verbose audit dump

## Diagnostic Metric Contract

The existing `v0.2` summary metrics remain unchanged:

- `Sessions analyzed`
- `Turns observed`
- `User prompts observed`
- `Checkpoints emitted`
- `Checkpoints per turn`
- `Checkpoints per user prompt`
- `Avg rows between checkpoints`
- `Avg seconds between checkpoints`
- `Flagged checkpoints`
- `Longest flagged streak`
- supporting `Prompt/Steer/Unknown user messages` counts

`v0.3` adds a compact checkpoint diagnostics payload and uses it to render new summary metrics.

### Checkpoint Contract Change

Each exported checkpoint row widens from the current `v0.1` schema to `v0.2` by adding:

- `schema_version = "v0.2"`
- `diagnostics: CheckpointDiagnostics`

The new diagnostics fields are:

- `task_frame_transitioned: bool`
  - `true` when this checkpoint's task-frame identity differs from the immediately previous
    checkpoint in the same session
  - `false` for the first checkpoint in a session
- `working_set_changed: bool`
  - `true` when this checkpoint's sorted distinct `task_frame.working_set_paths` set differs from
    the immediately previous checkpoint in the same session
  - `false` for the first checkpoint in a session
- `interval_command_count: usize`
  - number of command observations introduced by this checkpoint's interval, where the interval is
    the newly covered compact-row slice since the prior checkpoint in the same session
  - for the first checkpoint, the interval runs from session start through that checkpoint
- `interval_verification_command_count: usize`
  - subset of `interval_command_count` classified as verification-like by the current
    command-family heuristic
- `evidence_item_count: usize`
  - count of deduped evidence items local to this checkpoint after combining:
    - `task_frame.supporting_evidence`
    - `task_frame.counter_evidence`
    - all `drift_scores[*].evidence`

### Per-Session Summary Definitions

- `flagged checkpoint rate`
  - numerator: checkpoints where `Checkpoint.flagged = true`
  - denominator: checkpoints emitted for that session
  - unavailable when `checkpoints emitted = 0`
- `drift-class flagged frequency`
  - computed separately for `wrong_plan_branch`, `ignoring_repo_truth`, and `dead_end_thrash`
  - numerator: checkpoints where that class's `DriftScore.flagged = true`
  - denominator: checkpoints emitted for that session
  - unavailable when `checkpoints emitted = 0`
- `task-frame transition count`
  - count checkpoints where `diagnostics.task_frame_transitioned = true`
  - task-frame identity remains the existing tuple of:
    - `objective`
    - `truth_artifacts`
    - `working_set_paths`
    - `tools`
    - `command_families`
    - `verification_commands`
  - identity intentionally excludes confidence and evidence vectors
- `task-frame confidence distribution`
  - count checkpoints by `task_frame.confidence = low | medium | high`
  - unavailable only when the session has zero checkpoints
- `working-set churn`
  - numerator: checkpoints where `diagnostics.working_set_changed = true`
  - denominator: adjacent checkpoint pairs
  - unavailable when fewer than two checkpoints exist
  - this is checkpoint-snapshot churn only; it is not semantic work-area-shift modeling
- `verification density`
  - numerator: sum `diagnostics.interval_verification_command_count`
  - denominator: sum `diagnostics.interval_command_count`
  - verification-like uses the current command-family heuristic from `context/working_set.rs`
  - unavailable when the denominator is zero
- `average evidence items per checkpoint`
  - per-checkpoint input: `diagnostics.evidence_item_count`
  - per-session metric is the arithmetic mean across that session's checkpoints
  - unavailable when the session has zero checkpoints

### Top-Level Aggregation Rules

- summed across sessions:
  - `Turns observed`
  - `User prompts observed`
  - `Checkpoints emitted`
  - `Flagged checkpoints`
  - prompt/steer/unknown user-message counts
  - `task-frame transition count`
  - confidence-distribution bucket counts
  - per-drift-class flagged checkpoint counts
  - total interval command counts and total interval verification counts
- max across sessions:
  - `Longest flagged streak`
- weighted or ratio-based across the full checkpoint corpus:
  - `Checkpoints per turn`
  - `Checkpoints per user prompt`
  - `Avg rows between checkpoints`
  - `Avg seconds between checkpoints`
  - `flagged checkpoint rate`
  - each `drift-class flagged frequency`
  - `working-set churn`
  - `verification density`
  - `average evidence items per checkpoint`

### Trust And Non-Trust Boundaries

Trustworthy enough for `v0.3` export and summary use:

- whether a checkpoint was flagged
- which landed drift class flagged a checkpoint
- checkpoint-local task-frame confidence
- checkpoint-local truth-artifact and working-set snapshots
- command-family heuristics derived from real tool-call rows
- compact interval command counters exported directly on checkpoints

Not trustworthy enough to claim in this slice:

- semantic phase transitions
- time spent in a true phase taxonomy
- first grounding before first write
- first verification lag
- stronger work-area-shift modeling
- stronger repetition metrics beyond current evidence and command heuristics
- true file-read versus file-edit counts that need richer normalized event kinds
- tool success rate that needs stronger completion/outcome normalization

## Testing Strategy

Frameworks:

- integration tests in `crates/agent-drift-analyzer/tests/`
- helper-level tests inside `crates/agent-drift-analyzer/src/checkpoint/` only if the integration
  fixture cannot express the edge case cleanly

Required test layers:

1. Summary contract tests
   - top-level summary renders the new diagnostics lines
   - per-session blocks render the new diagnostics lines
2. Checkpoint contract tests
   - checkpoint JSONL rows serialize the new `diagnostics` object
   - checkpoint schema version bumps from `v0.1` to `v0.2`
3. Aggregation tests
   - summed, maxed, and weighted metrics follow the rules above
   - zero-denominator metrics render `unavailable` instead of `0.00`
4. Interval-slicing tests
   - `diagnostics.interval_command_count` and
     `diagnostics.interval_verification_command_count` use newly introduced compact-row intervals
   - verification density uses those interval counters rather than cumulative checkpoint windows
5. Heuristic-honesty tests
   - `diagnostics.task_frame_transitioned` reflects the documented task-frame identity change only
   - `diagnostics.working_set_changed` reflects checkpoint snapshot changes only
   - confidence distribution reports checkpoint counts, not inferred semantic states
6. Evidence-counting tests
   - duplicate evidence items do not inflate `diagnostics.evidence_item_count`
7. Bounded smoke proof
   - a current-schema analyzer run produces the new diagnostics block on `summary.md`
   - the emitted `checkpoints.jsonl` rows contain the compact diagnostics payload

## Boundaries

- Always:
  - keep this slice analyzer-only
  - preserve the compactor `v0.2` bundle contract
  - widen `checkpoints.jsonl` only with compact analyzer-local diagnostics that downstream consumers
    can rely on
  - name heuristic metrics literally and document what they do not prove
  - compute adjacent-pair metrics in session ordinal order
  - render `unavailable` for zero-denominator or missing-sample metrics
  - keep the summary concise and reviewable
- Ask first:
  - widening the compactor bundle contract
  - adding a new drift class
  - changing checkpoint segmentation logic
  - adding a second machine-readable diagnostics artifact beyond `checkpoints.jsonl` and
    `summary.md`
- Never:
  - claim semantic phase transitions or time-in-phase from the current data
  - treat working-set churn as a true workstream-shift model
  - present verification density as a success rate
  - infer file-read/file-edit counts that the current normalized event surface does not encode

## Success Criteria

This spec is satisfied when all of the following are true:

1. `summary.md` keeps the current `v0.2` metric block and adds a compact diagnostics slice with:
   - `Flagged checkpoint rate`
   - `Drift-class flagged frequency`
   - `Task-frame transition count`
   - `Task-frame confidence distribution`
   - `Working-set churn`
   - `Verification density`
   - `Average evidence items per checkpoint`
2. Each new metric has explicit per-session semantics, top-level aggregation rules, and
   zero-denominator behavior documented in repo docs.
3. `checkpoints.jsonl` bumps its schema version and carries a compact `diagnostics` object with:
   - `task_frame_transitioned`
   - `working_set_changed`
   - `interval_command_count`
   - `interval_verification_command_count`
   - `evidence_item_count`
4. The implementation does not add new drift classes or rework checkpoint segmentation.
5. Targeted analyzer tests prove:
   - weighted metrics are not computed by averaging pre-averaged session values
   - verification density does not double-count cumulative checkpoint windows
   - checkpoint JSONL serialization includes the new diagnostics payload
   - evidence counting dedupes repeated evidence items
   - unavailable metrics render explicitly
6. The docs explicitly state which seemingly related metrics remain blocked by current heuristic or
   normalization limits.

## Open Questions

None. The current repo state is sufficient to lock the `v0.3` scope without adding unresolved
requirements.
