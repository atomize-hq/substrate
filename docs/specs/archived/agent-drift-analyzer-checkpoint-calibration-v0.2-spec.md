# Spec: Agent Drift Analyzer Checkpoint Calibration v0.2

## Assumptions I'm Making

1. The current `agent-drift-analyzer` checkpoint JSONL contract is usable enough to calibrate
   checkpoint usefulness without widening the replay-facing schema first.
2. The immediate product need is operator-facing interpretability, not a new drift class or a new
   live-runtime behavior.
3. The current compactor/analyzer surfaces already expose enough evidence to compute meaningful
   checkpoint-density, spacing, and coverage metrics from rows, boundaries, timestamps, context,
   and drift scores.
4. `turn_id` is still a useful but imperfect proxy for conversational turns, but the landed
   `user_message_role: prompt | steer | unknown` classifier now lets this slice define
   `user prompts observed` narrowly as `prompt` rows while still reporting supporting steer/unknown
   counts.
5. The first slice should stay summary-first and deterministic; it should not add model
   adjudication, phase classification, or new compactor event kinds as prerequisites.
6. If a desired metric cannot be computed reliably from the landed row contract, the correct
   response is to omit it or mark it unavailable, not infer it from weak heuristics and present it
   as truth.

## Objective

Extend `agent-drift-analyzer` so its exported operator summary is useful for calibrating whether
checkpoint generation is sparse and semantic or merely chatty.

Primary user:

- the operator or engineer reviewing analyzer output and deciding whether current checkpoint
  segmentation and warning density reflect meaningful progress boundaries

Success means:

- the summary makes checkpoint density legible at a glance
- the summary exposes whether checkpoints are clustering around turns, prompt rows, rows, or time
- the summary surfaces enough session-level diagnostics to judge whether further checkpoint tuning
  should focus on cadence, task-frame stability, truth-grounding coverage, or warning thresholds

## Tech Stack

- Language: Rust 2021
- Product shape: extension of the existing library-first `agent-drift-analyzer` crate
- Existing input contract:
  - `manifest.json`
  - `rows.archival.jsonl`
  - `rows.compact.jsonl`
  - `dedupe-audit.jsonl`
  - analyzer-generated `Checkpoint` values
- Existing output contract:
  - `checkpoints.jsonl`
  - `summary.md`
- Supporting dependencies already in use:
  - `serde`
  - `serde_json`
  - `thiserror`
  - `camino`
  - `time`

Dependency posture:

- no requirement to widen compactor output before this slice
- no requirement to change sentinel replay/live contracts before this slice
- no default machine-learning or model dependency

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted crate validation:

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Bounded real-session proof:

```bash
export SESSION_ID=019e767c-e64b-7b93-a540-7a33a90f780f
export SMOKE_ROOT=target/hybrid-drift-smoke/$SESSION_ID
export COMPACTOR_OUT=$SMOKE_ROOT/compactor
export ANALYZER_OUT=$SMOKE_ROOT/analyzer

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
```

## Project Structure

```text
crates/agent-drift-analyzer/src/lib.rs
  Threads the loaded bundle into checkpoint export.

crates/agent-drift-analyzer/src/input.rs
  Defines session-scoped bundle inputs and stable row ordering.

crates/agent-drift-analyzer/src/context/
  Provides truth-artifact, working-set, and verification-command surfaces reused by calibration.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Defines checkpoint windows and stable checkpoint boundaries.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Owns summary metrics, summary rendering, and checkpoint bundle export.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary-contract regression coverage.

docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-*.md
  Spec, plan, and task chain for this calibration slice.
```

## Code Style

