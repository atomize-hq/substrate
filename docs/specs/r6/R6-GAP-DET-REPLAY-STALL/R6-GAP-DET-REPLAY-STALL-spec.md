# R6-GAP-DET-REPLAY-STALL — Concurrent Replay Output Attribution

Status: **COMPLETE PACKET / TASK `.4` IMPLEMENTATION AND PROOF COMMIT `6eda87e60` FRESH
INDEPENDENT `REVIEW CLEAN` / TASK `.5` AUTHORITY TRANSITION COMMITTED BY THIS CHANGE, FRESH REVIEW
PENDING** within still-active `R6-REPLAY`; active packet is now `none`. Witness `60cde3dd7` remains
the historical behavioral-red receipt. The bounded implementation/proof commit `6eda87e60` closes
`CTX-R6-02`: exact selected control `1 / 1` preserves checkpoint `5`
`TroubleshootingFrontier / Stalled / Medium` and flagged `Active / 30 / High` while attributing only
failed targets `420 -> 423` and `474 -> 477`; clean siblings `421 -> 425` and `475 -> 479` are
excluded. The complete ordered packet wall is green, including seven focused controls, all `20`
troubleshooting matches, exact sticky and frozen-corpus controls, progress acceptance, checkpoint
matches, full analyzer `402 / 402`, compactor normalization, and static gates. A fresh independent
built-in `default` reviewer returned `REVIEW CLEAN` for `6eda87e60`. Current sticky `CTX-R6-06`
authority remains `HistoricalOnly / 20`, unflagged; `Recovered / 20` remains historical baseline
only. This transition does not run `CTX-R6-06`, the replay family wall, `R6-CLOSE`, or R7/R8. Fresh
review of the authority-only transition is required before phase-owned `CTX-R6-06` replay proof and
then the R6 family wall begin.

## Objective And Preserved Witness

Close the analyzer's concurrent tool-output attribution seam while preserving the trusted true-stall
contract, truthful per-call evidence, canonical per-lane progress semantics, canonical score-state
transitions, and raw replay rows. Compactor normalization, shared comparability, scorer logic, and
recovery/state functions remain outside this packet's current edit authority.

The selected annotated real rollout `019eb311-c7ce-7f50-ae13-b51a5b5461c3` is a depth-1 built-in
`default` subagent found by the expanded 2026-06-01 through 2026-07-13 screen. The screen covered all
`3,423` rollouts (`1,112` user, `2,298` subagent, `13` unspecified), including `1,611`
verifier-bearing sessions and `225` sessions with at least two failed verifier calls without requiring
identical commands. Of those, `161` were subagent candidates and `189` had non-identical failed
commands; `224` analyzed successfully, `0` were malformed, and this rollout was the sole exact match.
The known `019f3a4f-89c4-73a2-9bc0-2589b7430485` analysis was stopped after two minutes because its
existing result already adjudicated it as `Regressing` and later `Advancing`.

The fixture preserves these call-ID-linked facts:

- failed target verifier calls `420 -> 423` and `474 -> 477`, both exit `101` with the same failing
  assertion;
- concurrent sibling verifier calls `421 -> 425` and `475 -> 479`, both exit `0`;
- later clean target verifier `492 -> 495`, concurrent with clean sibling `493 -> 496`;
- checkpoint `5` is `TroubleshootingFrontier / Stalled / Medium`, exposes
  `FailureSignatureRepeated`, and produces flagged `dead_end_thrash Active / 30 / High`.

At committed baseline, the preserved test is red because the analyzer attributes repeated-failure
evidence to successful sibling calls `421` and `475` instead of truthful failed calls `420` and `474`.
The statement that the input contract, stall classification, and score disposition were otherwise
correct applies only to that committed pre-candidate baseline. It is not a forward claim: truthful
pairing exposes a separate progress-selection red described below.

## Owning Seam And Locked Fix

`pair_output_rows` in `crates/agent-drift-analyzer/src/checkpoint/attempt.rs` currently pairs by
position and stops at the next `ToolCall`. With concurrent calls, that leaves the first call without
its output and lets the second call consume both outputs. The compactor already preserves correct
`call_id` values in each call/output `dedupe_identity`; no compactor edit is authorized.

