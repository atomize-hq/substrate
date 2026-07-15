# Plan: R6-GAP-DET-REPLAY-STALL

Status: **COMPLETE PACKET / TASK `.4` IMPLEMENTATION AND PROOF COMMIT `6eda87e60` FRESH
INDEPENDENT `REVIEW CLEAN` / TASK `.5` AUTHORITY TRANSITION SERIES `1ff592823` + `7839a7f47` FRESH
INDEPENDENT `REVIEW CLEAN`**; `R6-REPLAY` is now complete and active packet remains `none`. The selected Option-A
pairing/progress implementation and expected-disposition changes landed atomically at `6eda87e60`.
Exact `CTX-R6-02` is green at its locked `Stalled / Active` contract, full analyzer proof is
`402 / 402`, and every ordered packet proof/static gate is green. A fresh independent built-in
`default` reviewer returned `REVIEW CLEAN` for that implementation/proof commit. Current sticky
`CTX-R6-06` authority remains `HistoricalOnly / 20`, unflagged; clean-baseline `Recovered / 20`
remains historical only. Authority-only transition series `1ff592823` + `7839a7f47` clears the
packet without activating `R6-CLOSE` and is fresh independent built-in `default` `REVIEW CLEAN`.
Later phase-owned `CTX-R6-06` replay and the R6 family wall are green; proof/fix series
`b1791c1e3` + `e6d43eee9` + `61c9d5074` is fresh independent built-in `default` `REVIEW CLEAN`.
The later narrow phase transition activates `R6-CLOSE` at entry only with `CTX-R6-17` next.

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
7. Aggregate the terminal verified-edit epoch only: compute all lane tails and the newest informative
   tail, cut over after the latest preceding source/test edit, retain earlier attempts as directly
   comparable history, and do not let later unverified edits erase the last proven frontier.

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
insufficient; Task `.2A` accepted the docs-first amendment. Task `.2B` Option A then resolved
the semantic gate, and Tasks `.3`-`.5` are complete at their recorded review-clean boundaries.
Later phase-owned `CTX-R6-06` and the family wall are review-clean, `R6-REPLAY` is complete, and
`R6-CLOSE` is active at entry only. R7/R8 remain blocked.

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

### 3. Prepare The Selected Bounded Implementation — Complete At Focused-Candidate Boundary

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
compactor logic, raw fixtures outside Task `.3F`'s exact four-file synthetic identity exception,
public schemas, replay presentation, sentinel surfaces, R7, or R8.

**Complete stop state:** Task `.3` finished the authorized eight-file unstaged candidate and all five
focused units pass `1 / 1`. Task `.4` made a bounded private-helper fix preserving an existing
single-lane result; Task `.3B` then implemented and focus-proved the review-clean verified-edit-epoch
authority. The current preserved candidate patch SHA-256 is
`70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2` for those original eight
files. After `.3D`'s exact two-file wording repair, the complete ten-file unstaged candidate is
`7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`. No implementation files
were staged or committed, and no implementation review-clean claim exists.

### 3A. Commit And Fresh-Review The Verified-Edit-Epoch Docs Gate — Complete / Review-Clean

Record the unexpected Task `.4` witness and lock the terminal verified-edit-epoch refinement in
exactly this SPEC/PLAN/TASKS. Do not edit canonical mirrors and preserve the eight-file candidate
byte-for-byte. Run `git diff --check`, stage and commit only these three packet docs through the
required staged gate, and obtain fresh built-in `default` review/fix cycles until clean.

Checkpoint `5` has lane tails `392`/`393` `Advancing`, `474` `Stalled`, and `475`
`InsufficientEvidence`. Event `463` is the latest source/test edit before newest informative tail
`474`. Event `486` is inside checkpoint `5` (boundary `489`), but its would-be verifying calls
`492`/`493` and outputs `495`/`496` are outside checkpoint `5`; within that checkpoint, event `486` is
therefore a trailing unverified edit. The active terminal verified-edit epoch begins after `463`;
aggregate only tails after that cutover while retaining earlier attempts as directly comparable
history. Thus `474` plus `475` produces `Stalled`, and event `486` does not erase the last proven
frontier.

