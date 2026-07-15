# Tasks: R6-GAP-DET-REPLAY-STALL

Status: **COMPLETE PACKET / TASK `.4` IMPLEMENTATION AND PROOF COMMIT `6eda87e60` FRESH
INDEPENDENT `REVIEW CLEAN` / TASK `.5` AUTHORITY TRANSITION SERIES `1ff592823` + `7839a7f47` FRESH
INDEPENDENT `REVIEW CLEAN`**; `R6-REPLAY` is now complete and active packet remains `none`. Historical witness
`60cde3dd7` is preserved. Commit `6eda87e60` makes exact `CTX-R6-02` green with truthful call-ID
pairing, terminal verified-edit-epoch lane semantics, and the locked `Stalled / Active` disposition.
Its complete ordered proof passes seven focused controls, all `20` troubleshooting matches, exact
`CTX-R6-02`, exact sticky `CTX-R6-06`, frozen corpus, progress acceptance, checkpoint matches, full
analyzer `402 / 402`, compactor normalization, and static gates. A fresh independent built-in
`default` reviewer returned `REVIEW CLEAN`. Current sticky authority remains `HistoricalOnly / 20`,
unflagged; old `Recovered / 20` remains historical baseline only. This transition assigns no terminal
scorer disposition and does not run `CTX-R6-06`, the family wall, `R6-CLOSE`, or R7/R8.

## Required Commit Gate

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Every commit receives a fresh built-in `default` review. Findings are fixed in a new bounded commit
and sent to another fresh reviewer until `REVIEW CLEAN`.

## Task Ledger