The fix is locked to this behavior:

1. when the command row has a `call_id`, scan the concurrent rows and pair only output rows carrying
   the same `call_id`;
2. do not use positional, error, or unknown-row fallback when a command `call_id` exists;
3. when no command `call_id` exists, retain the current legacy adjacency behavior, including
   `ToolOutput`, `Error`, and immediately adjacent output-shaped `Unknown` handling; and
4. add one interleaving unit test named `checkpoints_pair_concurrent_tool_outputs_by_call_id` that
   proves each concurrent call receives only its own outcome.

The initial production scope was only `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`. Task
`.2A` accepted a docs-first same-packet amendment after that boundary proved insufficient. Task `.2B`
then selected Option A: preserve truthful pairing and canonical immediately-prior-`Active` transition
semantics, and reclassify the sticky expected disposition to `HistoricalOnly / 20`, unflagged. Task
`.3` may now implement only the selected boundary below. Progress construction and the named
source/test/fixture-expectation surfaces are authorized only as specified there; recovery/state
functions, scoring, shared comparability, fixture rows, compactor normalization, public schemas,
replay presentation, R7, and R8 remain forbidden.

## Mandatory Pre-Edit Decision Gate

Task `.1` recorded GitNexus evidence reports `pair_output_rows` as LOW risk (`1` direct caller, `17`
indexed impacts, `0` affected processes, `1` module), while its `build_command_attempts`
caller/context is HIGH (`16` direct indexed callers/tests). Repository rules require warning before
proceeding when impact is HIGH.