Commit `7812dd5ef` landed exactly this SPEC/PLAN/TASKS authority and received fresh independent
built-in `default` `REVIEW CLEAN`.

### 3B. Implement And Focus-Prove The Verified-Edit-Epoch Refinement — Complete

After `.3A` became committed and fresh-review-clean, Task `.3B` implemented only the locked
refinement within the already authorized `assess_troubleshooting_progress`/private-helper boundary.
It preserved the exact lane-tail definition, direct comparability, aggregation table, forbidden seams,
Task `.2B` Option A, and sticky authority. Red/green for exact tests
`troubleshooting_latest_verified_edit_epoch_supersedes_earlier_informative_lanes` and
`troubleshooting_unverified_trailing_edit_does_not_erase_latest_informative_lane` is complete, and the
candidate remains unstaged.

### 3C. Commit And Fresh-Review The Progress Expected-Fact Docs Gate — Review-Clean

Record Task `.4`'s ordered step-12 progress-acceptance red in exactly this SPEC/PLAN/TASKS. Case
`real-reopen-regressing-019e894a-ord7` remains
`TroubleshootingFrontier / Regressing / High` with `PreviouslyCleanScopeBroken`; raw rows, dimension,
status, confidence, signals, and evidence minima are unchanged. Truthful call-ID pairing exposes clean
event `109 -> 116`, exit `0`, followed by failing event `636 -> 638`, exit `101`. Only the stale facts
`earlier later stage verification attempt` and `later regressing verification attempt` must be
reconciled to `earlier clean verification attempt` and `later failing verification attempt`.

Commit `961a36574` landed exactly this SPEC/PLAN/TASKS authority and received fresh independent
built-in `default` `REVIEW CLEAN`. Canonical mirrors remained untouched, and the original eight-file
candidate remained byte-for-byte at patch SHA-256
`70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`.

### 3D. Reconcile And Focus-Prove The Two Progress Expected Facts — Complete At Exact-Edit Boundary

After `.3C` became committed and fresh-review-clean, Task `.3D` edited exactly
`crates/agent-drift-analyzer/tests/progress_acceptance.rs` and
`crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/real-reopen-regressing-019e894a-ord7/expected.json`.
It replaced only the two stale fact phrases with their truthful clean/failing wording. Focused
`progress_acceptance_cases_match_expected_progress_contract` then exited `101`; preserved log
`/tmp/r6-task3d-progress-acceptance.log`. The next reported case,
`synthetic-implementation-advancing`, is now `InsufficientEvidence`, not the locked `Advancing`.
Task `.3D` stopped with the complete ten-file candidate unstaged. Its patch SHA-256 is
`7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`; the original eight-file
portion remains at `70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`.

### 3E. Commit And Fresh-Review The Synthetic Fixture Identity Docs Gate — Complete / Review-Clean

The call-ID-exclusive pairing selected by this packet correctly refuses to associate an identified
command with an id-less output. In `synthetic-implementation-advancing`, verifier calls event `1`
(`call-1`) and event `5` (`call-5`) have semantic outputs at events `2` and `6`, but those outputs
lack `function_call_output` identities. The `apply_patch` command at event `4` (`call-4`) has no
output and requires no change. A deterministic audit of all `16` progress-acceptance bundles and
their `32` archival/compact row files finds one other synthetic malformed set:
`synthetic-zero-verifier-anti-flap` outputs at events `2`, `4`, `7`, `9`, and `11` lack the matching
identities for calls `call-1`, `call-3`, `call-6`, `call-8`, and `call-10`.

