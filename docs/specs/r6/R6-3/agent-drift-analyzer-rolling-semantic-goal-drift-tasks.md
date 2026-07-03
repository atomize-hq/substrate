# Tasks: Agent Drift Analyzer Rolling / Previous-Checkpoint Semantic Goal Drift (R6-3)

Status: task ledger created on 2026-07-02 from the `R6-3` SPEC/PLAN in this directory. `R6-3` is **not yet
implemented**; all tasks below are open. Sequenced after `R6-2` (the kickoff-anchored `semantic_goal_drift`
scorer + `SemanticGoalDrift` class), which closed on 2026-07-01 and passed a final sign-off review on
2026-07-02. This ledger becomes the closeout record as tasks land.

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

- [ ] Task R6-3.0.1: Lock the SPEC/PLAN/TASKS family.
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

- [ ] Task R6-3.0.2 (optional parity): Add a packet-prompts artifact matching the `R6-1`/`R6-2` four-file
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

## R6-3.1: Confirm Previous-Checkpoint Reachability And Run The Eligibility/Threshold Corpus Check

- [ ] Task R6-3.1.1: Confirm the previous-goal access path and the analyzer-local surfacing; run the
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

## R6-3.2: Rolling Scorer Extension + Evidence Tag

- [ ] Task R6-3.2.1: Extend `score_semantic_goal_drift` with the rolling comparison and rolling-tagged
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

## R6-3.3: Regressions And Acceptance Fixture

- [ ] Task R6-3.3.1: Complete the rolling regression matrix and commit the acceptance fixture.
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

- [ ] Task R6-3.3.2: Add the sentinel evidence-rendering test (test-only, no sentinel source change).
  - Acceptance: a sentinel test constructs a checkpoint with a flagged rolling-tagged `SemanticGoalDrift`
    score and asserts the operator surface renders the rolling evidence line(s) on the existing
    `SemanticGoalDrift` posture — mirroring the `R6-2` `operator_surface_renders_flagged_semantic_goal_drift_*`
    coverage. No `operator_surface.rs` source change is made (the `SemanticGoalDrift` mapping already
    exists); this proves the rolling reason renders with no sentinel code work.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`.
  - Files:
    - `crates/agent-drift-sentinel/tests/operator_surface.rs` (test-only)

- [ ] Task R6-3.3.3: Assert `R6-2` / `R5.75` / `R6-1` non-regression.
  - Acceptance: the `R6-2` kickoff-anchored acceptance witnesses, `R5.75-3` (delegated stability) /
    `R5.75-4` (zero-verifier anti-flap) witnesses, and the `R6-1` `dead_end_thrash` posture are unchanged by
    the rolling extension. Any change to a prior witness is treated as a regression, not a rebaseline.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture` and
    `cargo test -p agent-drift-sentinel -- --nocapture`.
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`

## R6-3.4: Smoke And Closeout

- [ ] Task R6-3.4.1: Full walls, confirm no schema/variant/sentinel-source change, and close out `R6-3`.
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
  - Verify: to be defined when (and if) opened.
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
