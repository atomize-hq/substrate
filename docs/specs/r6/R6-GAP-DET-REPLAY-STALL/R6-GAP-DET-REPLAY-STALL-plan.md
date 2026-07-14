# Plan: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / DECISION REQUIRED** within
`R6-REPLAY`. Witness `60cde3dd7` is preserved red. Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default`
`REVIEW CLEAN`. The current gate is DECISION REQUIRED
`R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`; no production edit is authorized before operator acceptance.

## Decisions

1. Preserve the trusted depth-1 built-in `default` rollout and its exact `CTX-R6-02` contract.
2. Treat the failure as analyzer-local output attribution, not compactor linkage, progress, or scoring.
3. Make call-ID-aware pairing authoritative whenever a command carries a call ID; retain legacy
   positional fallback only when it does not.
4. Limit production and unit-test work to `checkpoint/attempt.rs`.
5. Run focused proof before checkpoint/full-analyzer walls and keep the packet transition separate.

## Ordered Execution

### 0. Lock And Review The Packet

Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.0` is complete and review-clean. Preserve the witness and do not edit Rust before the Task 1 operator decision.

### 1. Refresh Impact And Obtain Operator Acceptance

Run upstream impact for `pair_output_rows` and context for `build_command_attempts`. Current evidence
is LOW for the edited helper (`1` direct caller, `17` indexed impacts, `0` processes, `1` module) and
HIGH for its caller/context (`16` direct indexed callers/tests). Because HIGH requires a warning,
issue `DECISION REQUIRED` ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` and stop the edit boundary until
the operator accepts the bounded one-file change and proof wall. Stop/recommend rejection if impact
becomes CRITICAL or the boundary widens.

### 2. Reconfirm The Preserved Red

After the docs and decision gates, run exact `CTX-R6-02`. The expected pre-edit failure is the
event-`420` truthful-evidence assertion: checkpoint `5` and `Active / 30 / High` remain correct, but
evidence names successful sibling calls `421`/`475`. If the witness is unexpectedly green or fails
for a different contract, preserve output and stop for authority reconciliation.

### 3. Implement The Locked Pairing Fix

In `crates/agent-drift-analyzer/src/checkpoint/attempt.rs` only:

- extract the command call ID from `dedupe_identity`;
- when present, scan concurrent rows through the applicable phase boundary and retain only matching
  call-ID outputs, with no positional fallback;
- when absent, preserve existing adjacent `ToolOutput`/`Error`/output-shaped `Unknown` behavior; and
- add `checkpoints_pair_concurrent_tool_outputs_by_call_id` covering interleaved calls and outputs.

Impact any additional existing symbol before editing it. Do not change compactor, progress, scoring,
fixture rows, or public contracts.

### 4. Prove, Commit, And Fresh-Review

Run in order:

```bash
cargo test -p agent-drift-analyzer checkpoints_pair_concurrent_tool_outputs_by_call_id -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance progress_acceptance_cases_match_expected_progress_contract -- --nocapture
cargo test -p agent-drift-analyzer checkpoint -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-session-compactor --test normalization normalization_maps_rollout_events_into_provenance_preserving_rows -- --exact --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer --all-targets
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
git diff --check
```

The compactor command is optional confirmation only. Record exact results in TASKS and the replay
ledger, run the staged detect/diff gate, commit atomically, and dispatch a fresh built-in `default`
reviewer. Fix findings in new commits and repeat fresh review until clean.

### 5. Transition Back To Replay And Stop The Packet

Only after the implementation/proof series is review-clean, make a separate narrow authority
transition that marks `CTX-R6-02` complete, marks this active packet complete, clears the active
packet, and leaves `R6-REPLAY` active with `CTX-R6-06` then the family wall next. Commit and obtain
fresh independent review. Do not activate `R6-CLOSE` or start R7/R8.

## Escalation Boundary

The mandatory HIGH-impact warning/acceptance is a material decision gate. Otherwise escalate only
for changed semantic authority, CRITICAL impact, an invalid preserved witness, required scope outside
`attempt.rs`, unisolatable unrelated work, or review proving the packet invalid. Routine red proof,
tests, commits, and review fixes remain autonomous after acceptance.
