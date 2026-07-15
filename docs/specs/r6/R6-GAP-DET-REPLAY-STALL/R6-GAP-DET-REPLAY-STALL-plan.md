# Plan: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / TASK `.1` ACCEPTANCE RECEIPT
`d788f45c9` REVIEW-CLEAN / TASK `.2` COMPLETE / TASK `.2A` ACCEPTED AND COMPLETE / TASK `.2B`
OPTION A ACCEPTED AND COMPLETE / TASK `.3` COMPLETE AT FOCUSED-CANDIDATE BOUNDARY / TASK `.3A`
COMMIT `7812dd5ef` REVIEW-CLEAN / TASK `.3B` COMPLETE / TASK `.3C` COMMIT `961a36574`
REVIEW-CLEAN / TASK `.3D` COMPLETE AT TWO-PHRASE CANDIDATE BOUNDARY / TASK `.3E` DOCS GATE
CURRENT / TASK `.3F` BLOCKED / TASK `.4` STOPPED AT ORDERED PROOF STEP 12** within
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
authorized source/test/helper/expected-disposition edits and focused TDD unit red/green are complete
at Task `.3`'s unstaged-candidate boundary. Task `.3A` commit `7812dd5ef` received fresh independent
built-in `default` `REVIEW CLEAN`; Task `.3B` implemented and focus-proved the verified-edit-epoch
refinement. Task `.4` reran ordered proof steps 1-11 green and stopped at the progress-acceptance
expected-fact red in step 12. Task `.3C` commit `961a36574` received fresh independent built-in
`default` `REVIEW CLEAN`. Task `.3D` applied exactly the two authorized phrase changes, then focused
proof exposed a later malformed-synthetic-identity red. Task `.3E` is the current docs-first
reconciliation gate; `.3F` is blocked until `.3E` is committed and fresh-review-clean, and Task `.4`
resumes from the beginning only after `.3F` completes.

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
the semantic gate and made Task `.3` current; Task `.3` is now complete at its focused-candidate
boundary. `CTX-R6-06`,
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

### 3E. Commit And Fresh-Review The Synthetic Fixture Identity Docs Gate — Current

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

Preserve the complete ten-file candidate byte-for-byte at patch SHA-256
`7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`. Stage and commit only this
SPEC/PLAN/TASKS gate, then obtain fresh built-in `default` review/fix cycles until `REVIEW CLEAN`.
No operator decision is required: these malformed synthetic output identities violate the already
selected truthful call-pairing contract rather than exposing a new product or semantic choice.

### 3F. Repair And Focus-Prove Synthetic Fixture Output Identities — Blocked On `.3E`

After `.3E` is committed and fresh-review-clean, edit exactly:

- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.archival.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.compact.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.archival.jsonl`; and
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.compact.jsonl`.

Add the seven matching output identities in both archival/compact copies, run a deterministic
identity audit over all `16` bundles/`32` row files, then run exact
`progress_acceptance_cases_match_expected_progress_contract`. Stop with the complete candidate
unstaged. Do not alter an expected posture or any surface outside the exact four-file authority.

### 4. Prove, Commit, And Fresh-Review — Stopped At Ordered Proof Step 12

Task `.4` completed proof steps 1-7 at `1 / 1` each, step 8 `troubleshooting` at `20 / 20` across
library and integration targets, and steps 9-11 at `1 / 1` each for exact `CTX-R6-02`, sticky
`CTX-R6-06`, and the frozen corpus. Step 12 first exposed the two stale phrases recorded by `.3C`, and
after `.3D` applied those phrases, focused proof exposed the later malformed synthetic identity red;
log `/tmp/r6-task3d-progress-acceptance.log`. After `.3F`, Task `.4` reruns the exact proof wall below
from the beginning, records actual results in TASKS and the
replay ledger, stages only intended files, runs staged GitNexus detect plus cached checks, commits
atomically, and enters fresh review/fix cycles.

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
No operator decision is required for `.3E`: the two malformed synthetic bundles carry identified
calls whose semantic outputs are id-less, directly violating the already-selected truthful
call-pairing contract. The narrow `.3F` repair changes only output identity metadata in four synthetic
row files and preserves both locked expected postures.
