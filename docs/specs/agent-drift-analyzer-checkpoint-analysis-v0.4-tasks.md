# Tasks: Agent Drift Analyzer Checkpoint Analysis v0.4

This task list implements:

- `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-plan.md`

## Task List

## Packet v0.4A: Internal CheckpointAnalysis Seam

- [x] Task: Lock the `v0.4` checkpoint-analysis contract in repo docs
  - Acceptance: the spec/plan/tasks chain explicitly locks:
    - the new internal `CheckpointAnalysis` seam
    - the four time surfaces: current, previous, interval, repetition
    - the class rename from `IgnoringRepoTruth` to `TruthGroundingGap`
    - the rule that `WrongPlanBranch` is interval-local
    - the rule that `TruthGroundingGap` is hybrid
    - the rule that `DeadEndThrash` uses explicit recovery semantics
  - Verify: doc review against the current live analyzer and sentinel crate surfaces
  - Files:
    - `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-spec.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-plan.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-tasks.md`

- [x] Task: Add the internal `CheckpointAnalysis` seam with no intended behavior change
  - Acceptance: the analyzer constructs `CheckpointAnalysis` for each checkpoint ordinal, the seam
    includes explicit current/previous/interval/repetition slices plus task-frame delta and
    recovery state scaffolding, and the first pass preserves current exported checkpoint behavior.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/lib.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Route checkpoint diagnostics through `CheckpointAnalysis`
  - Acceptance: `CheckpointDiagnostics` fields are derived from the explicit checkpoint-analysis
    time surfaces rather than a separate ad hoc path, and no exported diagnostics field changes in
    this packet.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

Packet `v0.4A` exit condition:

- `CheckpointAnalysis` exists and drives diagnostics
- exported checkpoint shape remains stable
- no scorer semantics are required to change yet

## Packet v0.4B: Current-State Scorers

- [ ] Task: Move `WrongPlanBranch` onto interval-local scope evidence
  - Acceptance: `WrongPlanBranch` reads the interval slice rather than cumulative prefix context,
    and later checkpoints clear after in-scope recovery even if earlier checkpoints were
    out-of-scope.
  - Verify:
    - `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs`

- [ ] Task: Rename `IgnoringRepoTruth` to `TruthGroundingGap`
  - Acceptance: the drift-class enum variant, scorer module naming, analyzer-facing strings, and
    tests all use `TruthGroundingGap`, and the new name is framed as a grounding heuristic rather
    than an intent claim.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/src/scoring/ignoring_repo_truth.rs`
    - `crates/agent-drift-analyzer/tests/ignoring_repo_truth.rs`
    - `crates/agent-drift-sentinel/src/operator_surface.rs`

- [ ] Task: Move `TruthGroundingGap` to hybrid semantics
  - Acceptance: the renamed class computes active state from the latest interval, preserves
    historical grounding-gap evidence with explicit historical reasons, and does not keep the
    active flag set solely because earlier intervals were ungrounded.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

Packet `v0.4B` exit condition:

- `WrongPlanBranch` is interval-local
- `TruthGroundingGap` name and semantics both land
- historical grounding gaps remain visible without forcing active late checkpoints

## Packet v0.4C: Historical Thrash Recovery And Proof

- [ ] Task: Move `DeadEndThrash` onto explicit repetition plus recovery inputs
  - Acceptance: `DeadEndThrash` reads repeated loop evidence from the repetition slice, computes
    recovery from the explicit recovery state, and clears the active flag after one clean
    verification interval while preserving historical evidence.
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task: Revalidate exported checkpoints against sentinel consumers
  - Acceptance: replay/live sentinel readers continue to consume exported checkpoints successfully
    after the analyzer semantic changes and renamed drift class.
  - Verify:
    - `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

- [ ] Task: Re-run the bounded sticky-session proof and refresh continuity docs
  - Acceptance: the known sticky late-session scenario no longer leaves late checkpoints active
    solely because of earlier prefix behavior, and any continuity docs that still describe the old
    cumulative-prefix semantics are updated.
  - Verify:
    - `cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR"`
    - doc review against the final analyzer behavior
  - Files:
    - `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-spec.md`
    - `docs/specs/agent-drift-analyzer-checkpoint-analysis-v0.4-plan.md`

Packet `v0.4C` exit condition:

- `DeadEndThrash` recovers explicitly
- sentinel consumers remain green
- the bounded sticky-session proof demonstrates honest late-session recovery
