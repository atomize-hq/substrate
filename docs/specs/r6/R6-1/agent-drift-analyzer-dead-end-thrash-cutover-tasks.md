# Tasks: Agent Drift Analyzer Dead-End-Thrash Cutover And Frontier-Aware Process Dimensions (R6-1)

Status: task ledger created on 2026-06-27 from the `R6-1` SPEC/PLAN in this directory. The docs lock is
already committed at HEAD (`eb7225b85`), and Task `R6-1.1.1` was completed on 2026-06-30 via a read-only
frontier-predicate/access-path investigation. Later implementation tasks remain open. This ledger is the
closeout record as tasks land.

Packet prerequisite rule: this packet names `R5.75` as landed. Verify it in live code/tests before
editing (it is, at HEAD — the analyzer wall and sentinel spot-checks are green). If a named prerequisite
were missing, stop and report it instead of compensating inside this packet.

## R6-1.0: Docs Lock

- [x] Task R6-1.0.1: Commit the SPEC/PLAN/TASKS family.
  - Acceptance: `docs/specs/r6/R6-1/` contains the spec, plan, and this tasks ledger, and they record the
    objective-independence boundary, the within-`DeadEndThrash` modeling of the two process dimensions,
    the frozen-corpus regression floor, and the no-`DriftClass`-variant / no-schema-bump boundary.
  - Verify: manual review against the `R6` MAP, the DESIGN doc, and live `dead_end_thrash.rs`.
  - Files:
    - `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-spec.md`
    - `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-plan.md`
    - `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md`
  - Closeout note (2026-06-30): the prerequisite re-check for `R6-1.1` confirmed the docs-lock family is
    already committed at HEAD in `eb7225b85` (`docs(r6): land R6 scorer-cutover spec family with review fixes`),
    so `R6-1.0` is treated as landed history rather than work to repeat in this packet.

## R6-1.1: Choose The Frontier-Access Path And Predicate (Investigation, No Committed Code)

- [x] Task R6-1.1.1: Decide how the scorer reaches the frontier judgment, and which predicate to use.
  - Acceptance: a recorded finding (in this ledger) that (a) names the chosen **access path** — default
    recommendation: compute `build_session_progress`/`build_session_archetype` before `score_session` and
    pass the frontier signal (or `SessionProgress`) into `score_dead_end_thrash` (additive input change +
    reorder; reorder confirmed safe because progress/archetype depend only on `analysis`, not
    `drift_scores`); reject recompute-in-scorer — and (b) names the `SessionProgress` signal(s) that
    express "frontier advanced in this interval" vs "no frontier movement". The chosen path + predicate
    must leave the frozen replay corpus (`019e93fa`/`019e940c`/`019e943c` cleared/0; `019e894a`
    recovered/20/unflagged) and the `R5.75-3`/`R5.75-4` witnesses unmoved. Resolves SPEC Open Question 1
    (both sub-questions) and sizes Task R6-1.2.1.
  - Verify: local experiment (read-only probe over the pipeline order, the corpus, and the witnesses); no
    code committed from this task.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/checkpoint/progress.rs`,
      `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (pipeline order / reorder target),
      `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
  - Finding (2026-06-30): read-only investigation resolved SPEC Open Question 1 as follows:
    - chosen access path for `R6-1.2`: compute `build_session_archetype` and `build_session_progress`
      before `score_session`, then pass the resulting `SessionProgress` (or an additive bool derived from
      it) into `score_dead_end_thrash`. At HEAD, `build_session_checkpoint_from_analysis_with_ordinal`
      still computes `drift_scores` before `build_session_archetype` / `build_session_progress`, and
      `session_progress` exists only on the exported `Checkpoint`, not on `CheckpointAnalysis`, so the
      scorer cannot read the frontier judgment at scoring time without this bounded reorder/input change.
      Recomputing the predicate inside `dead_end_thrash.rs` was rejected because it would fork the
      `progress.rs` source of truth and bypass the existing zero-verifier fallback / delegation-cap logic.
    - chosen frontier predicate: treat the frontier as advanced only when
      `session_progress.dimension == troubleshooting_frontier` and `session_progress.signals` contains one
      of the existing direct troubleshooting-advancement codes already used by `progress.rs` to return
      `ProgressStatus::Advancing`: `failure_frontier_advanced`, `failure_count_reduced`, or
      `verification_clean`. For `dead_end_thrash`, repeated activity with no such signal counts as
      **no frontier movement**.
    - exact signals read by that predicate:
      - `failure_frontier_advanced`: blocked-before-target -> target-exercised or later failure-class
        movement within the troubleshooting pipeline.
      - `failure_count_reduced`: the comparable failing count drops on the same troubleshooting scope.
      - `verification_clean`: a previously failing focused verification target reaches a clean result.
    - signals explicitly **not** treated as frontier movement: `failing_scope_edited` alone, planning
      signals such as `candidate_set_*` / `working_set_*`, and `delegation_visibility_limited`. Those can
      produce `mixed`, `stalled`, or non-troubleshooting lanes without a genuine troubleshooting-frontier
      advance, which is exactly the false-positive path this packet must avoid.
    - why this predicate is the live authority boundary: `progress.rs` already encodes the same split via
      `has_direct_troubleshooting_advancement_signal(...)` -> `ProgressStatus::Advancing`; positive-but-not-direct
      signals remain `mixed`, and negative-only signals become `stalled` / `regressing`. Reusing that
      boundary lets `R6-1.2` consume the existing frontier judgment rather than inventing a second model.
    - corpus / witness confirmation (read-only evidence only; no production changes):
      - frozen `dead_end_thrash` corpus: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_ -- --nocapture`
        passed, preserving `019e93fa-60d4-73d1-9092-014130b60e14`,
        `019e940c-a91b-7fe0-a967-b0bdd595b581`, and `019e943c-668e-7a03-992b-6a98cf3055da` at
        `cleared / flagged=false / raw_score=0`, and preserving
        `019e894a-86c9-71e3-b57b-e3d3285f0988` at `recovered / flagged=false / raw_score=20`.
      - `R5.75-3` / `R5.75-4` witnesses: `cargo test -p agent-drift-analyzer --test progress_acceptance progress_acceptance_cases_match_expected_progress_contract -- --nocapture`
        passed. The packet-owned zero-verifier witness `adapted-zero-verifier-097d97e914ca220f` remains
        `planning_convergence / stalled` with `failure_frontier_advanced` and `verification_clean`
        forbidden; the mixed delegated witness `adapted-parent-visible-da59436e63915185` remains
        `parent_visible_orchestration / stalled` with `failure_frontier_advanced` and
        `verification_clean` forbidden; and the native delegated proof
        `019eb970-3543-7ab1-a5d6-2a62c00c7185` remains `parent_visible_orchestration / mixed`. Because the
        chosen predicate is gated to troubleshooting-dimension direct-advance signals only, those
        witnesses stay outside the churn-suppression lane and remain unaffected.

