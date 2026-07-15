# Tasks: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / TASK `.1` ACCEPTANCE RECEIPT
`d788f45c9` REVIEW-CLEAN / TASK `.2` COMPLETE / TASK `.2A` ACCEPTED AND COMPLETE / TASK `.2B`
OPTION A ACCEPTED AND COMPLETE / TASK `.3` AUTHORIZED AND CURRENT / TASK `.4` BLOCKED** within
`R6-REPLAY`. Trusted
witness `60cde3dd7` preserves `CTX-R6-02` red. Task `.0` series `200725001` + `08fa86e94` +
`d03f5a355` + `9edf564d3` and Task `.1` receipt `d788f45c9` each received fresh independent built-in
`default` `REVIEW CLEAN`. Task `.2` is complete. The operator accepted Task `.2A` with
`DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A` on 2026-07-14, authorizing this docs-first
same-packet amendment but not unconditional implementation. Diagnosis proves an internal conflict
between frozen `Recovered` and canonical state semantics. The operator then replied exactly
`DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A`. Task `.2B` is complete: current
sticky `CTX-R6-06` authority is `HistoricalOnly / 20`, unflagged, and the old `Recovered / 20`
expectation is historical baseline evidence only. This approved authority/expected-disposition change
still awaits Task `.3` implementation preparation and Task `.4` integrated proof, commit, and review.
The uncommitted Task `.3` candidate is not yet a complete proof-ready candidate or a review-clean
result. Task `.4` is blocked until Task `.3` implementation/focused unit criteria are complete;
`CTX-R6-06` replay proof, the family wall, `R6-CLOSE`, and R7/R8 remain blocked.

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
  - Disposition: complete as an authority/expected-disposition decision only. Task `.3` owns the
    authorized source/test/helper/expected-disposition edits, focused TDD unit red/green, and a
    complete unstaged candidate diff ready for proof. Task `.4` owns the integrated proof wall,
    result recording, staging and commit gates, atomic implementation/proof commit, and fresh review.

- [ ] **R6-GAP-DET-REPLAY-STALL.3 — Prepare the selected bounded pairing/progress implementation — AUTHORIZED / CURRENT.**
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
    logic, compactor logic, raw fixtures, schemas, replay presentation, sentinel surfaces, R7, or R8.
  - TDD boundary: establish red, then green, for the focused pairing unit and the four named focused
    troubleshooting units while making the authorized source/test/helper/expected-disposition edits.
    Task `.3` completes only when the full intended implementation is present as one complete
    **unstaged** candidate diff ready for Task `.4` proof. Do not run the integrated/packet proof wall,
    stage files, run staged GitNexus/cached-diff gates, commit, or claim implementation review-clean
    in Task `.3`.
  - Current state: authorized but incomplete. Candidate patch SHA-256
    `030d3d3e97640ba8fd4cf71f29f886b2273ec6e653aefa622aacaeb7057fdae7`, backed up at
    `/tmp/r6-pairing-fix-secondary-red.patch`, remains uncommitted and unproven. Implement only the
    selected Task `.3` boundary; no recovery/state or other forbidden seam is authorized.

- [ ] **R6-GAP-DET-REPLAY-STALL.4 — Run exact integrated proof, commit, and close fresh review — BLOCKED.**
  - Activation: blocked until Task `.3` implementation/focused unit criteria are complete and its
    complete intended candidate diff remains unstaged.
  - Verify in order:

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

- [ ] **R6-GAP-DET-REPLAY-STALL.5 — Land and review the narrow packet transition.**
  - Prerequisite: Task 4 review-clean.
  - Mark `CTX-R6-02` and this packet complete, clear the active packet, keep `R6-REPLAY` active, and
    make `CTX-R6-06` then the family wall next.
  - Commit authority-only, run the staged gate, dispatch a fresh built-in `default` reviewer, and fix
    findings in new commits until clean.
  - Stop the packet after review-clean transition. Do not activate `R6-CLOSE` or start R7/R8.

## Preserved Witness Receipt

- Fixture: `019eb311-c7ce-7f50-ae13-b51a5b5461c3`, annotated real depth-1 built-in `default`
  subagent rollout.
- Commit: `60cde3dd7` (`test: preserve true-stall replay evidence red`).
- Raw target failures: `420 -> 423`, `474 -> 477`; successful siblings: `421 -> 425`, `475 -> 479`.
- Later clean target/sibling: `492 -> 495`, `493 -> 496`.
- Trusted disposition: checkpoint `5` `TroubleshootingFrontier / Stalled / Medium`; flagged
  `dead_end_thrash Active / 30 / High`.
- Committed-baseline defect: evidence names the successful siblings because `pair_output_rows` uses
  positional pairing across concurrent calls. The preserved uncommitted candidate corrects that
  attribution, but Task `.3` implementation preparation is incomplete and Task `.4` proof, commit,
  and review have not begun.
