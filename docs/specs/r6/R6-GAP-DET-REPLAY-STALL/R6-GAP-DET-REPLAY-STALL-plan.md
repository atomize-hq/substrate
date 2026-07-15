# Plan: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / TASK `.1` ACCEPTANCE RECEIPT
`d788f45c9` REVIEW-CLEAN / TASK `.2` COMPLETE / TASK `.2A` ACCEPTED AND COMPLETE / TASK `.2B`
OPTION A ACCEPTED AND COMPLETE / TASK `.3` AUTHORIZED AND CURRENT / TASK `.4` BLOCKED** within
`R6-REPLAY`. Witness `60cde3dd7` is preserved
red. Task `.0` series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` and Task `.1` decision
receipt `d788f45c9` each received fresh independent built-in `default` `REVIEW CLEAN`. Task `.2`
reconfirmed the exact pre-edit red. On 2026-07-14 the operator accepted Task `.2A` with
`DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A`. That scope decision authorized this
docs-first amendment, not unconditional implementation. Diagnosis proves the requested frozen
`Recovered` result conflicts with canonical state semantics. The operator then replied exactly
`DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A`. Task `.2B` is complete and
reclassifies current sticky `CTX-R6-06` authority to `HistoricalOnly / 20`, unflagged; `Recovered / 20`
is historical baseline evidence only. This authority/expected-disposition change is approved, but its
authorized source/test/helper/expected-disposition edits, focused TDD unit red/green, and complete
unstaged candidate remain pending in current Task `.3`. Task `.4` owns the integrated/packet proof
wall, actual-result recording, staging and commit gates, atomic implementation/proof commit, and fresh
review/fix loop.

## Decisions

1. Preserve the trusted depth-1 built-in `default` rollout and its exact `CTX-R6-02` contract.
2. Treat the original failure as analyzer-local output attribution, not compactor linkage or scoring;
   truthful pairing separately exposes a progress-lane selection defect.
3. Make call-ID-aware pairing authoritative whenever a command carries a call ID; retain legacy
   positional fallback only when it does not.
4. Preserve the initial `checkpoint/attempt.rs` candidate; permit any wider amendment only after the
   current semantic decision and its fresh-review-clean docs gate.
5. Run focused proof before checkpoint/full-analyzer walls and keep the packet transition separate.
6. Preserve canonical score-state transitions: `Recovered` requires the immediately previous
   same-class score to be `Active`.

## Ordered Execution

### 0. Lock And Review The Packet

Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.0` is complete and review-clean. Historical boundary: the witness remained preserved and Rust was not authorized until the Task 1 operator decision, now accepted.

### 1. Refresh Impact And Obtain Operator Acceptance