Keep metrics explicit and deterministic. Metrics should be named after what they actually measure,
not what we hope they approximate.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSummaryMetrics {
    pub turns_observed: usize,
    pub user_prompts_observed: usize,
    pub checkpoints_emitted: usize,
    pub checkpoints_per_turn: Option<f64>,
    pub checkpoints_per_user_prompt: Option<f64>,
    pub avg_rows_between_checkpoints: Option<f64>,
    pub avg_seconds_between_checkpoints: Option<f64>,
    pub longest_flagged_streak: usize,
}
```

Conventions:

- preserve deterministic ordering
- prefer `Option` for unavailable metrics instead of fake zeroes
- compute from stable bundle fields and checkpoint boundaries before introducing new heuristics
- treat `user prompts observed` as role-aware `prompt` rows, not all `UserMessage` rows
- keep the summary concise enough for operators to scan during replay review

## Locked Metric Definitions

- `turns observed`
  - per session: count distinct non-null `turn_id` values across `archival_rows`
  - top level: sum per-session `turns observed`
- `user prompts observed`
  - per session: count `compact_rows` where `kind = user_message` and `user_message_role = prompt`
  - top level: sum per-session `user prompts observed`
- `checkpoints emitted`
  - per session: count emitted analyzer checkpoints for that session
  - top level: sum per-session `checkpoints emitted`
- `checkpoints per turn`
  - `checkpoints emitted / turns observed`
  - unavailable when `turns observed = 0`
- `checkpoints per user prompt`
  - `checkpoints emitted / user prompts observed`
  - unavailable when `user prompts observed = 0`
- `avg rows between checkpoints`
  - compute adjacent gaps only from successive checkpoint `boundary.end` row positions within a
    session
  - use session archival ordering as the source of truth; fall back to compact-row ordering only
    when a boundary row is absent from archival rows
  - top level: weighted average over all session-local adjacent checkpoint gaps
- `avg seconds between checkpoints`
  - compute adjacent gaps only from successive checkpoint `boundary.end` timestamps within a
    session
  - include only adjacent pairs where both boundary rows carry timestamps
  - unavailable when no adjacent timestamped pair exists
  - top level: weighted average over all session-local adjacent timestamped checkpoint gaps
- `flagged checkpoints`
  - count checkpoints where `flagged = true`
- `longest flagged streak`
  - per session: max contiguous run of flagged checkpoints in ordinal order
  - top level: max per-session `longest flagged streak`
- `distinct task frames`
  - count distinct inferred task-frame identities across a session's checkpoints
  - identity uses `objective`, `truth_artifacts`, `working_set_paths`, `tools`,
    `command_families`, and `verification_commands`
  - identity intentionally excludes confidence and evidence vectors so incidental support growth
    does not create a new frame by itself
- `truth artifacts referenced`
  - count distinct `task_frame.truth_artifacts` across a session's checkpoints
- `verification commands observed`
  - count distinct `task_frame.verification_commands` across a session's checkpoints

## Testing Strategy

Frameworks:

- unit tests beside summary-metric helpers
- integration tests in `crates/agent-drift-analyzer/tests/`

Required test layers:

1. Summary-contract tests
   - top-level summary includes the new checkpoint-calibration fields
   - per-session blocks include the new session diagnostics
2. Determinism tests
   - repeated runs on the same bundle produce identical metric lines
   - missing timestamps or empty denominators degrade to explicit unavailable metrics
3. Boundary-metric tests
   - rows-between-checkpoints and seconds-between-checkpoints use checkpoint boundaries rather than
     cumulative row counts
4. Context-metric tests
   - truth-artifact counts, distinct task frames, and verification-command counts reflect the
     current analyzer context surfaces
5. Bounded real-session proof
   - the known smoke session summary renders the new metric block successfully

## Boundaries

- Always:
  - report both `turns observed` and `user prompts observed`
  - derive `user prompts observed` from `user_message_role = prompt`
  - keep prompt/steer/unknown counts available as supporting diagnostics
  - compute checkpoint spacing from stable checkpoint boundaries and row timestamps
  - preserve the current `checkpoints.jsonl` contract unless a separate schema decision is made
  - keep unavailable metrics explicit rather than silently guessing
- Ask first:
  - widening `Checkpoint` with new fields for calibration
  - adding a new machine-readable calibration artifact beyond `summary.md`
  - adding phase-duration metrics that need new phase classification semantics
  - widening compactor normalization with first-class `file_read`, `file_edit`, or
    `tool_call_completed` event kinds
- Never:
  - present `turn_id` as synonymous with user prompts
  - use cumulative checkpoint-window row counts as if they were interval counts
  - distort the summary into a verbose audit dump that hides the primary ratios

## Success Criteria

The spec is satisfied when:

1. The analyzer summary reports these top-level metrics:
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
   and may also retain supporting `Prompt/Steer/Unknown user messages` counts if they remain
   helpful to operator review.
2. Each session block reports:
   - `Turns observed`
   - `User prompts observed`
   - `Checkpoints emitted`
   - `Checkpoints per turn`
   - `Checkpoints per user prompt`
   - `Avg rows between checkpoints`
   - `Avg seconds between checkpoints`
   - `Flagged checkpoints`
   - `Distinct task frames`
   - `Truth artifacts referenced`
   - `Verification commands observed`
   - `Longest flagged streak`
3. The summary remains deterministic across repeated runs on the same bundle.
4. The summary does not require widening the replay-facing checkpoint JSONL contract.
5. The slice documents which next-level metrics remain blocked on richer normalization or new
   analyzer semantics.

## Open Questions

1. Should the first slice include only averages, or also median/min/max for checkpoint spacing?
2. Should a follow-up slice collapse `distinct task frames` onto a narrower normalized frame label
   once operators have more calibration feedback?
3. Should a follow-up slice add a separate machine-readable calibration artifact once the operator
   summary shape stabilizes?
4. Which next-level metrics are worth a compactor-contract widening versus a pure analyzer follow-up?