After these docs are committed and fresh-review-clean, refresh both upstream impacts and issue a
structured `DECISION REQUIRED` report with ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`. That gate
was satisfied on 2026-07-14 when the operator explicitly selected Option A, accepting the bounded
one-file change and proof wall below. This acceptance does not authorize scope outside
`attempt.rs`; if refreshed impact becomes CRITICAL or changes the owning boundary, recommend
stopping instead.

## Acceptance And Exact Proof

Task `.3` used the first five focused unit commands below for TDD red/green and completed the full
unstaged implementation candidate. Task `.3B` completed the two verified-edit-epoch unit tests added
below. Task `.3D` completed the two-file expected-fact reconciliation described below. Task `.3F`
owns only the four-file synthetic output-identity repair and focused progress-acceptance proof. Task
`.4` reruns every focused unit and owns the exact integrated/packet proof sequence, result
recording, staging, staged GitNexus/cached-diff gates, atomic implementation/proof commit, and fresh
review/fix loop.

Under selected Task `.2B` Option A, the implementation is acceptable only when:

- the interleaving unit test proves call-ID-exclusive output pairing;
- exact `CTX-R6-02` is green with evidence on target events `420` and `474`, not siblings `421` and
  `475`;
- checkpoint `5` remains `TroubleshootingFrontier / Stalled / Medium` with
  `FailureSignatureRepeated` and no direct frontier-advance signals;
- its score remains flagged `Active / 30 / High`;
- the sticky `CTX-R6-06` expectation alone is reclassified to canonical
  `HistoricalOnly / 20`, unflagged, without changing any raw rollout row;
- the progress corpus, checkpoint family, and full analyzer remain green; and
- the clean counter-evidence remains target `492 -> 495` and sibling `493 -> 496`.

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

The compactor normalization command is a read-only optional cross-check of the already-correct
call-ID contract; no compactor change is allowed.

## Post-Pairing Stop State And Resolved Semantic Decision

Task `.2` reconfirmed the exact committed-baseline red with exit `101` at
`acceptance_fixtures.rs:463`, on the event-`420` truthful-evidence assertion. The preserved log is
`/tmp/r6-replay-stall-pre-edit-red.log`.

The Task `.3` candidate remains uncommitted and unproven. Its patch SHA-256 is
`030d3d3e97640ba8fd4cf71f29f886b2273ec6e653aefa622aacaeb7057fdae7`, with a preservation copy at
`/tmp/r6-pairing-fix-secondary-red.patch`. It establishes truthful diagnostic pairing:

- `420 -> 423` is `Failed`, exit `101`; `421 -> 425` is `Clean`, exit `0`;
- `474 -> 477` is `Failed`, exit `101`; `475 -> 479` is `Clean`, exit `0`; and
- `checkpoints_pair_concurrent_tool_outputs_by_call_id` passes `1 / 1`.

That is not sufficient acceptance. Exact `CTX-R6-02` exits `101` at
`acceptance_fixtures.rs:455`, where actual progress is `InsufficientEvidence` instead of the locked
`Stalled`; the log is `/tmp/r6-ctx-r6-02-green.log`. The test stops there before evidence and score
assertions. Candidate walls are red: analyzer library `144` pass / `13` fail; checkpoints integration
`109` pass / `24` fail; `dead_end_thrash` `15` pass / `3` fail; and progress corpus `2` pass / `1`
fail. Synthetic helpers also carry unmatched call IDs.

Diagnostics show `assess_troubleshooting_progress` selects clean sibling `475`, while comparability
excludes the failed target lane, producing `InsufficientEvidence`. Its GitNexus upstream impact at
`f898d61e7` is LOW (`4` direct, `12` total, `0` processes, `2` modules). Shared comparability helpers
must not be changed under the current authority.

Task `.3` subsequently completed the selected lane implementation and its five focused units. Task
`.3A` commit `7812dd5ef` then locked the verified-edit-epoch refinement and received fresh independent
built-in `default` `REVIEW CLEAN`; Task `.3B` implemented it and made both named focused tests green.
The original eight-file unstaged candidate, including Task `.4`'s bounded preservation of an existing
single-lane result, remains at patch SHA-256
`70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`. Task `.3C` commit
`961a36574` received fresh independent built-in `default` `REVIEW CLEAN`; Task `.3D` then applied
only its two authorized expected-fact phrases. The resulting complete ten-file unstaged candidate has
patch SHA-256 `7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`.

Task `.4` ordered proof steps 1-7 each pass `1 / 1`; step 8 passes the `troubleshooting` filter at
`20 / 20` across library and integration targets; and exact `CTX-R6-02`, sticky `CTX-R6-06`, and the
frozen corpus each pass `1 / 1` in steps 9-11. Step 12
`progress_acceptance_cases_match_expected_progress_contract` initially exited `101` because case
`real-reopen-regressing-019e894a-ord7` still requires stale wording. The preserved log is
`/tmp/r6-task4b-12-progress-acceptance.log`. No files were staged or committed.

The case remains `TroubleshootingFrontier / Regressing / High` with
`PreviouslyCleanScopeBroken`; status, dimension, confidence, signals, evidence minima, and raw rows
are unchanged. Truthful call-ID pairing exposes the stable facts as clean event `109 -> 116`, exit
`0`, followed by failing event `636 -> 638`, exit `101`. Only the stale phrases
`earlier later stage verification attempt` and `later regressing verification attempt` must become
`earlier clean verification attempt` and `later failing verification attempt`, respectively.

After `.3D` made exactly those two changes, focused progress acceptance again exited `101`; log
`/tmp/r6-task3d-progress-acceptance.log`. The later reported case
`synthetic-implementation-advancing` is `InsufficientEvidence`, not locked `Advancing`. Its verifier
calls event `1` (`call-1`) and event `5` (`call-5`) have semantic outputs at events `2` and `6`, but
the outputs are id-less and correctly do not pair under call-ID-exclusive semantics. The
`apply_patch` call event `4` (`call-4`) has no output and needs no change.

A deterministic audit of all `16` progress-acceptance bundles and their `32` archival/compact row
files found the complete malformed synthetic set. In addition to the two implementation outputs,
`synthetic-zero-verifier-anti-flap` has id-less semantic outputs at event `2` for `call-1`, event `4`
for `call-3`, event `7` for `call-6`, event `9` for `call-8`, and event `11` for `call-10`. Each of
the seven semantic rows appears in both archival and compact form, for `14` physical repairs.

Task `.2A` asked the operator to preserve both the `CTX-R6-02` Stalled/Active contract and the then-frozen
`CTX-R6-06` Recovered contract while accepting a docs-first amendment beyond `attempt.rs`, including
the recorded HIGH `recovery_state` boundary. The operator replied
`DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A` on 2026-07-14. That completes Task `.2A` as a
scope decision, not as unconditional implementation authority. Amendment diagnosis then proved an
internal conflict in the accepted option:

- clean `f898d61e7` passes exact
  `acceptance_fixtures_representative_sticky_success_tail_stays_recovered` (`1 / 1`; log
  `/tmp/r6-clean-head-sticky-exact.log`) because checkpoint `9` is
  `TroubleshootingFrontier / Regressing` with flagged `Active / 40`, followed immediately by
  checkpoint `10` at `Recovered / 20`;
- the pairing candidate truthfully associates checkpoint `9`'s concurrent clean verifier wall, so
  checkpoint `9` becomes `TroubleshootingFrontier / Advancing`, unflagged
  `HistoricalOnly / 20`, and checkpoint `10` remains unflagged `HistoricalOnly / 20`;
- event `831 -> 837`, exit `1`, is expected-negative human evidence, but it is not causal:
  `recovery_state` does not read attempt outcomes; and
- canonical `Recovered` requires the immediately previous same-class score to be `Active`. Once
  truthful pairing removes that prior `Active`, `HistoricalOnly` is the canonical transition. Holding
  `Recovered` would require redefining the state contract rather than fixing output attribution.

Current verified GitNexus upstream impact at `f898d61e7` is: `assess_troubleshooting_progress` LOW
(`4` direct / `12` impacted / `0` processes / `2` modules); `recovery_state` HIGH (`1 / 30 / 3 / 2`);
`drift_state_for_score` LOW (`1 / 4 / 1 / 2`); and `assign_drift_states` LOW (`2 / 4 / 1 / 2`). The
accepted HIGH `recovery_state` boundary is not edit authority because diagnosis proves it is the wrong
seam. `recovery_state`, `drift_state_for_score`, `assign_drift_states`, and every other recovery/state
function remain forbidden unless a later explicit operator choice authorizes them.

Do not create a nested or successor packet. The following is the resolved historical `.2B` decision
report; it is retained as the durable rationale for selected Option A:

```text
DECISION RESOLVED
ID: R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02
PHASE/PACKET: R6-REPLAY / R6-GAP-DET-REPLAY-STALL
QUESTION: Resolve the conflict between truthful output pairing plus canonical score-state transitions and the previously frozen CTX-R6-06 Recovered expectation.
REPO EVIDENCE: Clean f898d61e7 passes the sticky exact control because checkpoint 9 is Regressing/Active40 and checkpoint 10 is Recovered20. Truthful pairing makes checkpoint 9 Advancing/HistoricalOnly20 and checkpoint 10 HistoricalOnly20. Event 831->837 is expected-negative human evidence but is not causal, and recovery_state does not consume attempt outcomes. Canonical Recovered requires an immediately previous same-class Active score.
WHY AUTHORITY CANNOT DECIDE: Task .2A accepted preserving both contracts, but live diagnosis proves that preserving frozen Recovered would contradict canonical state semantics. Choosing which authority changes is an operator-owned semantic decision.
OPTIONS:
A. Reclassify sticky CTX-R6-06 to HistoricalOnly/20/unflagged, preserve truthful pairing and canonical transition semantics, and authorize the conditional bounded amendment below.
B. Discard the pairing candidate and retain the frozen Recovered expectation, leaving CTX-R6-02 unresolved and all downstream replay gates blocked.
C. Explicitly redefine Recovered beyond immediately previous same-class Active and authorize a broad cross-family contract/test review.
RECOMMENDATION: A. It preserves truthful per-call evidence and the existing canonical score-state contract; only the stale sticky expected disposition changes. C is not recommended because it broadens a local attribution/progress repair into a cross-family state-contract change.
SAFE WORK ALREADY COMPLETED: Task .0 and Task .1 are review-clean; Task .2 and the accepted Task .2A scope decision are complete; the candidate, clean-baseline proof, and every secondary red are preserved but uncommitted.
RESOLUTION: A. Reclassify sticky CTX-R6-06 to HistoricalOnly / 20, unflagged; preserve truthful pairing and canonical immediately-prior-Active transition semantics; authorize the bounded Task .3 amendment.
REPLY RECEIVED: DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A
STILL BLOCKED: Task .4 until Task .3 implementation/focused unit criteria complete; CTX-R6-06 replay proof, family wall, packet transition, R6-CLOSE, and R7/R8.
```

Task `.2B` Option A selects this implementation boundary for current Task `.3`:

1. preserve the current `attempt.rs` pairing candidate unchanged in intent;
2. edit only `assess_troubleshooting_progress` plus new private lane helpers, using the existing
   `attempts_are_comparable` read-only and without transitive clustering or a shared-comparability
   change. Keep current-checkpoint attempts in event order. A current attempt is a lane tail exactly
   when no later current attempt satisfies existing `attempts_are_comparable` with it. Evaluate each
   tail against the prior-checkpoint attempts plus earlier-current attempts that are directly
   comparable to that tail, using the existing one-tail troubleshooting logic;
3. repair malformed synthetic helpers by keeping general-purpose rows ID-less and giving explicit
   identity helpers matching call/output IDs;
4. aggregate only informative lane results, where every status other than `InsufficientEvidence` is
   informative. Zero informative lanes return ordinary `InsufficientEvidence`; one returns unchanged;
   any `Mixed` returns `Mixed`; `Advancing` plus either `Stalled` or `Regressing` returns `Mixed`;
   otherwise any `Regressing` returns `Regressing`; otherwise any `Stalled` returns `Stalled`; and the
   remaining informative case returns `Advancing`. An `InsufficientEvidence` lane never overrides an
   informative lane;
5. for multiple informative lanes, merge signals and counter-evidence in lane-tail event order and let
   existing finalization deduplicate and cap them. Choose the minimum confidence among informative
   lanes and additionally cap `Mixed` at `Medium`. A single informative lane preserves its confidence
   and evidence unchanged;
6. add focused tests with these exact names and contracts:
   `troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane`
   (`Stalled + InsufficientEvidence => Stalled / Medium`, with failure-lane evidence only),
   `troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling`
   (`Advancing + InsufficientEvidence => Advancing` unchanged),
   `troubleshooting_conflicting_informative_lanes_aggregate_mixed` (positive plus negative informative
   lanes produce `Mixed`, both signal polarities, and conservative confidence), and
   `troubleshooting_regressing_dominates_only_negative_lanes` (`Regressing + Stalled => Regressing`);
7. rename the future sticky test to
   `acceptance_fixtures_representative_sticky_success_tail_is_historical_after_truthful_pairing`,
   update that test function and assertion plus its sticky `expected.json` to
   `HistoricalOnly / 20`, unflagged, and never alter raw rollout rows. Exact
   `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture` must update the
   sticky assertion to `HistoricalOnly / 20`, unflagged while its other three explicit postures remain
   unchanged; and
8. leave all authorized edits as one complete unstaged candidate after the focused pairing/progress
   unit red/green; Task `.4` then runs focused `CTX-R6-02`/sticky proof before the checkpoint,
   progress-corpus, `dead_end_thrash`, full-analyzer, compactor normalization, and static walls.

Task `.3A` locked this same-packet refinement without changing any preceding lane-tail, direct
comparability, aggregation-table, forbidden-seam, Task `.2B`, or sticky authority:

1. compute every current-checkpoint lane-tail result and identify the newest informative tail;
2. define the active terminal verified-edit epoch as beginning after the latest source/test edit that
   precedes that newest informative tail;
3. aggregate only lane-tail results after that cutover. Attempts before the cutover remain eligible as
   directly comparable history for the retained tails;
4. do not let a later unverified edit erase the last proven frontier; and
5. prove the refinement with exact tests
   `troubleshooting_latest_verified_edit_epoch_supersedes_earlier_informative_lanes` and
   `troubleshooting_unverified_trailing_edit_does_not_erase_latest_informative_lane`.

For checkpoint `5`, tails `392`/`393` are `Advancing`, tail `474` is `Stalled`, and tail `475` is
`InsufficientEvidence`; the edit cutover is `463`. Only `474` and `475` participate in terminal-epoch
aggregation, so the result is `Stalled`. Event `486` is inside checkpoint `5` (boundary `489`), but
its would-be verifying calls `492`/`493` and outputs `495`/`496` are outside checkpoint `5`; as a
trailing unverified edit within checkpoint `5`, it does not erase that result.

No operator decision was required: `CTX-R6-02` remained locked at `Stalled / Active`, and Task `.2B`
changed only sticky `CTX-R6-06`. Commit `7812dd5ef` reconciled the unexpected red inside those
authorities and received fresh independent built-in `default` `REVIEW CLEAN`. Task `.3B` then
implemented the refinement and completed both focused red/green tests without widening the authorized
seam.

Task `.3C` recorded the ordered step-12 progress-acceptance red and locked the narrow expected-fact
reconciliation in exactly this packet's SPEC/PLAN/TASKS; canonical mirrors remained untouched.
Commit `961a36574` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.3D` then
edited exactly:

- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`; and
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/real-reopen-regressing-019e894a-ord7/expected.json`.

Within those two files, it replaced only the stale fact wording with
`earlier clean verification attempt` and `later failing verification attempt`. It then preserved the
later synthetic identity red without widening scope.

Task `.3E` landed its exact packet-doc gate at `4615d9e3c` and received fresh independent built-in
`default` `REVIEW CLEAN`. Task `.3F` then edited exactly:

- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.archival.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-implementation-advancing/rows.compact.jsonl`;
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.archival.jsonl`; and
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/synthetic-zero-verifier-anti-flap/rows.compact.jsonl`.

In both archival/compact copies, add only matching `dedupe_identity` values with
`type: function_call_output` to implementation outputs `2` (`call-1`) and `6` (`call-5`), and
zero-verifier outputs `2` (`call-1`), `4` (`call-3`), `7` (`call-6`), `9` (`call-8`), and `11`
(`call-10`). Do not change the output-less implementation `apply_patch` call `4`, another row,
fixture, expected contract, source, test, packet/canonical mirror, native/adapted/real fixture, or
any other surface. Task `.3F`'s deterministic audit passed all `16` bundles / `32` row files /
`8,388` rows with seven logical and `14` physical locked identities, and exact
`progress_acceptance_cases_match_expected_progress_contract` passed `1 / 1`.

The implementation fixture must remain
`AutonomousImplementation / ImplementationVerificationWall / Advancing / Medium` with
`FailureFrontierAdvanced` and `WorkingSetConcentrated`. The zero-verifier fixture must remain
`PlanningConvergence / InsufficientEvidence / Low`. No operator decision is required: malformed
synthetic output identities violate already-selected truthful call pairing, so this is a bounded
fixture-contract repair, not a new product or semantic choice.

Under Option A, do not edit `recovery_state`, `drift_state_for_score`, `assign_drift_states`,
shared comparability, scorer logic, compactor logic, raw fixtures outside Task `.3F`'s exact four-file
synthetic identity exception, sentinel surfaces, R7, or R8.

## Commit, Review, And Exit

Tasks `.3` through `.3F` are complete at their recorded boundaries. Task `.4` reran the ordered proof
from the beginning and committed the implementation/proof atomically at `6eda87e60`; seven focused
controls, all `20` troubleshooting matches, exact `CTX-R6-02`, exact sticky, frozen corpus, progress
acceptance, checkpoint matches, full analyzer `402 / 402`, compactor normalization, and static gates
are green. Staged detect reported medium risk across `16` files / `54` symbols / three affected
troubleshooting-tail flows with no HIGH or CRITICAL result, and cached gates were clean. A fresh
independent built-in `default` reviewer returned `REVIEW CLEAN` for `6eda87e60`.

This authority-only transition completes `CTX-R6-02` and the packet, clears the active packet to
`none`, and returns control to still-active `R6-REPLAY`. Fresh independent review of this transition
is required before phase-owned `CTX-R6-06` replay proof and then the R6 family wall begin. It does
not activate `R6-CLOSE` or start R7/R8.