Run upstream impact for `pair_output_rows` and context for `build_command_attempts`. Task `.1`
recorded evidence was LOW for the edited helper (`1` direct caller, `17` indexed impacts, `0`
processes, `1` module) and HIGH for its caller/context (`16` direct indexed callers/tests). The
current refreshed evidence remains LOW for `pair_output_rows` (`1` direct caller, `18` indexed
impacts, `0` processes, `1` module) and HIGH for `build_command_attempts` (`17` direct indexed
callers/tests, `0` processes, `1` module). Because HIGH requires a warning,
issue `DECISION REQUIRED` ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` and stop the edit boundary until
the operator accepts the bounded one-file change and proof wall. Stop/recommend rejection if impact
becomes CRITICAL or the boundary widens.

**Complete:** the Task `.1` recorded evidence remained LOW for `pair_output_rows` (`1` direct caller,
`17` indexed impacts, `0` processes, `1` module) and HIGH for `build_command_attempts` caller context
(`16` direct indexed callers/tests, `0` processes, `1` module). The operator replied
`DECISION R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE: A` on 2026-07-14; receipt `d788f45c9` is fresh
independent `REVIEW CLEAN`. That decision authorized Tasks `.2`-`.4` only inside the locked
`attempt.rs` boundary. Task `.2` is complete, but the Task `.3` candidate proved that boundary
insufficient; Task `.2A` accepted the docs-first amendment. Task `.2B` Option A has since resolved
the semantic gate and makes Task `.3` current. `CTX-R6-06`,
`R6-CLOSE`, and R7/R8 remain blocked.

### 2. Reconfirm The Preserved Red — Complete

After the docs and decision gates, run exact `CTX-R6-02`. The expected pre-edit failure is the
event-`420` truthful-evidence assertion: checkpoint `5` and `Active / 30 / High` remain correct, but
evidence names successful sibling calls `421`/`475`. If the witness is unexpectedly green or fails
for a different contract, preserve output and stop for authority reconciliation.

Receipt: exact `CTX-R6-02` exited `101` at `acceptance_fixtures.rs:463`, the event-`420`
truthful-evidence assertion. Output is preserved at `/tmp/r6-replay-stall-pre-edit-red.log`.

### 2A. Obtain Post-Pairing Scope Acceptance — Accepted And Complete

The uncommitted Task `.3` candidate makes the call-ID unit regression green and pairs target/sibling
outcomes truthfully, but exact `CTX-R6-02` now stops at line `455` with
`InsufficientEvidence != Stalled`; its broader candidate walls are red. Frozen `CTX-R6-06` also
changes from `Recovered` to `HistoricalOnly`. The operator replied
`DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A` on 2026-07-14, accepting a docs-first,
fresh-review-clean same-packet amendment beyond `attempt.rs`, including the recorded HIGH
`recovery_state` boundary. This completes `.2A` as a scope decision only. It does not authorize a
blind `recovery_state` edit or any implementation before the amendment reconciles its internal
semantic conflict. Do not create a nested or successor packet.

### 2B. Resolve The Recovered-Semantics Conflict — Option A Accepted And Complete

Clean `f898d61e7` passes exact
`acceptance_fixtures_representative_sticky_success_tail_stays_recovered` (`1 / 1`; log
`/tmp/r6-clean-head-sticky-exact.log`) because checkpoint `9` is `Regressing / Active 40` and
checkpoint `10` is `Recovered 20`. The candidate truthfully pairs checkpoint `9`'s concurrent clean
wall, making checkpoint `9` `Advancing / HistoricalOnly 20`, unflagged, and checkpoint `10`
`HistoricalOnly 20`, unflagged. Event `831 -> 837` is expected-negative human evidence, not the
cause; `recovery_state` does not read attempt outcomes. Canonical `Recovered` requires an immediately
previous same-class `Active` score, which no longer exists after truthful pairing.

The operator replied exactly
`DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A` on 2026-07-14. Selected Option A
reclassifies sticky `CTX-R6-06` to `HistoricalOnly / 20`, unflagged, preserves truthful pairing and
canonical immediately-prior-`Active` transition semantics, and authorizes the bounded Task `.3`
amendment. The old `Recovered / 20` expectation remains historical clean-baseline evidence only.
Option B and Option C are rejected. This completes `.2B` as an authority/expected-disposition
decision, not as source/test/fixture-expectation implementation or proof.

Verified impacts at `f898d61e7`: `assess_troubleshooting_progress` LOW (`4 / 12 / 0 / 2`);
`recovery_state` HIGH (`1 / 30 / 3 / 2`); `drift_state_for_score` LOW (`1 / 4 / 1 / 2`); and
`assign_drift_states` LOW (`2 / 4 / 1 / 2`). The accepted HIGH `recovery_state` boundary is not edit
authority because diagnosis proves it is the wrong seam. All recovery/state functions remain
forbidden unless a later explicit choice authorizes them.

### 3. Prepare The Selected Bounded Implementation — Authorized And Current

The original `attempt.rs` candidate remains preserved and uncommitted:

- extract the command call ID from `dedupe_identity`;
- when present, scan concurrent rows through the applicable phase boundary and retain only matching
  call-ID outputs, with no positional fallback;
- when absent, preserve existing adjacent `ToolOutput`/`Error`/output-shaped `Unknown` behavior; and
- add `checkpoints_pair_concurrent_tool_outputs_by_call_id` covering interleaved calls and outputs.

Under selected `.2B` Option A within fresh-review-clean amendment `d631e0c56` + `6498c343f`:

- keep the `attempt.rs` pairing candidate unchanged in intent;
- edit only `assess_troubleshooting_progress` plus new private lane helpers. Keep current attempts in
  event order; a current attempt is a lane tail exactly when no later current attempt satisfies the
  existing `attempts_are_comparable` with it. Evaluate each tail against prior-checkpoint plus
  earlier-current attempts directly comparable to that tail through the existing one-tail logic. Use
  shared comparability read-only, with no transitive clustering or comparability edit;
- treat every status except `InsufficientEvidence` as informative. Aggregate deterministically: zero
  informative lanes gives ordinary `InsufficientEvidence`; one is returned unchanged; any `Mixed`
  gives `Mixed`; `Advancing` plus `Stalled` or `Regressing` gives `Mixed`; otherwise any `Regressing`
  gives `Regressing`; otherwise any `Stalled` gives `Stalled`; otherwise return `Advancing`.
  `InsufficientEvidence` never overrides an informative lane;
- for multiple informative lanes, merge signals and counter-evidence in lane-tail event order and let
  existing finalization deduplicate/cap. Use the minimum informative-lane confidence, with `Mixed`
  additionally capped at `Medium`; preserve a single informative lane's confidence and evidence;
- repair malformed synthetic helpers by making general helpers ID-less and explicit identity helpers
  call/output-ID matched;
- add `troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane`
  (`Stalled + InsufficientEvidence => Stalled / Medium`, failure-lane evidence only),
  `troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling`
  (`Advancing + InsufficientEvidence => Advancing` unchanged),
  `troubleshooting_conflicting_informative_lanes_aggregate_mixed` (positive plus negative gives
  `Mixed`, both signal polarities, conservative confidence), and
  `troubleshooting_regressing_dominates_only_negative_lanes` (`Regressing + Stalled => Regressing`);
- rename the future sticky test to
  `acceptance_fixtures_representative_sticky_success_tail_is_historical_after_truthful_pairing` and
  update its function/assertion plus sticky `expected.json` to `HistoricalOnly / 20`, unflagged; never
  edit raw rollout rows. The frozen `dead_end_thrash` corpus test must change only that sticky
  assertion and keep its other three explicit postures unchanged.

Impact every additional existing symbol before editing it. Under Option A, do not edit
`recovery_state`, `drift_state_for_score`, `assign_drift_states`, shared comparability, scorer logic,
compactor logic, raw fixtures, public schemas, replay presentation, sentinel surfaces, R7, or R8.

**Incomplete stop state:** the uncommitted candidate patch
`030d3d3e97640ba8fd4cf71f29f886b2273ec6e653aefa622aacaeb7057fdae7` is preserved at
`/tmp/r6-pairing-fix-secondary-red.patch`. Its unit regression passes `1 / 1`, and diagnostics pair
`420 -> 423` and `474 -> 477` as failed while `421 -> 425` and `475 -> 479` are clean. Exact
`CTX-R6-02` remains red at line `455`; analyzer-library, checkpoints-integration,
`dead_end_thrash`, and progress-corpus walls are respectively `144/13`, `109/24`, `15/3`, and
`2/1` pass/fail. These are preserved pre-implementation candidate results, not green proof. Task `.3`
is now authorized to prepare exactly the selected implementation boundary. Use TDD red/green for the
focused pairing unit and four named troubleshooting units, and finish every authorized
source/test/helper/expected-disposition edit as one complete unstaged candidate diff ready for proof.
Task `.3` stops there: no integrated/packet proof wall, staging, staged GitNexus/cached-diff gate,
commit, or implementation review-clean claim belongs to it. The current candidate remains incomplete.

### 4. Prove, Commit, And Fresh-Review — Blocked Until Task `.3` Implementation/Focused Unit Criteria Complete

Task `.4` activates only after Task `.3` has produced the complete unstaged implementation candidate
and completed its focused TDD unit red/green criteria. Task `.4` then owns the exact integrated/packet
proof wall below, actual-result recording in packet TASKS and the replay ledger, staging only intended
files, staged GitNexus detect, cached diff check and inspection, the atomic implementation/proof
commit, and fresh built-in `default` review/fix cycles until clean.

Run in order:

```bash
cargo test -p agent-drift-analyzer checkpoints_pair_concurrent_tool_outputs_by_call_id -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_conflicting_informative_lanes_aggregate_mixed -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_regressing_dominates_only_negative_lanes -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_representative_sticky_success_tail_is_historical_after_truthful_pairing -- --exact --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture -- --exact --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance progress_acceptance_cases_match_expected_progress_contract -- --nocapture
cargo test -p agent-drift-analyzer checkpoint -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-session-compactor --test normalization normalization_maps_rollout_events_into_provenance_preserving_rows -- --exact --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer --all-targets
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
git diff --check
```

The compactor command is optional confirmation only. Record actual results in TASKS and the replay
ledger, stage only intended files, run staged GitNexus detect plus cached diff check and inspection,
commit the implementation/proof atomically, and dispatch a fresh built-in `default` reviewer. Fix
findings in new gated commits and repeat fresh review until clean.

### 5. Transition Back To Replay And Stop The Packet

Only after the implementation/proof series is review-clean, make a separate narrow authority
transition that marks `CTX-R6-02` complete, marks this active packet complete, clears the active
packet, and leaves `R6-REPLAY` active with `CTX-R6-06` then the family wall next. Commit and obtain
fresh independent review. Do not activate `R6-CLOSE` or start R7/R8.

## Escalation Boundary

The original HIGH-impact warning/acceptance, post-pairing Task `.2A` scope decision, and resolved Task
`.2B` Option-A semantic decision are material gates. Otherwise escalate only
for changed semantic authority, CRITICAL impact, an invalid preserved witness, required scope outside
the selected `.2B` boundary, unisolatable unrelated work, or review proving the packet invalid.
Routine red proof, tests, commits, and review fixes remain autonomous after acceptance.