- [x] **R6-GAP-DET-REPLAY-STALL.0 — Commit and independently review the packet docs gate.**
  - Files: exactly this SPEC/PLAN/TASKS plus the already-authorized `expected.json` correction from
    clean target `493 -> 495` to truthful target `492 -> 495`, retaining sibling `493 -> 496`.
  - Verify: JSON parse; local Markdown links/status wording; required staged gate; `git diff --check`.
  - Review: fresh built-in `default`; bounded docs/annotation fix commits; fresh reviewer until clean.
  - Boundary: do not run the witness or edit Rust until this task is committed and review-clean.
  - Current receipt: full series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.0` is complete.

- [x] **R6-GAP-DET-REPLAY-STALL.1 — Refresh impact and obtain the required operator decision.**
  - Prerequisite: Task 0 review-clean.
  - Refresh upstream impact for `pair_output_rows`; inspect `build_command_attempts` as its caller and
    packet-wide context.
  - Task `.1` recorded evidence: `pair_output_rows` LOW (`1` direct caller, `17` indexed impacts, `0`
    processes, `1` module); `build_command_attempts` context HIGH (`16` direct indexed callers/tests).
  - Current refreshed evidence: `pair_output_rows` LOW (`1` direct caller, `18` indexed impacts, `0`
    processes, `1` module); `build_command_attempts` context HIGH (`17` direct indexed callers/tests,
    `0` processes, `1` module).
  - Required output: structured `DECISION REQUIRED` ID
    `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`, recommending acceptance of the locked one-file fix plus
    full proof wall. Record explicit operator acceptance before any Rust edit.
  - Stop if refreshed risk is CRITICAL or the authorized boundary changes.
  - Receipt: on 2026-07-14 the operator explicitly replied
    `DECISION R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE: A`, accepting the recorded HIGH caller-context
    risk, locked `attempt.rs`-only change, and full packet proof wall. Decision receipt commit
    `d788f45c9` received fresh independent built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-REPLAY-STALL.2 — Reconfirm the preserved behavioral red.**
  - Prerequisites: Tasks 0 and 1 complete.
  - Run exact `acceptance_fixtures_integrated_true_stall_stays_active`.
  - Expected: exit `101` at the event-`420` truthful-evidence assertion; checkpoint `5` is still
    `TroubleshootingFrontier / Stalled / Medium`, score `Active / 30 / High`, flagged, but repeated
    evidence is misattributed to sibling calls `421`/`475` rather than target calls `420`/`474`.
  - Unexpected green/different red: preserve output and stop for authority reconciliation.
  - Receipt: exact test exited `101` at `acceptance_fixtures.rs:463`, the event-`420` evidence
    assertion. Log: `/tmp/r6-replay-stall-pre-edit-red.log`.

- [x] **R6-GAP-DET-REPLAY-STALL.2A — Resolve the post-pairing progress/recovery scope gate.**
  - Decision: `R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01`.
  - Evidence: the uncommitted pairing candidate makes the new unit test `1 / 1` green and truthfully
    pairs `420 -> 423` Failed `101`, `421 -> 425` Clean `0`, `474 -> 477` Failed `101`, and
    `475 -> 479` Clean `0`.
  - Secondary reds: exact `CTX-R6-02` exits `101` at `acceptance_fixtures.rs:455` because actual
    progress is `InsufficientEvidence` rather than locked `Stalled`; the test stops before evidence or
    score assertions. Log: `/tmp/r6-ctx-r6-02-green.log`.
  - Walls: library `144` pass / `13` fail; checkpoints integration `109` / `24`;
    `dead_end_thrash` `15` / `3`; progress corpus `2` / `1`; synthetic helpers have unmatched IDs.
  - Diagnosis: `assess_troubleshooting_progress` selects clean sibling `475`; comparability excludes
    the failed target lane. Impact is LOW (`4` direct / `12` total / `0` processes / `2` modules);
    shared comparability helpers remain out of scope.
  - Initial frozen-control diagnosis: rollout `019e894a-86c9-71e3-b57b-e3d3285f0988` becomes
    `HistoricalOnly` instead of `Recovered` after truthful pairing. Recorded `recovery_state` impact
    is HIGH (`1` direct / `30` impacted / `3` processes / `2` modules).
  - Receipt: on 2026-07-14 the operator explicitly replied
    `DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A`, selecting the docs-first same-packet
    amendment and accepting the recorded HIGH `recovery_state` boundary.
  - Disposition: complete as a scope decision only. It authorized this amendment, not unconditional
    implementation. Amendment diagnosis proved that `recovery_state` is the wrong seam and that
    preserving frozen `Recovered` conflicts with canonical state semantics. At this historical `.2A`
    boundary Task `.2B` still had to resolve that conflict; selected Option A has since done so.

- [x] **R6-GAP-DET-REPLAY-STALL.2B — Resolve the recovered-semantics conflict — OPTION A ACCEPTED.**
  - Decision: `R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02`.
  - Clean baseline: at `f898d61e7`, exact
    `acceptance_fixtures_representative_sticky_success_tail_stays_recovered` passes `1 / 1` (log
    `/tmp/r6-clean-head-sticky-exact.log`). Checkpoint `9` is `Regressing` with flagged
    `Active / 40`; checkpoint `10` is `Recovered / 20`.
  - Candidate truth: output pairing makes checkpoint `9`'s concurrent clean wall visible, so
    checkpoint `9` is `Advancing`, unflagged `HistoricalOnly / 20`, and checkpoint `10` is unflagged
    `HistoricalOnly / 20`.
  - Corrected causality: expected-negative human evidence `831 -> 837`, exit `1`, is not causal.
    `recovery_state` does not read attempt outcomes. Canonical `Recovered` requires an immediately
    previous same-class `Active`; truthful pairing removes that prerequisite.
  - Verified impacts at `f898d61e7`: `assess_troubleshooting_progress` LOW (`4 / 12 / 0 / 2`);
    `recovery_state` HIGH (`1 / 30 / 3 / 2`); `drift_state_for_score` LOW (`1 / 4 / 1 / 2`);
    `assign_drift_states` LOW (`2 / 4 / 1 / 2`). Accepted HIGH `recovery_state` scope is not edit
    authority because it is the wrong seam; all recovery/state functions remain forbidden unless a
    later explicit operator choice authorizes them.
  - Selected Option A: reclassify sticky `CTX-R6-06` to
    `HistoricalOnly / 20`, unflagged; preserve truthful pairing and canonical transition semantics;
    authorize the selected bounded amendment in Task `.3`.
  - Rejected Option B: discard the pairing candidate and retain frozen `Recovered`, leaving `CTX-R6-02`
    unresolved and every downstream replay gate blocked.
  - Rejected Option C: explicitly redefine `Recovered` beyond immediately previous
    same-class `Active` and authorize a broad cross-family contract/test review.
  - Receipt: on 2026-07-14 the operator explicitly replied
    `DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A`. This selects current
    `HistoricalOnly / 20`, unflagged authority while preserving truthful pairing and canonical
    immediately-prior-`Active` transition semantics.
  - Disposition: complete as an authority/expected-disposition decision only. Task `.3` has since
    completed the authorized source/test/helper/expected-disposition edits and focused TDD boundary as
    one unstaged candidate. Task `.3A`/`.3B` completed the unexpected exact-red refinement; Task
    `.3C`/`.3D` own the step-12 expected-fact reconciliation before Task `.4` resumes integrated proof,
    result recording, staging/commit gates, atomic commit, and fresh review.

- [x] **R6-GAP-DET-REPLAY-STALL.3 — Prepare the selected bounded pairing/progress implementation — COMPLETE AT FOCUSED-CANDIDATE BOUNDARY.**
  - Initial candidate file: `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`.
  - Change: call-ID-exclusive scanning when a command ID exists; no positional fallback in that lane;
    unchanged legacy adjacency/Error/Unknown fallback without an ID.
  - Unit regression: `checkpoints_pair_concurrent_tool_outputs_by_call_id` proves interleaved calls
    receive only their matching outputs and outcomes.
  - Selected Option-A amendment within fresh-review-clean series `d631e0c56` + `6498c343f`:
    - keep the `attempt.rs` candidate unchanged in intent;
    - edit only `assess_troubleshooting_progress` plus new private lane helpers. Keep current attempts
      in event order; a current attempt is a lane tail exactly when no later current attempt satisfies
      existing `attempts_are_comparable` with it. Evaluate each tail with the existing one-tail logic
      against prior-checkpoint plus earlier-current attempts directly comparable to that tail. Shared
      comparability is read-only; do not add transitive clustering or edit `attempts_are_comparable`;
    - aggregate only informative lanes (statuses other than `InsufficientEvidence`) using this table:
      zero informative => ordinary `InsufficientEvidence`; one => unchanged; any `Mixed` => `Mixed`;
      `Advancing` plus `Stalled` or `Regressing` => `Mixed`; otherwise any `Regressing` =>
      `Regressing`; otherwise any `Stalled` => `Stalled`; otherwise => `Advancing`.
      `InsufficientEvidence` never overrides an informative result;
    - for multiple informative lanes, merge signals and counter-evidence in lane-tail event order and
      pass them through existing finalization dedupe/caps. Confidence is the minimum informative-lane
      confidence and `Mixed` is additionally capped at `Medium`; a single informative lane preserves
      confidence and evidence unchanged;
    - repair malformed synthetic helpers by keeping general helpers ID-less and giving explicit
      identity helpers matched call/output IDs;
    - add exact tests `troubleshooting_concurrent_clean_sibling_does_not_erase_repeated_failed_lane`
      (`Stalled + InsufficientEvidence => Stalled / Medium`, failure-lane evidence only),
      `troubleshooting_advancing_lane_ignores_unrelated_insufficient_sibling`
      (`Advancing + InsufficientEvidence => Advancing` unchanged),
      `troubleshooting_conflicting_informative_lanes_aggregate_mixed` (positive plus negative =>
      `Mixed`, both signal polarities, conservative confidence), and
      `troubleshooting_regressing_dominates_only_negative_lanes`
      (`Regressing + Stalled => Regressing`); and
    - rename the future sticky test to
      `acceptance_fixtures_representative_sticky_success_tail_is_historical_after_truthful_pairing`,
      updating its test function/assertion plus sticky `expected.json` to
      `HistoricalOnly / 20`, unflagged, never raw rows. In
      `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`, update that
      sticky assertion to `HistoricalOnly / 20`, unflagged while the other three explicit postures
      remain unchanged.
  - Impact every additional existing symbol before editing it. Under selected Option A, do not edit
    `recovery_state`, `drift_state_for_score`, `assign_drift_states`, shared comparability, scorer
    logic, compactor logic, raw fixtures outside Task `.3F`'s exact four-file synthetic identity
    exception, schemas, replay presentation, sentinel surfaces, R7, or R8.
  - TDD receipt: the focused pairing unit and four named troubleshooting units each pass `1 / 1`.
    Every authorized source/test/helper/expected-disposition edit is present in one complete
    eight-file **unstaged** candidate. Task `.3` did not stage, commit, or claim implementation
    review-clean.
  - Current receipt: complete at the focused-candidate boundary. After Task `.4`'s bounded
    single-lane preservation fix and Task `.3B`'s verified-edit-epoch refinement, the preserved
    candidate patch SHA-256 is
    `70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`. It remains uncommitted;
    no recovery/state or other forbidden seam is authorized.

- [x] **R6-GAP-DET-REPLAY-STALL.3A — Land and fresh-review the verified-edit-epoch docs gate — REVIEW-CLEAN.**
  - Unexpected witness: Task `.4` steps 1-5 each pass `1 / 1`; step 6 `troubleshooting` passes
    `18 / 18`; step 7 exact `CTX-R6-02` exits `101` because checkpoint `5` is `Mixed / Medium`, not
    locked `Stalled / Medium`. Log: `/tmp/r6-task4-07-ctx-r6-02-exact.log`. Diagnostic:
    `/tmp/r6-task4-debug-ctx-checkpoint5.json`.
  - Diagnosis: current-checkpoint lane tails `392`/`393` are `Advancing`, `474` is `Stalled`, and
    `475` is `InsufficientEvidence`. The latest source/test edit before newest informative tail `474`
    is event `463`. Event `486` is inside checkpoint `5` (boundary `489`), but its would-be verifying
    calls `492`/`493` and outputs `495`/`496` are outside checkpoint `5`; within that checkpoint,
    event `486` is therefore a trailing unverified edit.
  - Locked refinement: compute all lane-tail results and the newest informative tail; begin the active
    terminal verified-edit epoch after the latest source/test edit preceding that tail; aggregate only
    tails after the cutover. Earlier attempts remain eligible as directly comparable history; later
    unverified edits do not erase the last proven frontier. At checkpoint `5`, cutover `463` retains
    tails `474` and `475`, producing `Stalled`.
  - Authority preserved: do not change the exact lane-tail definition, direct comparability,
    aggregation table, forbidden seams, Task `.2B` Option A, or sticky authority. No operator decision
    is required because `CTX-R6-02` remains locked at `Stalled / Active`, and Task `.2B` changed only
    sticky `CTX-R6-06`.
  - Files: exactly this packet SPEC/PLAN/TASKS; canonical mirrors remained untouched and the eight-file
    candidate was preserved byte-for-byte.
  - Receipt: commit `7812dd5ef` received fresh independent built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-REPLAY-STALL.3B — Implement and focus-prove verified-edit-epoch selection — COMPLETE.**
  - Activation receipt: `.3A` commit `7812dd5ef` is fresh-review-clean.
  - The locked refinement is implemented only within the already authorized
    `assess_troubleshooting_progress`/private-helper boundary; every authority and forbidden seam
    listed in `.3A` is preserved.
  - Exact tests
    `troubleshooting_latest_verified_edit_epoch_supersedes_earlier_informative_lanes` and
    `troubleshooting_unverified_trailing_edit_does_not_erase_latest_informative_lane` completed red,
    then green. The complete candidate remains unstaged.

- [x] **R6-GAP-DET-REPLAY-STALL.3C — Land and fresh-review the progress expected-fact docs gate — REVIEW-CLEAN.**
  - Unexpected witness: Task `.4` steps 1-7 each pass `1 / 1`; step 8 `troubleshooting` passes
    `20 / 20` across library and integration targets; steps 9-11 exact `CTX-R6-02`, sticky
    `CTX-R6-06`, and frozen corpus each pass `1 / 1`. Step 12
    `progress_acceptance_cases_match_expected_progress_contract` exits `101`; log
    `/tmp/r6-task4b-12-progress-acceptance.log`.
  - Diagnosis: case `real-reopen-regressing-019e894a-ord7` remains
    `TroubleshootingFrontier / Regressing / High` with `PreviouslyCleanScopeBroken`. Truthful call-ID
    pairing exposes clean event `109 -> 116`, exit `0`, followed by failing event `636 -> 638`, exit
    `101`. Raw rows, status, dimension, confidence, signals, and evidence minima are unchanged.
  - Locked reconciliation: replace only `earlier later stage verification attempt` with
    `earlier clean verification attempt` and `later regressing verification attempt` with
    `later failing verification attempt`.
  - Files: exactly this packet SPEC/PLAN/TASKS; canonical mirrors remained untouched and the original
    eight-file candidate was preserved byte-for-byte at patch SHA-256
    `70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`.
  - Authority: no operator decision is required because the classification and semantic authority do
    not change; only stale expected wording is reconciled to the corrected pairing facts.
  - Receipt: commit `961a36574` received fresh independent built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-REPLAY-STALL.3D — Reconcile two progress expected facts and preserve the next red — COMPLETE AT EXACT-EDIT BOUNDARY.**
  - Activation receipt: `.3C` commit `961a36574` is fresh-review-clean.
  - Files: exactly `crates/agent-drift-analyzer/tests/progress_acceptance.rs` and
    `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/real-reopen-regressing-019e894a-ord7/expected.json`.
  - Receipt: changed only the two phrases locked in `.3C`; no source, other fixture, packet/canonical
    docs, raw rows, status, dimension, confidence, signals, evidence minima, or other expected fact
    changed.
  - Focused `progress_acceptance_cases_match_expected_progress_contract` exits `101`; log
    `/tmp/r6-task3d-progress-acceptance.log`. The next reported case,
    `synthetic-implementation-advancing`, is `InsufficientEvidence`, not locked `Advancing`.
  - The complete ten-file candidate remains unstaged at patch SHA-256
    `7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`; its original eight-file
    portion remains `70ed522cc30f8e4fb3f36f946502105e440f7719c1917fe41c399e13876884c2`.

- [x] **R6-GAP-DET-REPLAY-STALL.3E — Land and fresh-review the synthetic fixture identity docs gate — REVIEW-CLEAN.**
  - Unexpected witness: after `.3D`'s exact two-phrase repair, focused progress acceptance exits `101`
    because `synthetic-implementation-advancing` is `InsufficientEvidence`, not `Advancing`; log
    `/tmp/r6-task3d-progress-acceptance.log`.
  - Diagnosis: in that synthetic fixture, verifier calls event `1` (`call-1`) and event `5`
    (`call-5`) have semantic outputs events `2` and `6`, but the outputs are id-less and therefore do
    not pair under the selected call-ID-exclusive contract. Event `4` (`call-4`) is `apply_patch`, has
    no output, and needs no change.
  - Complete deterministic malformed set: an audit of all `16` progress-acceptance bundles and `32`
    archival/compact row files also finds id-less semantic outputs in
    `synthetic-zero-verifier-anti-flap`: event `2` for `call-1`, event `4` for `call-3`, event `7` for
    `call-6`, event `9` for `call-8`, and event `11` for `call-10`.
  - Locked repair: in both archival and compact copies, add only matching `dedupe_identity` values
    with `type: function_call_output` to those seven semantic outputs (`14` physical rows). Do not
    change `call-4`, any other row, another fixture, an expected contract, source, test, packet or
    canonical mirror, or any native/adapted/real fixture.
  - Locked postures: keep `synthetic-implementation-advancing` at
    `AutonomousImplementation / ImplementationVerificationWall / Advancing / Medium` with
    `FailureFrontierAdvanced` and `WorkingSetConcentrated`; keep
    `synthetic-zero-verifier-anti-flap` at `PlanningConvergence / InsufficientEvidence / Low`.
  - Files: exactly this packet SPEC/PLAN/TASKS. Preserve the complete ten-file candidate byte-for-byte
    at `7f12cdd2d4c0c92ea08027b3610f22f5168c5ad99b973b0ed3cbcc508cbf98db`; stage and commit only these
    docs, then obtain fresh built-in `default` review/fix cycles until `REVIEW CLEAN`.
  - Authority: no operator decision is required because malformed synthetic output identities
    violate the already-selected truthful call-pairing contract; this is not a new product or semantic
    choice.
  - Receipt: commit `4615d9e3c` received fresh independent built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-REPLAY-STALL.3F — Repair and focus-prove synthetic fixture output identities — COMPLETE.**
  - Activation: only after `.3E` is committed and fresh-review-clean.
  - Files: exactly the `rows.archival.jsonl` and `rows.compact.jsonl` files for
    `synthetic-implementation-advancing` and `synthetic-zero-verifier-anti-flap` under
    `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/`.
  - Add matching `function_call_output` identities only to implementation events `2` (`call-1`) and
    `6` (`call-5`), and zero-verifier events `2` (`call-1`), `4` (`call-3`), `7` (`call-6`), `9`
    (`call-8`), and `11` (`call-10`), in both archival/compact copies.
  - Run a deterministic identity audit over all `16` bundles/`32` row files, then run exact
    `progress_acceptance_cases_match_expected_progress_contract`. Both must be green with the locked
    postures unchanged; stop with the complete candidate unstaged.
  - Do not edit another row, fixture, expected contract, source, test, packet/canonical mirror,
    native/adapted/real fixture, or an identified call without an output.
  - Receipt: deterministic audit passed `16` bundles / `32` row files / `8,388` rows with all seven
    locked logical outputs present as `14` matching physical `function_call_output` identities; log
    `/tmp/r6-task4d-00-identity-audit.log`. Exact progress acceptance passed `1 / 1`, preserving both
    locked postures; log `/tmp/r6-task4d-12.log`. The four fixture-row files are included in the
    complete atomic candidate and no other progress fixture row was edited.

- [x] **R6-GAP-DET-REPLAY-STALL.4 — Run exact integrated proof, commit, and close fresh review — COMPLETE / REVIEW-CLEAN.**
  - Partial receipt: proof steps 1-7 each pass `1 / 1`; step 8 `troubleshooting` passes `20 / 20`
    across library and integration targets; steps 9-11 each pass `1 / 1`. Step 12 first exposed the
    stale expected-fact wording recorded in `.3C`; after `.3D` applied the exact repair, focused proof
    exposed the later malformed synthetic identity red. No files were staged or committed.
  - Resume only after Task `.3F` completes, then rerun the exact proof sequence from the beginning.
  - Verify in order:

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

  - The compactor normalization command is optional confirmation, not edit authority.
  - Selected Option-A acceptance: target evidence events `420`/`474`; siblings `421`/`475` excluded;
    `CTX-R6-02` checkpoint/score unchanged; clean target `492 -> 495`, sibling `493 -> 496`; sticky
    expected disposition alone becomes `HistoricalOnly / 20`, unflagged, with raw rows unchanged; the
    frozen `dead_end_thrash` corpus updates that sticky assertion while its other three postures remain
    unchanged; all walls green.
  - Record actual results in this TASKS and the replay ledger; stage only the intended implementation,
    tests, expected-disposition, TASKS, and ledger files; run staged GitNexus detect, cached diff check,
    and cached diff inspection; commit the implementation/proof atomically; then obtain fresh built-in
    `default` review and fix findings in new gated commits until `REVIEW CLEAN`.
  - Preserved bounded-gap witness: the first from-the-beginning rerun reached step 13 and exposed six
    delegation tests whose generic synthetic helpers assigned false call identities, causing
    call-ID-exclusive pairing to withhold their outputs; log `/tmp/r6-task4c-13.log`. The bounded
    helper-only repair keeps general helper rows ID-less and uses explicit identified helpers only
    where delegation identity is semantically required, across the existing test modules/helpers in
    `checkpoint/mod.rs`, `checkpoint/progress.rs`, `tests/checkpoints.rs`, and
    `tests/dead_end_thrash.rs`. Audit log `/tmp/r6-task4d-00b-delegation-helper-audit.log` confirms the
    general/identified split. No production scoring, recovery/state, comparability, compactor, schema,
    replay-presentation, R7, or R8 seam changed.
  - Final ordered proof receipt with `CARGO_TARGET_DIR=/tmp/r6-task4d-target`: steps 1-7 each passed
    `1 / 1`; step 8 passed `20 / 20` (`10` library + `10` integration); exact `CTX-R6-02`, exact sticky
    `CTX-R6-06`, frozen corpus, and progress acceptance each passed `1 / 1`; checkpoint filter passed
    `62` library + `131` checkpoint integration plus all matching integration targets; full analyzer
    passed `402 / 402` including `163` library, `8` acceptance, `133` checkpoints, `18`
    `dead_end_thrash`, `3` progress acceptance, `22` truth-grounding, and `6` wrong-plan-branch tests.
    Exact compactor normalization passed `1 / 1`; format, analyzer all-target check, analyzer all-target
    clippy with `-D warnings`, and `git diff --check` were green. Logs:
    `/tmp/r6-task4d-01.log` through `/tmp/r6-task4d-19.log`.
  - Commit boundary: complete fourteen-file code/test/fixture candidate SHA-256
    `ebe4e7156416ecf387ae1ef88a067cf1d05855f8fc1089e080e92e7824c5d8a3` plus this TASKS and the
    proof ledger are committed atomically by this change. Final staged GitNexus detect is `medium`
    across exactly `16` files / `54` symbols / `3` affected troubleshooting-tail flows, with no
    `HIGH` or `CRITICAL` result; cached-diff check and complete inspection are clean. Fresh independent
    built-in `default` reviewer returned `REVIEW CLEAN` for commit `6eda87e60`. Task `.4` is complete.

- [x] **R6-GAP-DET-REPLAY-STALL.5 — Land and review the narrow packet transition — REVIEW-CLEAN.**
  - Prerequisite: Task 4 review-clean.
  - This transition marks `CTX-R6-02` and this packet complete, clears the active packet to `none`,
    keeps `R6-REPLAY` active, and made `CTX-R6-06` then the family wall the next phase-owned work;
    both later passed under the separate replay proof receipt.
  - Result: transition commit `1ff592823` plus sticky-wording fix `7839a7f47` received fresh
    independent built-in `default` `REVIEW CLEAN`.
  - Stop the packet after review-clean transition. Do not activate `R6-CLOSE` or start R7/R8.

## Preserved Witness Receipt

- Fixture: `019eb311-c7ce-7f50-ae13-b51a5b5461c3`, annotated real depth-1 built-in `default`
  subagent rollout.
- Commit: `60cde3dd7` (`test: preserve true-stall replay evidence red`).
- Raw target failures: `420 -> 423`, `474 -> 477`; successful siblings: `421 -> 425`, `475 -> 479`.
- Later clean target/sibling: `492 -> 495`, `493 -> 496`.
- Trusted disposition: checkpoint `5` `TroubleshootingFrontier / Stalled / Medium`; flagged
  `dead_end_thrash Active / 30 / High`.
- Historical committed-baseline defect: evidence named successful siblings because
  `pair_output_rows` used positional pairing across concurrent calls. Commit `6eda87e60` corrects
  that attribution and preserves the trusted disposition; its ordered proof is green and a fresh
  independent built-in `default` reviewer returned `REVIEW CLEAN`. The packet is complete and active
  packet is `none`; transition series `1ff592823` + `7839a7f47` is fresh independent `REVIEW CLEAN`.
  Later phase-owned `CTX-R6-06` replay and the family wall are green; their separate proof/fix series
  `b1791c1e3` + `e6d43eee9` + `61c9d5074` received fresh independent built-in `default` `REVIEW
  CLEAN`. The later narrow phase transition activates `R6-CLOSE` at entry only with `CTX-R6-17`
  next; this packet itself did not activate `R6-CLOSE`.
