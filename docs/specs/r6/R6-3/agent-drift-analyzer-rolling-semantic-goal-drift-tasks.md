# Tasks: Agent Drift Analyzer Rolling / Previous-Checkpoint Semantic Goal Drift (R6-3)

Status: task ledger created on 2026-07-02 from the `R6-3` SPEC/PLAN in this directory. `R6-3` landed on
2026-07-03 after the full analyzer + sentinel walls, with the closeout evidence recorded inline below.
Sequenced after `R6-2` (the kickoff-anchored `semantic_goal_drift` scorer + `SemanticGoalDrift` class),
which closed on 2026-07-01 and passed a final sign-off review on 2026-07-02. This ledger is the closeout
record for the full packet.

Packet prerequisite rule: this packet names `R6-2` (the landed `score_semantic_goal_drift` scorer, the
`DriftClass::SemanticGoalDrift` variant, the `CheckpointAnalysis.sanctioned_replan` field, and the
`analysis.previous` slice on `CheckpointAnalysis`) as landed. Verify all four in live code/tests before
editing. If a named prerequisite is missing, or if `score_semantic_goal_drift(analysis, kickoff_anchor)`
does not match the signature described in the SPEC (it must already receive `analysis`, which carries
`.previous`), stop and report it instead of compensating inside this packet.

Surfacing decision (load-bearing): rolling drift is tagged evidence on the existing `SemanticGoalDrift`
class, **not** a new `DriftClass` variant, and there is **no** `schema_version` bump (SPEC Resolved
Decisions 1-2). The three flip-conditions that would promote rolling to its own variant in a later `R6`
iteration are: (i) rolling needs a different `raw_score` / threshold / debounce / warning policy than
kickoff drift; (ii) operators need separate sentinel labels/actions; or (iii) acceptance evidence shows
aggregate `SemanticGoalDrift` reporting hides gradual-vs-abrupt in a way that matters operationally. None
holds today. Do not reopen the variant question mid-implementation.

## R6-3.0: Docs Lock

- [x] Task R6-3.0.1: Lock the SPEC/PLAN/TASKS family.
  - Acceptance: `docs/specs/r6/R6-3/` contains the spec, plan, and this tasks ledger, and they record: the
    evidence-not-variant surfacing decision plus its three flip-conditions; the no-`schema_version`-bump /
    analyzer-local boundary; the symmetric eligibility bar (both adjacent goals at the current-goal bar);
    the symmetric-extraction reuse of the `R6-2` disjoint-set distance primitive
    (`goal_specific_terms(.., Some(summary))` on both sides); the `sanctioned_replan` reuse; and the
    independent-compute / co-fire contract, including that an absent or ineligible kickoff anchor must not
    short-circuit the rolling comparison. Content is consistent with `docs/specs/r6/MAP.md` item 6, the
    DESIGN `R6-3` charter, and the landed `R6-2` SPEC.
  - Verify: manual review against the `R6` MAP, the DESIGN doc, the `R6-2` SPEC, and live
    `checkpoint/mod.rs` (`analysis.previous`, `sanctioned_replan`) + `scoring/semantic_goal_drift.rs`.
  - Files:
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Result (2026-07-02): locked by commit `2d023ecc9` (`docs: lock R6-3 rolling semantic goal drift spec
    family`). The spec family records the evidence-not-variant surfacing decision, the no-schema-bump
    analyzer-local boundary, the symmetric eligibility/extraction rules, the `sanctioned_replan` reuse, and
    the independence/co-fire contract required by the MAP and DESIGN docs.

