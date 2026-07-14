# Plan: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / TASK `.1` ACCEPTANCE RECEIPT
`d788f45c9` REVIEW-CLEAN / TASK `.2` COMPLETE / TASK `.2A` DECISION REQUIRED / TASK `.3`
INCOMPLETE** within `R6-REPLAY`. Witness `60cde3dd7` is preserved red. Task `.0` series
`200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` and Task `.1` decision receipt `d788f45c9`
each received fresh independent built-in `default` `REVIEW CLEAN`. Task `.2` reconfirmed the exact
pre-edit red. The uncommitted Task `.3` candidate exposes secondary progress/recovery reds; Task `.2A`
decision `R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01` is current.

## Decisions

1. Preserve the trusted depth-1 built-in `default` rollout and its exact `CTX-R6-02` contract.
2. Treat the failure as analyzer-local output attribution, not compactor linkage, progress, or scoring.
3. Make call-ID-aware pairing authoritative whenever a command carries a call ID; retain legacy
   positional fallback only when it does not.
4. Limit production and unit-test work to `checkpoint/attempt.rs`.
5. Run focused proof before checkpoint/full-analyzer walls and keep the packet transition separate.

## Ordered Execution

### 0. Lock And Review The Packet

Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.0` is complete and review-clean. Historical boundary: the witness remained preserved and Rust was not authorized until the Task 1 operator decision, now accepted.

### 1. Refresh Impact And Obtain Operator Acceptance

Run upstream impact for `pair_output_rows` and context for `build_command_attempts`. Current evidence
is LOW for the edited helper (`1` direct caller, `17` indexed impacts, `0` processes, `1` module) and
HIGH for its caller/context (`16` direct indexed callers/tests). Because HIGH requires a warning,
issue `DECISION REQUIRED` ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` and stop the edit boundary until
the operator accepts the bounded one-file change and proof wall. Stop/recommend rejection if impact
becomes CRITICAL or the boundary widens.

**Complete:** refreshed evidence remained LOW for `pair_output_rows` (`1` direct caller, `17`
indexed impacts, `0` processes, `1` module) and HIGH for `build_command_attempts` caller context
(`16` direct indexed callers/tests, `0` processes, `1` module). The operator replied
`DECISION R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE: A` on 2026-07-14; receipt `d788f45c9` is fresh
independent `REVIEW CLEAN`. That decision authorized Tasks `.2`-`.4` only inside the locked
`attempt.rs` boundary. Task `.2` is complete, but the Task `.3` candidate proved that boundary
insufficient; Task `.2A` now gates further work. `CTX-R6-06`, `R6-CLOSE`, and R7/R8 remain blocked.

### 2. Reconfirm The Preserved Red — Complete

After the docs and decision gates, run exact `CTX-R6-02`. The expected pre-edit failure is the
event-`420` truthful-evidence assertion: checkpoint `5` and `Active / 30 / High` remain correct, but
evidence names successful sibling calls `421`/`475`. If the witness is unexpectedly green or fails
for a different contract, preserve output and stop for authority reconciliation.

Receipt: exact `CTX-R6-02` exited `101` at `acceptance_fixtures.rs:463`, the event-`420`
truthful-evidence assertion. Output is preserved at `/tmp/r6-replay-stall-pre-edit-red.log`.

### 2A. Obtain Post-Pairing Scope Acceptance — Current Gate

The uncommitted Task `.3` candidate makes the call-ID unit regression green and pairs target/sibling
outcomes truthfully, but exact `CTX-R6-02` now stops at line `455` with
`InsufficientEvidence != Stalled`; its broader candidate walls are red. Frozen `CTX-R6-06` also
changes from `Recovered` to `HistoricalOnly`. The prior accepted scope does not authorize progress,
recovery, or synthetic-helper changes. Issue decision
`R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01` and stop. Option A preserves both contracts and
authorizes a docs-first, fresh-review-clean same-packet amendment for the smallest call-ID cutover
scope beyond `attempt.rs`, including accepted HIGH `recovery_state` impact. Option B discards the
candidate and leaves the replay gates blocked. Do not create a nested or successor packet.

### 3. Implement The Locked Pairing Fix

In `crates/agent-drift-analyzer/src/checkpoint/attempt.rs` only:

- extract the command call ID from `dedupe_identity`;
- when present, scan concurrent rows through the applicable phase boundary and retain only matching
  call-ID outputs, with no positional fallback;
- when absent, preserve existing adjacent `ToolOutput`/`Error`/output-shaped `Unknown` behavior; and
- add `checkpoints_pair_concurrent_tool_outputs_by_call_id` covering interleaved calls and outputs.

Impact any additional existing symbol before editing it. Do not change compactor, progress, scoring,
fixture rows, or public contracts.

**Incomplete stop state:** the uncommitted candidate patch
`030d3d3e97640ba8fd4cf71f29f886b2273ec6e653aefa622aacaeb7057fdae7` is preserved at
`/tmp/r6-pairing-fix-secondary-red.patch`. Its unit regression passes `1 / 1`, and diagnostics pair
`420 -> 423` and `474 -> 477` as failed while `421 -> 425` and `475 -> 479` are clean. Exact
`CTX-R6-02` remains red at line `455`; analyzer-library, checkpoints-integration,
`dead_end_thrash`, and progress-corpus walls are respectively `144/13`, `109/24`, `15/3`, and
`2/1` pass/fail. Do not commit or widen this candidate before Task `.2A` is resolved and any Option-A
amendment is committed and fresh-review-clean.

### 4. Prove, Commit, And Fresh-Review — Blocked

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

The original HIGH-impact warning/acceptance and the post-pairing Task `.2A` decision are material
decision gates. Otherwise escalate only
for changed semantic authority, CRITICAL impact, an invalid preserved witness, required scope outside
`attempt.rs`, unisolatable unrelated work, or review proving the packet invalid. Routine red proof,
tests, commits, and review fixes remain autonomous after acceptance.