## R6-1.2: Dead-End-Thrash Cutover

- [ ] Task R6-1.2.1: Re-score `dead_end_thrash` to be frontier-aware (decisive-step, not streak-length).
  - Acceptance: run `gitnexus_impact` on `score_dead_end_thrash` and report the blast radius first. Then:
    repeated activity with an advancing frontier yields `flagged=false` (historical context) with
    churn-with-progress evidence; repeated activity with no frontier movement flags with stall-named
    evidence and a decisiveness-based score; reclassification is guarded on the existing active-thrash
    condition so cleared controls stay cleared/0; `DriftScore`'s output shape and the `DriftClass` enum
    are unchanged (the `score_session`/scorer **input** may change additively only via the R6-1.1 access
    path); every decision carries a named `EvidenceRef`. Add the minimal churn-vs-stall proof here
    (TDD); the full matrix is R6-1.3.
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs` (minimal proof only)

## R6-1.3: Regressions And Corpus Invariance

- [ ] Task R6-1.3.1: Complete the churn-vs-stall matrix (do not re-add R6-1.2's minimal proof).
  - Acceptance: `tests/dead_end_thrash.rs` adds: advancing-frontier-with-repeated-failures →
    `flagged=false`; repeated-activity-no-movement → flagged with stall evidence; a single decisive stuck
    step scored as decisive (not diluted by streak length).
  - Verify: `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`

- [ ] Task R6-1.3.2: Assert frozen-corpus and `R5.75` invariance.
  - Acceptance: `tests/acceptance_fixtures.rs` proves the frozen `dead_end_thrash` corpus keeps posture
    (controls cleared/0; sticky recovered/20/unflagged); `tests/checkpoints.rs` /
    `tests/progress_acceptance.rs` prove `R5.75-3`/`R5.75-4` witnesses unchanged.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

## R6-1.4: Guardrail-5 Bridge Coverage

- [ ] Task R6-1.4.1: Cover the `R5.75-6` bridge needle lists.
  - Acceptance: `tests/checkpoints.rs` adds regressions that exercise `anchor_text_looks_grounded_goal` /
    `narrowed_objective_looks_subordinate` so a phrasing the bridge misses fails the wall (a novel-phrasing
    miss is caught as a bridge gap, not silently mis-scored). The closeout note records legacy-surface
    string-truth as explicit migration debt, not target state.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R6-1.5: Smoke And Closeout

- [ ] Task R6-1.5.1: Full + touched sentinel walls, then MAP status update.
  - Acceptance: the full analyzer wall and the touched sentinel spot-checks are green; the `R6-1` entry in
    `docs/specs/r6/MAP.md` is updated (status + routing note pointing to `R6-2` as the next active seam).
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel warning_policy -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `docs/specs/r6/MAP.md`

## Deferred / Ask-First

- [ ] Task R6-1.X.1: Promote a process dimension to its own `DriftClass` variant.
  - Acceptance: only if operators need `stall` or `expected churn` as distinct top-level classes; must
    update the sentinel `operator_surface.rs` mapping (`drift_class_name`, `historical_reason_prefixes`,
    `checkpoint_had_active_class`) and stay additive.
  - Verify: to be defined when approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-sentinel/src/operator_surface.rs`

- [ ] Task R6-1.X.2: Re-weight the operator-facing `raw_score` bands.
  - Acceptance: only if `R6-1.1` evidence shows the streak bands mislead operators; gated by the sentinel
    spot-checks staying meaningful.
  - Verify: to be defined when approved.
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