- [x] Task R6-3.0.2 (optional parity): Add a packet-prompts artifact matching the `R6-1`/`R6-2` four-file
  convention.
  - Acceptance: if full parity with the earlier packets is wanted, a
    `agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md` artifact exists using the live
    `R6-3.0`-`R6-3.4` numbering and the same source-of-truth constraints as this ledger, including the
    evidence-not-variant decision, the three flip-conditions, the analyzer-local / no-`schema_version`-bump
    boundary, the symmetric `Some(summary)` extraction rule, the `analysis.sanctioned_replan` reuse, and the
    independence rule that an absent or ineligible kickoff anchor must not short-circuit rolling. Not
    required for implementation to proceed.
  - Verify: manual review against the `R6-2` packet-prompts artifact for shape/numbering parity.
  - Files:
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md` (new, optional)
  - Result (2026-07-02): added in commit `28d782c05` (`docs: add R6-3 rolling semantic goal drift packet
    prompts`), keeping the four-file packet-parity artifact in sync with the live `R6-3.0`-`R6-3.4`
    numbering and source-of-truth constraints.

## R6-3.1: Confirm Previous-Checkpoint Reachability And Run The Eligibility/Threshold Corpus Check

- [x] Task R6-3.1.1: Confirm the previous-goal access path and the analyzer-local surfacing; run the
  eligibility/threshold corpus check.
  - Acceptance: (a) recorded confirmation, from live code, that `CheckpointAnalysis` exposes `previous` and
    that the previous goal is reachable at `analysis.previous.<slice>.context.objective` with the same
    `ObjectiveSummary` shape the current goal uses, with **no** new plumbing and **no** signature change to
    `score_semantic_goal_drift` (do not repeat the `R6-2` first-draft mistake of asserting a non-existent
    access path); (b) recorded confirmation that extending the scorer + adding rolling evidence prefixes
    touches no `DriftClass` enum, `export.rs` class list, or sentinel `operator_surface.rs` source and needs
    no `schema_version` bump (the `R6-3` analogue of `R6-2.2`, but a confirmation, not an impact-gated
    variant decision); (c) a corpus finding measuring how often adjacent confident `TaskStatement` goals
    occur and how often they are fully disjoint under the reused distance primitive — confirming the
    symmetric `Medium+` bar fires on real abrupt pivots without over-firing on ordinary evolution, or
    recording that a synthetic acceptance fixture is needed (as `R6-2.4` did) and/or that eligibility
    tightening / the graduated-distance revisit is warranted. Resolves SPEC Open Question 1. **Stop/go
    gate:** `R6-3.2` does not begin the scorer on the symmetric `Medium+` bar unless this check is clean;
    if the corpus shows over-fire, the bar is retuned (tighten eligibility / hold `previous` higher) or the
    graduated-distance revisit is opened first. Record the explicit go/no-go decision in this ledger.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`; finding recorded in this ledger.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - (read-only) `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - (read-only) `crates/agent-drift-analyzer/src/context/objective.rs`
  - Result (2026-07-02):
    - **Previous-goal access path confirmed from live code:** `CheckpointAnalysis.previous` is
      `Option<CheckpointSlice>`, so the previous goal is reachable at
      `analysis.previous.as_ref()?.context.objective` (and the rolling scorer's structured side would be
      `analysis.previous.as_ref()?.context.objective.structured.as_ref()?`). Both the current side
      (`analysis.current.context.objective`) and the previous side
      (`analysis.previous.as_ref()?.context.objective`) are `ObjectiveSummary`. No new plumbing is needed,
      and `score_semantic_goal_drift(analysis, kickoff_anchor)` already receives the `analysis` handle that
      carries `.previous`, so no signature change is required.
    - **Analyzer-local surfacing confirmed:** the `R6-2` prerequisites are landed in live code
      (`score_semantic_goal_drift`, `DriftClass::SemanticGoalDrift`, `CheckpointAnalysis.sanctioned_replan`,
      and `CheckpointAnalysis.previous`). Extending the scorer plus adding rolling evidence prefixes stays
      analyzer-local to `scoring/semantic_goal_drift.rs`; it does **not** require a new `DriftClass`,
      any `checkpoint/export.rs` class-list change, any sentinel
      `crates/agent-drift-sentinel/src/operator_surface.rs` source change, or a `schema_version` bump.
    - **Corpus finding:** across the committed bundle-shaped fixture corpus used here, the check covered
      `18` fixtures, `120` emitted checkpoints, and `102` adjacent checkpoint pairs. Of those adjacent
      pairs, `23` were eligible adjacent `Medium+` / confident `TaskStatement` pairs on **both** sides, and
      `0` of those `23` eligible adjacent pairs were fully disjoint under the reused disjoint-set
      distance primitive. That is: adjacent confident goals do occur in the corpus, but this committed
      corpus contains no real abrupt-pivot witness for the rolling signal and shows no over-fire on
      ordinary evolution.
    - **Decision:** **GO-with-synthetic-fixture.** Proceed on the symmetric `Medium+` bar because the live
      corpus shows no over-fire that would force a retune / tighter eligibility / graduated-distance-first
      prerequisite, but add the explicit synthetic rolling acceptance witness in `R6-3.3` (the same style
      of proof move `R6-2.4` used) because abrupt pivots are too rare in the current committed corpus to
      prove positive firing honestly.
    - **Explicit stop/go gate:** this is **not** a `NO-GO`. `R6-3.2` may proceed on the symmetric
      `Medium+` bar, with the synthetic positive witness deferred to `R6-3.3`.

## R6-3.2: Rolling Scorer Extension + Evidence Tag

- [x] Task R6-3.2.1: Extend `score_semantic_goal_drift` with the rolling comparison and rolling-tagged
  evidence.
  - Acceptance: `scoring/semantic_goal_drift.rs` reads the previous goal from `analysis.previous`, applies
    the landed `eligible_current_goal` bar to it (symmetric with the current goal), and computes a rolling
    divergence via the reused disjoint-set primitive with **symmetric** extraction
    (`goal_specific_terms(.., Some(summary))` on both sides). The landed `semantic_goal_diverged` must
    **not** be reused with the previous side passed as a bare `StructuredObjective` — that drops the
    previous side's `comparison_key` and silently reintroduces the `R6-2` asymmetry (SPEC Resolved
    Decision 4); use two `EligibleCurrentGoal`-like values or an equivalent symmetric helper. The scorer
    excludes sanctioned replans via the existing `analysis.sanctioned_replan`, and computes kickoff-anchored
    and rolling **independently** — the landed `eligible_anchor_goal` early return must be **restructured so
    an absent or ineligible kickoff anchor does not short-circuit the rolling comparison** (rolling must
    still be able to fire with no anchor). The scorer flags the single `SemanticGoalDrift` claim when either
    comparison fires — attaching evidence tagged with distinct rolling reason prefixes (e.g.
    `ROLLING_CURRENT_REASON_PREFIX` / `ROLLING_PREVIOUS_REASON_PREFIX`) so operators can tell rolling from
    kickoff-anchored. Co-fire is allowed; the co-fire evidence ordering / current-goal de-dup is decided and
    asserted (resolves SPEC Open Question 2). No new `DriftClass` variant, no `schema_version` bump, no
    `scoring/mod.rs` signature change, and `DriftScore`'s posture / `raw_score` / `confidence` shape are
    unchanged. Minimal TDD proof lands here: rolling-flag, **rolling-fires-when-kickoff-anchor-absent**,
    first-checkpoint-`None` no-claim, and rolling-vs-replan.
  - Verify: `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture` and
    `cargo test -p agent-drift-analyzer -- --nocapture`.
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/...` (minimal proof only)
  - Result (2026-07-02):
    - Live scorer confirmation: `score_semantic_goal_drift` now reads the previous goal from
      `analysis.previous` via `eligible_previous_goal(analysis)`, applies the same
      `eligible_current_goal(..)` bar to that previous side, and computes rolling divergence with
      `rolling_goal_diverged(&EligibleCurrentGoal, &EligibleCurrentGoal)`, preserving the previous side's
      `comparison_key`/summary-backed extraction instead of falling back to an asymmetric bare
      `StructuredObjective`.
    - Independence + replan confirmation: kickoff-anchored and rolling comparisons are computed
      independently before the no-claim path, so an absent/ineligible kickoff anchor no longer
      short-circuits rolling, while `analysis.sanctioned_replan` still suppresses both paths.
    - Evidence/boundary confirmation: rolling evidence lands with distinct
      `ROLLING_CURRENT_REASON_PREFIX` / `ROLLING_PREVIOUS_REASON_PREFIX` tags, co-fire ordering is asserted
      in the co-fire scorer test (originally
      `semantic_goal_drift_cofire_preserves_family_order_and_keeps_both_current_goal_lines`; superseded on
      2026-07-03 by `semantic_goal_drift_cofire_dedupes_current_goal_and_surfaces_rolling_previous` — see the
      "Post-Landing Codex Review Fixes" section, which de-dups the redundant rolling current-goal line so the
      previous-goal line survives the default evidence cap), and no new `DriftClass`, no `schema_version`
      bump, and no `scoring/mod.rs` signature change landed — the implementation commit `d47c5e751` touched
      only `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`.
    - Minimal proof confirmation: the scorer test module covers rolling flag, rolling-fires-when-kickoff-
      anchor-absent, first-checkpoint-`None` no-claim, and rolling-vs-replan via
      `semantic_goal_drift_flags_rolling_pivot_with_named_previous_and_current_evidence`,
      `semantic_goal_drift_rolling_still_flags_without_confident_anchor`,
      `semantic_goal_drift_skips_rolling_when_previous_checkpoint_is_absent`, and
      `semantic_goal_drift_skips_rolling_pivots_when_sanctioned_replan_is_present`.
    - Verifier status: `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture` and
      `cargo test -p agent-drift-analyzer -- --nocapture` were rerun successfully in the parent thread,
      and the live scorer file still matches implementation commit `d47c5e751`, so those verifier results
      still apply to the current repo state.

