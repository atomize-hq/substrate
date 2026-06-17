# Tasks: Agent Drift Analyzer v0.1

This task list implements:

- [agent-drift-analyzer-v0.1-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-v0.1-spec.md:1)
- [agent-drift-analyzer-v0.1-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-analyzer-v0.1-plan.md:1)

## Task List

- [x] Task: Scaffold the library-first crate and thin binary entrypoint
  - Acceptance: `agent-drift-analyzer` exists as a workspace member with `src/lib.rs` and a minimal `src/main.rs` that delegates to library-owned behavior.
  - Verify: `cargo build -p agent-drift-analyzer`
  - Files: `Cargo.toml`, `crates/agent-drift-analyzer/Cargo.toml`, `crates/agent-drift-analyzer/src/lib.rs`, `crates/agent-drift-analyzer/src/main.rs`

- [x] Task: Implement compactor artifact loading and contract checks
  - Acceptance: the analyzer loads `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, and `dedupe-audit.jsonl`, validates session scope, respects the landed bundle publication contract, and fails clearly on malformed input.
  - Verify: `cargo test -p agent-drift-analyzer input_contract -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/input.rs`, `crates/agent-drift-analyzer/src/lib.rs`, `crates/agent-drift-analyzer/tests/input_contract.rs`

- [x] Task: Gate the compactor artifact surface before analyzer heuristics
  - Acceptance: the analyzer no longer needs to guess around missing row information; if the landed row contract is not sufficient for useful working-set inference, the implementation pauses for that compactor-contract decision rather than encoding distorted assumptions.
  - Verify: `cargo test -p agent-drift-analyzer input_contract -- --nocapture`
  - Files: `docs/specs/agent-drift-analyzer-v0.1-plan.md`, `docs/specs/agent-drift-analyzer-v0.1-spec.md`, `docs/specs/agent-session-compactor-v0.1-spec.md`

- [x] Task: Implement deterministic context assembly
  - Acceptance: the analyzer derives objective, candidate truth artifacts, working-set paths, tools, and command families from the landed row kinds, dedupe identities, and payload text in a stable way.
  - Verify: `cargo test -p agent-drift-analyzer context_assembly -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/context/mod.rs`, `crates/agent-drift-analyzer/src/context/objective.rs`, `crates/agent-drift-analyzer/src/context/working_set.rs`, `crates/agent-drift-analyzer/tests/context_assembly.rs`

- [x] Task: Implement task-frame inference and confidence shaping
  - Acceptance: the analyzer emits deterministic task-frame hypotheses, confidence levels, and counter-evidence without requiring a plan artifact.
  - Verify: `cargo test -p agent-drift-analyzer task_frame -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/inference/mod.rs`, `crates/agent-drift-analyzer/src/inference/task_frame.rs`, `crates/agent-drift-analyzer/tests/task_frame.rs`

- [x] Task: Implement `wrong_plan_branch` scoring
  - Acceptance: branch-divergence scoring uses task-frame and working-set evidence, with explicit thresholds and evidence refs.
  - Verify: `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/scoring/mod.rs`, `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`, `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs`

- [x] Task: Implement `ignoring_repo_truth` scoring
  - Acceptance: truth-grounding scoring uses candidate truth artifacts and validation-phase evidence, with explicit thresholds and evidence refs.
  - Verify: `cargo test -p agent-drift-analyzer ignoring_repo_truth -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/scoring/mod.rs`, `crates/agent-drift-analyzer/src/scoring/ignoring_repo_truth.rs`, `crates/agent-drift-analyzer/tests/ignoring_repo_truth.rs`

- [x] Task: Implement `dead_end_thrash` scoring over repetition-preserving evidence
  - Acceptance: thrash scoring reads archival or repetition-preserving evidence rather than compacted-only rows, with explicit repeated-failure and repeated-command coverage.
  - Verify: `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/scoring/mod.rs`, `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`, `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`

- [x] Task: Implement checkpoint segmentation and checkpoint contract
  - Acceptance: deterministic checkpoint boundaries are emitted with task frame, drift scores, evidence refs, and expected-next-step fields.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, `crates/agent-drift-analyzer/src/checkpoint/schema.rs`, `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task: Implement summary and output bundle export
  - Acceptance: the analyzer writes `checkpoints.jsonl` and `summary.md`, and any extra machine-readable artifact is justified and documented rather than added casually.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/src/lib.rs`, `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Wire the thin CLI to library behavior
  - Acceptance: the CLI accepts documented input/output paths, validates session inputs, and delegates to library-owned analyzer behavior.
  - Verify: `cargo run -p agent-drift-analyzer -- --input-dir target/agent-session-compactor/session-<session-id> --output-dir target/agent-drift-analyzer/session-<session-id>`
  - Files: `crates/agent-drift-analyzer/src/main.rs`, `crates/agent-drift-analyzer/src/cli.rs`, `crates/agent-drift-analyzer/src/lib.rs`

- [x] Task: Gate end-to-end validation and freeze the checkpoint contract for replay consumers
  - Acceptance: analyzer outputs are deterministic and reviewable enough for replay-mode sentinel consumption.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files: `crates/agent-drift-analyzer/tests/end_to_end.rs`, `docs/specs/agent-drift-analyzer-v0.1-plan.md`, `docs/specs/agent-drift-sentinel-v0.2-spec.md`
