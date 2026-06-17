# Tasks: Agent Drift Analyzer Checkpoint Diagnostics v0.3

This task list implements:

- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-plan.md`

## Task List

## Packet v0.3A: Checkpoint Contract And Checkpoint-Local Diagnostics

- [ ] Task: Lock the `v0.3` diagnostics contract, schema shape, and render shape in repo docs
  - Acceptance: the spec/plan/task chain defines exact semantics, top-level aggregation rules, and
    unavailable behavior for `Flagged checkpoint rate`, `Drift-class flagged frequency`,
    `Task-frame transition count`, `Task-frame confidence distribution`, `Working-set churn`,
    `Verification density`, and `Average evidence items per checkpoint`, and it explicitly locks the
    compact checkpoint diagnostics payload plus schema-version bump.
  - Verify: doc review against the live analyzer files
    `crates/agent-drift-analyzer/src/checkpoint/export.rs`,
    `crates/agent-drift-analyzer/src/checkpoint/schema.rs`,
    `crates/agent-drift-analyzer/src/checkpoint/mod.rs`,
    `crates/agent-drift-analyzer/src/context/working_set.rs`,
    `crates/agent-drift-analyzer/src/inference/mod.rs`, and
    `crates/agent-drift-analyzer/src/scoring/*.rs`
  - Files:
    - `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-spec.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-plan.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-tasks.md`

- [ ] Task: Widen the checkpoint schema with a compact diagnostics payload
  - Acceptance: `Checkpoint` export rows bump schema version and serialize a compact `diagnostics`
    object carrying:
    - `task_frame_transitioned`
    - `working_set_changed`
    - `interval_command_count`
    - `interval_verification_command_count`
    - `evidence_item_count`
    without changing compactor input contracts or adding new drift classes.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task: Add checkpoint-derived diagnostics helpers in `checkpoint/export.rs`
  - Acceptance: exporter helpers can compute, from emitted checkpoints and their diagnostics:
    - flagged checkpoint rate
    - per-drift-class flagged counts and rates
    - task-frame transition count
    - task-frame confidence distribution
    - working-set churn from checkpoint snapshot changes
    - average evidence items per checkpoint with evidence deduplication
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

Packet `v0.3A` exit condition:

- `checkpoints.jsonl` bumps to `schema_version = "v0.2"` with the compact diagnostics payload
- checkpoint-local diagnostics are exported and covered by targeted tests
- no interval-derived summary work is required to declare the packet complete

## Packet v0.3B: Interval Metrics, Summary, And Bounded Proof

- [ ] Task: Reconstruct interval rows for verification density and persist interval counters
  - Acceptance: verification density is computed from newly introduced `compact_rows` between
    adjacent checkpoint boundaries rather than from full cumulative checkpoint windows, it uses the
    existing `collect_command_observations` plus `verification_like` heuristic without changing the
    compactor contract, and it persists the interval counts onto each checkpoint diagnostics
    payload.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/src/context/working_set.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task: Render the compact diagnostics slice in `summary.md`
  - Acceptance: the summary preserves the existing `v0.2` metric block, adds the agreed `v0.3`
    diagnostics lines at both the top level and per-session level, keeps stable ordering, renders
    zero-denominator cases as `unavailable` instead of fake zeroes, and stays aligned with the
    machine-readable checkpoint diagnostics payload.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task: Extend targeted analyzer tests for schema, aggregation, interval, and evidence edge cases
  - Acceptance: targeted tests prove:
    - checkpoint JSONL rows serialize the compact diagnostics payload and bumped schema version
    - weighted metrics are computed from real numerators/denominators rather than from averaged
      session values
    - verification density does not double-count cumulative checkpoint windows
    - confidence-distribution counts match emitted checkpoints
    - evidence deduplication prevents inflated average-evidence counts
    - unavailable metrics render explicitly when adjacent pairs or command observations are missing
  - Verify:
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer context_assembly -- --nocapture`
    - `cargo test -p agent-drift-analyzer input_contract -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`
    - `crates/agent-drift-analyzer/tests/context_assembly.rs`
    - `crates/agent-drift-analyzer/tests/input_contract.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`

- [ ] Task: Validate the new export and summary on a bounded current-schema analyzer smoke
  - Acceptance: a bounded run against a freshly generated `v0.2` compactor bundle renders the new
    diagnostics slice in `summary.md`, emits the compact diagnostics payload in
    `checkpoints.jsonl`, and remains concise and reviewable.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"`
    - `sed -n '1,140p' "$ANALYZER_OUT/summary.md"`
    - `sed -n '1,20p' "$ANALYZER_OUT/checkpoints.jsonl"`
  - Files:
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`
    - `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

- [ ] Task: Refresh continuity docs if the live export contract makes older packet notes stale
  - Acceptance: any repo docs that still describe the analyzer as if the new diagnostics only exist
    in `summary.md` are updated to mention the landed `v0.3` checkpoint diagnostics payload and its
    analyzer-only scope.
  - Verify: doc review against the final rendered summary shape, the checkpoint payload, and the
    current live analyzer files
  - Files:
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-spec.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-plan.md`

Packet `v0.3B` exit condition:

- interval-derived verification metrics are exported without cumulative-window double counting
- `summary.md` renders the full `v0.3` diagnostics slice
- bounded smoke proof and continuity-doc refresh are complete