## R6-3.3: Regressions And Acceptance Fixture

- [x] Task R6-3.3.1: Complete the rolling regression matrix and commit the acceptance fixture.
  - Acceptance: regressions cover all rolling guard states (flag / first-checkpoint-`None` /
    previous-present-but-unknown); step-size pivot (adjacent goals fully disjoint) → flagged vs slow
    evolution (adjacent goals share one specific term) → not flagged; rolling-vs-sanctioned-replan → not
    flagged; independence/co-fire (kickoff-only checkpoint still flags with kickoff evidence and rolling
    silent — proving `R6-2` behavior preserved; rolling-only; both tags co-fire on one claim; **and the
    load-bearing anchor-absent case — no eligible kickoff anchor (absent or below `High`) + an eligible,
    disjoint previous goal → rolling still flags**, locking that the anchor early return was restructured and
    not left short-circuiting rolling); and a structured-source proof (patched display string and structured
    goal disagree → score off the structured goal), **including the symmetric-extraction lock (SPEC Resolved
    Decision 4): a previous goal whose only distinguishing term lives in its `comparison_key` (not in
    `target`/constraints) still drives rolling divergence, so a naive `semantic_goal_diverged(cur,
    prev_structured)` reuse fails this regression**. A rolling-drift acceptance fixture (an abrupt mid-session
    pivot) is committed and locked like the `semantic_goal_drift_acceptance` corpus.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`.
  - Files:
    - `crates/agent-drift-analyzer/tests/...` (new rolling acceptance + scorer regressions)
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - Result (2026-07-02): landed in commit `74fc7eb19` (`test: lock R6-3.3 rolling semantic goal drift
    coverage`). The scorer regression matrix now covers the rolling guard states, rolling-only /
    kickoff-only / co-fire independence, the anchor-absent path, the structured-source proof, and the
    symmetric-`comparison_key` extraction lock. The bounded acceptance fixture
    `tests/fixtures/semantic_goal_drift_acceptance/synthetic-rolling-mid-session-pivot` is committed and
    exercised by `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`.

- [x] Task R6-3.3.2: Add the sentinel evidence-rendering test (test-only, no sentinel source change).
  - Acceptance: a sentinel test constructs a checkpoint with a flagged rolling-tagged `SemanticGoalDrift`
    score and asserts the operator surface renders the rolling evidence line(s) on the existing
    `SemanticGoalDrift` posture — mirroring the `R6-2` `operator_surface_renders_flagged_semantic_goal_drift_*`
    coverage. No `operator_surface.rs` source change is made (the `SemanticGoalDrift` mapping already
    exists); this proves the rolling reason renders with no sentinel code work.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`.
  - Files:
    - `crates/agent-drift-sentinel/tests/operator_surface.rs` (test-only)
  - Result (2026-07-02): the test-only sentinel coverage now includes
    `operator_surface_renders_flagged_semantic_goal_drift_rolling_evidence_lines` in
    `crates/agent-drift-sentinel/tests/operator_surface.rs`, proving rolling-tagged evidence renders on the
    existing `SemanticGoalDrift` posture with no `operator_surface.rs` source change.