Lock the repair to the four archival/compact files for those two synthetic bundles. Add only the
matching `dedupe_identity` with `type: function_call_output` to those seven semantic outputs in both
copies (`14` physical rows). Do not edit another row, fixture, expected contract, source, test,
packet/canonical mirror, native/adapted/real fixture, or a call without an output. Preserve
`synthetic-implementation-advancing` at
`AutonomousImplementation / ImplementationVerificationWall / Advancing / Medium` with
`FailureFrontierAdvanced` and `WorkingSetConcentrated`; preserve `synthetic-zero-verifier-anti-flap`
at `PlanningConvergence / InsufficientEvidence / Low`.

Commit `4615d9e3c` landed this SPEC/PLAN/TASKS gate and received fresh independent built-in `default`
`REVIEW CLEAN`. No operator decision was required because the malformed synthetic identities
violated the selected truthful call-pairing contract.

### 3F. Repair And Focus-Prove Synthetic Fixture Output Identities — Complete

After `.3E` is committed and fresh-review-clean, edit exactly:

- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.archival.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.compact.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.archival.jsonl`; and
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.compact.jsonl`.

The seven matching output identities were added in both archival/compact copies. The deterministic
audit passed all `16` bundles / `32` row files / `8,388` rows with seven logical and `14` physical
locked identities, and exact progress acceptance passed `1 / 1`. No expected posture or surface
outside the exact four-file authority changed.

### 4. Prove, Commit, And Fresh-Review — Complete / Review-Clean

Task `.4` reran the exact proof wall below from the beginning and committed the complete
implementation/proof candidate at `6eda87e60`. Seven focused controls passed `1 / 1`,
`troubleshooting` passed `20 / 20`, all four exact acceptance/progress controls passed `1 / 1`,
checkpoint matches passed `62` library + `131` integration plus matching targets, full analyzer
passed `402 / 402`, compactor normalization passed `1 / 1`, and all static gates were green. Staged
detect reported medium risk across exactly `16` files / `54` symbols / three affected flows, with no
HIGH or CRITICAL result; cached gates were clean. A fresh independent built-in `default` reviewer
returned `REVIEW CLEAN`.

Run in order:

```bash
cargo test -p agent-drift-analyzer checkpoints_pair_concurrent_tool_outputs_by_call_id -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_conflicting_informative_lanes_aggregate_mixed -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_regressing_dominates_only_negative_lanes -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_latest_verified_edit_epoch_supersedes_earlier_informative_lanes -- --nocapture
cargo test -p agent-drift-analyzer troubleshooting_unverified_trailing_edit_does_not_erase_latest_informative_lane -- --nocapture
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

The compactor command was optional confirmation only and passed. The actual results are recorded in
TASKS and the replay ledger.

### 5. Transition Back To Replay And Stop The Packet — Review-Clean

Commit `6eda87e60` is fresh independent `REVIEW CLEAN`; authority-only transition series
`1ff592823` + `7839a7f47` marks `CTX-R6-02` and the packet complete, clears the active packet to
`none`, keeps `R6-REPLAY` active, and is fresh independent built-in `default` `REVIEW CLEAN`.
Phase-owned `CTX-R6-06` replay and the family wall later passed; their separate proof/fix series
`b1791c1e3` + `e6d43eee9` + `61c9d5074` received fresh independent built-in `default` `REVIEW
CLEAN`. The later narrow phase transition activates `R6-CLOSE` at entry only with `CTX-R6-17` next;
this packet itself did not activate `R6-CLOSE` or start R7/R8.

## Escalation Boundary

The original HIGH-impact warning/acceptance, post-pairing Task `.2A` scope decision, and resolved Task
`.2B` Option-A semantic decision are material gates. Otherwise escalate only
for changed semantic authority, CRITICAL impact, an invalid preserved witness, required scope outside
the selected `.2B` boundary, unisolatable unrelated work, or review proving the packet invalid.
Routine red proof, tests, commits, and review fixes remain autonomous after acceptance.
No operator decision is required for `.3E`: the two malformed synthetic bundles carry identified
calls whose semantic outputs are id-less, directly violating the already-selected truthful
call-pairing contract. The narrow `.3F` repair changes only output identity metadata in four synthetic
row files and preserves both locked expected postures.
