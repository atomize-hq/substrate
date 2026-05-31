# Tasks: Agent Drift Analyzer Checkpoint Calibration v0.2

This task list implements:

- [agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md:1)
- [agent-drift-analyzer-checkpoint-calibration-v0.2-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-plan.md:1)

## Task List

- [x] Task: Lock the checkpoint-calibration metric contract
  - Acceptance: the repo docs define exact semantics for `turns observed`, role-aware `user prompts observed`, checkpoint-density ratios, row/time spacing metrics, and flagged-streak counting.
  - Verify: doc review against the spec and plan before code changes start
  - Files: `docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md`, `docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-plan.md`

- [x] Task: Add deterministic session summary metric helpers
  - Acceptance: the analyzer can compute per-session turn counts, role-aware prompt counts, checkpoints-per-turn, checkpoints-per-user-prompt, and longest flagged streaks from existing bundle and checkpoint surfaces.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Compute checkpoint interval metrics from adjacent boundaries
  - Acceptance: the analyzer reports average rows-between-checkpoints and average seconds-between-checkpoints using adjacent checkpoint boundaries rather than cumulative window sizes.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` and `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, `crates/agent-drift-analyzer/tests/checkpoints.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Add per-session diagnostic metrics from existing analyzer context
  - Acceptance: each session block reports distinct task-frame count, truth artifacts referenced, verification commands observed, and remains compatible with the already-landed prompt/steer/unknown supporting counts without widening the checkpoint JSONL contract.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/src/lib.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Render the operator summary in the agreed compact format
  - Acceptance: `summary.md` includes the selected top-level and per-session metrics, stays concise, and renders unavailable metrics explicitly when timestamps or denominators are missing.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Validate the checkpoint-calibration summary on the bounded real-session smoke
  - Acceptance: the known smoke session `019e767c-e64b-7b93-a540-7a33a90f780f` renders the new calibration metrics successfully and the output remains deterministic across reruns.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture` and `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"`
  - Files: `crates/agent-drift-analyzer/tests/end_to_end.rs`, `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

- [x] Task: Document next-level blocked metrics and contract gaps
  - Acceptance: the docs explicitly record which proposed follow-up metrics still need richer normalization or a widened analyzer contract, instead of leaving those gaps implicit.
  - Verify: doc review against the spec/plan/task chain and implementation-order doc
  - Files: `docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-spec.md`, `docs/specs/agent-drift-analyzer-checkpoint-calibration-v0.2-plan.md`, `docs/specs/hybrid-drift-sentinel-implementation-order.md`