- [x] Task R6-3.3.3: Assert `R6-2` / `R5.75` / `R6-1` non-regression.
  - Acceptance: the `R6-2` kickoff-anchored acceptance witnesses, `R5.75-3` (delegated stability) /
    `R5.75-4` (zero-verifier anti-flap) witnesses, and the `R6-1` `dead_end_thrash` posture are unchanged by
    the rolling extension. Any change to a prior witness is treated as a regression, not a rebaseline.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture` and
    `cargo test -p agent-drift-sentinel -- --nocapture`.
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - Result (2026-07-03): the follow-up commits `8605f78d2` (`test: restore R6-2 kickoff witness in R6-3.3
    coverage`) and `2738d9e0c` (`test: fail-close R6-2 semantic goal drift witness`) finished the
    non-regression lock. On 2026-07-03 the full walls are green again, including
    `tests/progress_acceptance.rs`, `tests/semantic_goal_drift_acceptance.rs`, and the full
    `agent-drift-sentinel` package wall, so the `R6-2` / `R5.75` / `R6-1` witnesses remain unchanged.

## R6-3.4: Smoke And Closeout

- [x] Task R6-3.4.1: Full walls, confirm no schema/variant/sentinel-source change, and close out `R6-3`.
  - Acceptance: the full analyzer wall and full sentinel wall are green; a recorded confirmation that no new
    `DriftClass` variant, no `schema_version` bump, and no `export.rs` / sentinel `operator_surface.rs`
    source change landed (grep/diff evidence in this ledger). The `R6-4` open/defer decision is recorded:
    did any `R6-1`/`R6-2`/`R6-3` replay evidence show a `progress.rs` reset error caused by objective-string
    quality? If yes, route to `R6-4`; if no, keep `R6-4` deferred to the later full-migration phase. The MAP
    `R6-3` status is promoted to landed and the `R6-4` decision updated.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture` and
    `cargo test -p agent-drift-sentinel -- --nocapture`; `git diff` shows no `DriftClass` enum /
    `schema_version` / `export.rs` / `operator_surface.rs` source change.
  - Files:
    - `docs/specs/r6/MAP.md` (R6-3 status/routing + R6-4 open/defer decision)
  - Result (2026-07-03): Packets `R6-3.0` through `R6-3.3` are confirmed landed in the committed range
    (`2d023ecc9`, `28d782c05`, `ba05e6c3e`, `d47c5e751`, `486c0729d`, `74fc7eb19`, `8605f78d2`,
    `2738d9e0c`), and the required full walls are green:
    `cargo test -p agent-drift-analyzer -- --nocapture` and
    `cargo test -p agent-drift-sentinel -- --nocapture`.
  - Finding: No new variant/schema/sentinel-source fallout landed across `R6-3`. Using pre-`R6-3` base
    commit `169dd2757` (`docs: reconcile R6-2 anchor-term wording and file sign-off backlog`) and the
    fixed `R6-3` closeout upper bound `08bfaed64` (`docs: close R6-3 rolling semantic goal drift`),
    `git diff 169dd2757..08bfaed64 -- crates/agent-drift-analyzer/src/checkpoint/schema.rs crates/agent-drift-analyzer/src/checkpoint/export.rs crates/agent-drift-sentinel/src/operator_surface.rs`
    is empty. Live grep confirms the only relevant analyzer/sentinel surfacing remains the existing
    `DriftClass::SemanticGoalDrift` entry in `checkpoint/schema.rs`, `checkpoint/export.rs`, and sentinel
    `operator_surface.rs`, with no `RollingSemanticGoalDrift` symbol and the schema gates still capped at
    `v0.7`.
  - Finding: `R6-4` stays **deferred**. The existing MAP already recorded that no `R6-1`/`R6-2` replay
    evidence showed a `progress.rs` reset error caused by objective-string quality, and the committed
    `R6-3` range itself does not touch `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    (`git diff 169dd2757..08bfaed64 -- crates/agent-drift-analyzer/src/checkpoint/progress.rs` is empty;
    `git log 169dd2757..08bfaed64 -- crates/agent-drift-analyzer/src/checkpoint/progress.rs` is empty).
    The 2026-07-03 analyzer wall still includes the progress and semantic-goal-drift acceptance corpora
    green, so no new replay/reset evidence justifies opening conditional `R6-4`.

## Post-Landing Codex Review Fixes (2026-07-03)

A post-landing codex second-opinion review (consult session `019f2894`) confirmed the packet landed to
intent (no new `DriftClass` variant, no `schema_version` bump, no `schema.rs`/`export.rs`/sentinel
`operator_surface.rs` source change, symmetric extraction, `sanctioned_replan` reuse, anchor does not
short-circuit rolling, both walls green) and surfaced one operator-visibility defect plus two test gaps,
all fixed here.

- [x] Fix R6-3.F1: De-dup the shared current-goal line on co-fire (SPEC Open Question 2 re-resolved).
  - Defect: under the sentinel's **default** `WarningPolicy` (`max_evidence_lines = 3`), a co-firing
    `SemanticGoalDrift` checkpoint emitted 4 evidence lines ordered
    `[current, kickoff-anchor, rolling-current, rolling-previous]`; the redundant rolling current-goal line
    (same goal already named by the kickoff current line) pushed the informative `rolling ... previous goal:`
    line — the one naming what the goal lurched away from — past the cap, so the operator never saw it. The
    only sentinel rolling test masked this by widening the cap to 4.
  - Fix: `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` now suppresses the
    `ROLLING_CURRENT_REASON_PREFIX` line when kickoff drift already surfaced the current goal, so co-fire
    emits `[current, kickoff-anchor, rolling-previous]` (3 lines, survives the default cap). The rolling
    current-goal line is still emitted for a rolling-**only** claim. No posture / `raw_score` / `confidence`
    change; still a single `SemanticGoalDrift` claim.
  - Locked by: `semantic_goal_drift_cofire_dedupes_current_goal_and_surfaces_rolling_previous` (analyzer,
    asserts 3 de-duped lines in order and no rolling current-goal line) and the acceptance fixture
    `synthetic-rolling-mid-session-pivot/expected.json` (drops `rolling semantic goal drift current goal:`
    from `required_reason_prefixes`; adds it to `forbidden_reason_prefixes`).
- [x] Fix R6-3.F2: Assert rolling co-fire evidence ordering at the sentinel surface under the **default**
  policy (Testing Strategy item 7 / Open Question 2 gap).
  - Gap: the sentinel rolling render test overrode `max_evidence_lines` to 4 and only asserted presence, not
    ordering, so it never exercised default-policy rendering.
  - Fix: replaced with `operator_surface_renders_cofire_rolling_previous_line_in_order_under_default_policy`
    in `crates/agent-drift-sentinel/tests/operator_surface.rs` — uses `WarningPolicy::default()` (cap 3),
    asserts exactly three rendered lines in order (`current`, `kickoff anchor`, `rolling previous goal`), and
    that the rolling current-goal line does not render. Test-only; no sentinel source change.
- [x] Fix R6-3.F3: Add the present-but-below-`High` kickoff-anchor rolling case (Testing Strategy item 4d
  claimed "absent or below `High`" but only `None` was tested).
  - Gap: `semantic_goal_drift_rolling_still_flags_without_confident_anchor` passed `None`; the distinct
    `eligible_anchor_goal` confidence-filter rejection path (a present anchor below the `High` bar) was
    unexercised.
  - Fix: added `semantic_goal_drift_rolling_still_flags_with_below_high_anchor` (analyzer) — a
    `Medium`-confidence anchor is passed as `Some(&anchor)`, rejected by the `High` bar, and rolling still
    flags with rolling-only evidence, proving an ineligible anchor does not short-circuit rolling.
- Verification: `cargo test -p agent-drift-analyzer -- --nocapture` and
  `cargo test -p agent-drift-sentinel -- --nocapture` both green on 2026-07-03 after the fixes.

## Deferred / Ask-First

- [ ] Task R6-3.X.1: Promote rolling drift to its own `DriftClass::RollingSemanticGoalDrift` variant.
  - Acceptance: opened **only** if a SPEC Resolved Decision 1 flip-condition is met — (i) rolling needs a
    different `raw_score` / threshold / debounce / warning policy than kickoff drift; (ii) operators need
    separate sentinel labels/actions; or (iii) acceptance evidence shows aggregate `SemanticGoalDrift`
    reporting hides gradual-vs-abrupt in a way that matters operationally. If opened, it pays the same
    lockstep forward-compat break + `schema_version` bump (`v0.8`) across `scoring/mod.rs` sort order,
    `checkpoint/schema.rs`, `checkpoint/export.rs`, and sentinel `operator_surface.rs` that `R6-2` paid for
    `SemanticGoalDrift`, and is impact-gated via `gitnexus_impact` like `R6-2.2`. If no flip-condition
    appears, this is not written.
  - Verify: to be defined when (and if) opened, with its own SPEC/PLAN/TASKS delta.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-sentinel/src/operator_surface.rs`

- [ ] Task R6-3.X.2: Graduated/weighted semantic distance (shared with `R6-2` Resolved Decision 7 debt).
  - Acceptance: only if `R6-2` or `R6-3` acceptance evidence shows the binary disjoint-set rule under- or
    over-fires (rolling is the more likely over-fire path, since adjacent checkpoints evolve more often than
    kickoff-vs-current). Replace the disjoint-set overlap with a graduated/weighted distance across both the
    kickoff-anchored and rolling comparisons. Deferred to a later `R6` iteration; not written here.
  - **Containment first cut LANDED (2026-07-05).** The acceptance condition was met by the `R6-3.5`
    batch re-run (FINDINGS "R6-3.5 Result": all 5 surviving target-resolved disjoint pairs are legitimate
    narrowing/progression, i.e. the binary rule's over-fire residue). The bounded first cut adds one
    relation beyond exact term equality: **structural path/symbol containment**
    (`goals_in_structural_containment` / `structural_path_ancestor_or_equal` in
    `scoring/semantic_goal_drift.rs`) — a goal is related to another when one structured target
    structurally contains the other, so the canonical narrowing (crate/directory root → one file inside
    it, `foo::bar` → `foo::bar::baz`, and the broadening direction back out) no longer reads as a pivot on
    either the kickoff-anchored or rolling comparison. **Containment is computed on the raw target strings
    (`target.display`/`paths`/`symbols`/…) split only on real structural separators (`/`, `\`, `::`), NOT
    on the normalized term set.** A codex review of the first draft (which ran containment on the
    normalized `_`-flattened terms) found that `normalize_goal_term` collapses `/`, `-`, and `.` to the
    same `_`, so a normalized-prefix test treated `docs/specs/r6-map` and `docs/specs/r6/map.md` as
    ancestor/descendant and silently dropped a real pivot; splitting the raw path on structural separators
    only keeps `-`/`.` inside a segment, so that pivot still fires. Sibling artifacts sharing a lexical
    stem stay unrelated and still fire, preserving every pinned true-positive fixture; comparison_key-only
    goals (no concrete target) fall through to the disjoint check unchanged. Multi-anchor containment is
    **symmetric**, not any-pair (codex re-review §P2, two rounds): a `/goal` clause can name several
    concrete targets (all preserved on `target.paths`/`symbols`), so containment holds only when EVERY
    concrete anchor of BOTH goals is structurally related (ancestor-or-equal, either direction) to some
    anchor of the other. That fires on both a narrowing that adds an unrelated target
    (`docs/specs/r6` → `docs/specs/r6/MAP.md + crates/other/src/lib.rs`) and a broadening that adds one
    (`docs/specs/r6/MAP.md` → `docs/specs/r6 + crates/other/src/lib.rs`); a one-directional all-within test
    masked the broadening case. Segment comparison is **case-sensitive** (codex re-review §P3): the carve-out is applied to raw Rust symbol refs and to
    paths on case-sensitive filesystems, where `Foo::Bar` ≠ `foo::bar`, so a case-only difference stays a
    real pivot rather than a masked narrowing. Segmentation canonicalizes identity-preserving spellings
    before comparison (codex re-review round 4, an over-fire fix): a `./` current-dir segment is dropped
    and a trailing `:line`/`:line:col` reference is stripped (`strip_line_suffix`, mirroring the upstream
    `strip_line_ref` in `context/objective.rs`), so `src/lib.rs` relates to `./src/lib.rs` and `exec.rs`
    to `exec.rs:1537` — a narrowing that only adds one of those common accepted-upstream forms no longer
    fires. Both spellings are no-ops, so canonicalizing them can only remove over-fires, never mask a
    pivot: a Rust `a::b` symbol tail is non-numeric and left intact, and a leading-dot dotfile dir
    (`.github`) remains a real segment (only an exact `.` segment is dropped). The line strip runs on
    the leaf segment after path splitting, not on the whole string (codex re-review round 5): a
    whole-string `split_once(':')` stops at a Windows drive-letter colon, so `C:/repo/src/lib.rs:42`
    never canonicalized and a same-file narrowing still fired on Windows absolute paths; per-leaf
    application mirrors how the upstream `strip_line_ref` is applied (to leaves) in
    `context/objective.rs`. Pinned by unit assertions in
    `structural_path_ancestry_respects_real_separators_only` (incl. Windows drive forms,
    mutation-checked load-bearing against the whole-string variant) and the scorer-level
    `semantic_goal_drift_does_not_flag_line_suffix_narrowing_of_same_file` test (mutation-checked
    load-bearing against the line-strip). Still open for the full graduated/weighted metric:
    family-stem narrowing (`audit-trio.report.json` → `audit-trio.model-selection/…report.json`), doc
    progression, plan→code→plan work cycles, dotted work-item narrowing (`R6-3` → `R6-3.5`, no structural
    separator), the shared-constraint-term masking false negative, and the anchor comparison_key
    asymmetry. **Known residual over-fire, deliberately deferred (codex re-review §P2):** a bare
    `CrateOrPackage` target (`agent-drift-analyzer`) is not matched as an ancestor of a path form of the
    same crate (`crates/agent-drift-analyzer/src/…`), so `Review the agent-drift-analyzer crate` →
    `Update crates/agent-drift-analyzer/src/…` still fires. Closing it needs the package-root convention
    (which lives in `context/objective.rs`, not the scorer); a loose "segment appears anywhere" rule would
    reintroduce the §P1 false negatives, and over-firing never masks drift, so it waits for the graduated
    metric. **Known residual over-fire on case-insensitive filesystems, deliberately declined (codex
    round 6):** a case-only respelling of the same path (`C:/Repo/src` → `c:/repo/src/lib.rs`, `MAP.md` →
    `map.md`) fails containment and still fires on filesystems where those are the same file (default
    macOS/Windows). This is the flip side of the accepted §P3 decision — codex round 2 flagged
    case-INSENSITIVE comparison as a drift-masking false negative (`Foo::Bar` ≠ `foo::bar` for Rust
    symbols; case-distinct paths are different files on Linux, the dominant traced environment), and codex
    round 6 flags case-SENSITIVE comparison as an over-fire; both cannot be satisfied without knowing the
    traced filesystem's case semantics, which the bundle does not carry. Per the scorer's standing rule
    (a false positive is recoverable, a masked pivot is not), the case-sensitive §P3 decision stands;
    closing this needs filesystem-semantics metadata on the bundle or an anchor-type-aware rule
    (symbols case-sensitive, paths per-platform), deferred to the graduated metric. **Known residual
    over-fire, absolute vs repo-relative respelling (2026-07-05 corpus re-check):** an absolute and a
    repo-relative spelling of the same subtree do not relate on raw segments (`docs/legacy` vs
    `/Users/…/handbook/docs/legacy/HARNESS.md`), so a survey→specific-files progression spelled across
    the two forms stays disjoint. Conservative direction (over-fire, never drift-masking); unlike the
    bare-crate case the bundle already carries `session_meta.payload.cwd`, so cwd-prefix stripping is a
    plausible bounded close under the graduated metric.
  - Verify (first cut, run green 2026-07-05, incl. post-codex-review fix): scorer unit tests
    (`cargo test -p agent-drift-analyzer --lib scoring::semantic_goal_drift`, incl. the
    `structural_path_ancestry_respects_real_separators_only` unit test and the
    `semantic_goal_drift_still_flags_hyphen_collision_pivot_through_scorer` regression guard), the
    acceptance corpus with the new `synthetic-kickoff-narrowing-into-anchored-subtree` case
    (`cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`), and the
    full analyzer wall. Verify for the full graduated metric: to be defined when (and if) opened.
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/synthetic-kickoff-narrowing-into-anchored-subtree/`
  - Follow-on validation result (R6-3.6, 2026-07-05): the bounded acceptance wall now has `10` cases with
    **`5/5` positive controls firing** and **`5/5` negative controls staying quiet**, including
    acceptance-level coverage for the hyphen-collision / same-basename-without-extension / sibling-stem
    non-containment boundaries. The fresh seed-42 corpus rerun (`110` sessions / `44` repos / `938`
    checkpoints, `110/110` analyzed clean) held at **`0` fires**, with the explicit export-derivable
    funnel now reported by `scripts/dev/drift-batch-scan/tabulate.py`: `263` structured-target
    checkpoints, `255` analysis-only stable-target-proxy checkpoints, `138` current-bar eligible,
    `263` target-resolved eligible, `221` adjacent target-eligible pairs, `213` same-target exact-match
    suppressions, `8` changed-target candidates, and `7` remaining disjoint pairs. All `7` are still
    documented non-pivots in the open graduated-distance remainder (doc progression, generic
    `spec/plan` respelling, family-stem narrowing, sibling work under one dir, plan→code→plan,
    absolute-vs-repo-relative subtree respelling, broad spec→examples progression). Export limits are now
    explicit: structural-containment suppressions, scorer-true stable-target-hygiene suppressions, and
    `sanctioned_replan` suppressions are not derivable from current checkpoint exports without duplicating
    Rust scorer logic or widening export/schema. Routing decision unchanged: proceed to the **remaining**
    graduated / weighted-distance work in `R6-3.X.2`; keep `R6-3.X.3` deferred. The docs-only scaffold for
    that next remainder packet is `R6-3.X.2B` at
    `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-{spec,plan,tasks}.md`,
    covering analyzer-local relation taxonomy as the **only authoritative routing surface** (numeric score
    explanatory only), pairwise role-shift relations instead of implied hidden cycles, stronger family/generic
    false-negative guards, explicit GitNexus impact-analysis requirements before editing shared scorer helpers,
    prior-witness non-regression, a seed-42 / `R6-3.6`-baseline-pinned 110-session rerun, and a validation-strata
    gate that cannot pass via all-unknown reporting. `RepoRelativeEquivalentAfterCwdStrip` stays conditional:
    land it only if the scorer seam can already reach cwd/session-root truth analyzer-locally; otherwise defer
    / ask-first rather than widening scope into `context/objective.rs` or other deferred surfaces.

- [ ] Task R6-3.X.3: Loosen the shared drift-eligibility bar from `unknowns.is_empty()` to a
  target-resolved gate. **Batch scan done 2026-07-03; the data argues AGAINST loosening in isolation — see
  "Batch scan outcome" below. Still deferred / ask-first. Codex `019f2964` reviewed the batch and a
  junk-filter gate re-count (2026-07-03) confirmed extraction-first; both are recorded below.**
  - Motivation: real-session probes on 2026-07-03 (`019e9864-…` exploratory, `019f2837-…` concrete-goal)
    run through the live `agent-session-compactor` -> `agent-drift-analyzer` pipeline both produced `0`
    eligible checkpoints and `0` rolling/kickoff drift, and a codex consult (`019f2927`) confirmed the cause:
    `eligible_current_goal` requires `unknowns.is_empty()` — every structured field resolved, including
    off-surface `success_conditions` / `deliverables` cues the extractor deliberately rejected (normal
    scaffolding like "Verify…" / "Return with…"). For a drift signal only the grounded target's movement
    matters; `deliverables` do not participate in divergence and `success_conditions` barely do. So the bar
    is mismatched to the drift question, and the real-world bottleneck is objective-decomposition coverage
    upstream of the disjoint-set metric, not the metric itself.
  - Proposed change (codex-recommended, not yet locked): gate `eligible_current_goal` on `TaskStatement` +
    `confidence >= Medium` (High for the anchor) + a **grounded target present** (+ optionally no
    `primary_goal` unknown) — **not** "non-empty `comparison_key` alone," which would admit `unknown_target`
    and is too weak.
  - Blast radius / risk: local to `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`, but the
    helper is shared, so this affects **both** the `R6-2` kickoff-anchored path and the `R6-3` rolling path.
    The main risk is not missing deliverables — it is that once more checkpoints qualify, the coarse binary
    disjoint-set distance fires more often, including **over-fire** on legitimate narrowing/reframing
    (`R6-2` Resolved Decision 7 / MAP item 5 debt). This task is therefore coupled to the graduated-distance
    revisit (`R6-3.X.2`): loosening eligibility without addressing the distance metric may trade under-fire
    for over-fire.
  - Batch scan outcome (2026-07-03, done): 110 real sessions across 43 repos and 11 analyzable months
    (2025-09 → 2026-07; pre-`session_meta` rollouts before ~2025-09 do not produce session-scoped rows and
    cannot be analyzed), 882 checkpoints, run through the live compactor -> analyzer pipeline.
    - Coverage: `TaskStatement`+confident = 714/882 (81%). Current-bar eligible (`unknowns.is_empty()`) =
      156/882 (**17.7%** — so the bar is NOT inert; the two-session probe was unrepresentative). Failure
      blockers among TS+confident: `target` 366, `deliverables` 284, `success_conditions` 202.
    - Loosening yield: a target-resolved bar would admit 348/882 (39.5%), **+192 checkpoints (2.2×)**. Of 299
      adjacent target-eligible pairs, 286 (95.7%) keep the same target, 13 change, **12 are term-disjoint**
      (~12 rolling firing candidates vs the 1 rolling fire the current bar produced).
    - **The decisive finding — the firings are false positives.** The current bar's only 6 flagged
      checkpoints are all ONE session (`0199f9ec`, docs, 2025-10, ords 36-41) and are driven by GARBAGE
      target extraction (current goal parsed as coordinate/fragment noise `0_0_0_0_4000_n|5_n_n|…` while the
      real goal per the kickoff anchor was architecture/auth docs); the single real rolling fire is the same
      garbage. Eyeballing all 12 hypothetical target-only disjoint pairs: they are dominated by (a) garbage /
      fragment targets (`isolated.\n-`->`\n-`, `README.md`->`5\n\n`, `SKILL.md`->`GPT-5.4`) and (b)
      legitimate narrowing / progression (`audit-trio.report.json`->`audit-trio.model-selection/…report.json`;
      `PLAN-04.md`->`exec.rs:1537`->findings->`PLAN-04.md`, a normal plan->code->plan cycle). **No clear
      "abandoned goal A for unrelated goal B" pivot appeared.**
  - Revised verdict (2026-07-03, confirmed by codex `019f2964` and the junk-filter gate re-count): **do NOT loosen the eligibility bar in
    isolation.** The batch shows the problem is not only low coverage but that both the current firings and
    the firings loosening would add are dominated by two over-fire sources — garbage target extraction and
    the disjoint-set metric misreading narrowing/progression as drift. The strict `unknowns.is_empty()` bar
    currently acts as an accidental over-fire suppressor. Gate any loosening behind BOTH (a) objective-
    extraction robustness (suppress garbage/fragment targets in `context/objective.rs`) and (b) the
    graduated-distance metric (`R6-3.X.2`); loosening alone would multiply false positives (~1 -> ~12, nearly
    all spurious), not surface real drift.
  - Junk-filter gate re-count (2026-07-03, done — cheapest cut of the FINDINGS Step 2, analysis-only, no
    code changed; the batch + pipeline scripts were recovered and preserved as reusable tooling at
    `scripts/dev/drift-batch-scan/`): a term-level junk-target filter (escaped-newline residue,
    number/coordinate runs, model/version tokens, prose-`etc`) applied to the same 110-session batch shows
    **0/6 fires survive** (every real firing, including the one rolling fire, is garbage extraction),
    **8/156** current-bar-eligible checkpoints rested on junk-only targets, and the **12 disjoint pairs drop
    to 9** — none of which is a real "abandoned goal A for unrelated goal B" pivot (2 are prose garbage a
    stronger guard also kills, ~7 are legitimate narrowing / progression / plan->code->plan cycles). This
    confirms the ordering quantitatively: extraction hardening in `context/objective.rs` is the dominant
    lever, the graduated-distance metric (`R6-3.X.2`) owns the narrowing residue, and this bar stays
    deferred. Full detail: `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md` → "Gate Result".
  - Verify: to be defined when (and if) opened, with its own SPEC/PLAN/TASKS delta and impact analysis on
    the shared `eligible_current_goal` helper.
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - (upstream, weaker lever) `crates/agent-drift-analyzer/src/context/objective.rs`
