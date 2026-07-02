# Spec: Agent Drift Analyzer Rolling / Previous-Checkpoint Semantic Goal Drift (R6-3)

Status: draft spec created on 2026-07-02 after `R6-2` closed (the kickoff-anchored `semantic drift from
kickoff/plan/docs` dimension is landed and sign-off-reviewed). This spec is the implementation authority for
the `R6-3` packet defined in `docs/specs/r6/MAP.md` (item 6) and `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
("Packet Decomposition", `R6-3` section). `R6-3` is the **rolling / previous-checkpoint** half of the
semantic-goal-drift signal: the complement to `R6-2`'s cumulative kickoff-anchored measure.

Authority order for this packet:
`docs/specs/r6/MAP.md` owns landing order and the `R6` family routing;
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` owns the resolved
objective-consumption boundary and the new-consumer architecture;
`docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md` owns the landed
kickoff-anchored scorer this packet extends;
this SPEC/PLAN/TASKS family owns the `R6-3` implementation contract; the live crate
(`crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` for the landed scorer,
`src/checkpoint/mod.rs` for `CheckpointAnalysis` / `analysis.previous` / `sanctioned_replan`,
`src/context/objective.rs` for the structured sidecar) is the ground truth for current behavior.

## Assumptions I'm Making

1. **`R6-3` extends the landed `R6-2` scorer; it does not add a new `DriftClass` variant and does not bump
   the schema (resolved — see Resolved Decisions 1 and 2).** Rolling drift surfaces as **tagged evidence on the
   existing `DriftClass::SemanticGoalDrift` class**, not as a new `DriftClass::RollingSemanticGoalDrift`.
   There is one operator posture ("the goal moved"); the cumulative-vs-step-size distinction lives in the
   evidence reason prefixes. This is an analyzer-local change to `scoring/semantic_goal_drift.rs` and its
   evidence — **no** lockstep to `checkpoint/schema.rs` (the `DriftClass` enum is unchanged), `checkpoint/export.rs`,
   or sentinel `operator_surface.rs`, and **no** `schema_version` bump (stays `v0.7`). This is why `R6-3`
   has no `R6-2.2`-style variant-blast-radius packet.
2. **The previous-checkpoint goal is reachable with no new plumbing.** The scorer already receives
   `analysis: &CheckpointAnalysis`, and `CheckpointAnalysis` already carries `previous: Option<CheckpointSlice>`
   (`checkpoint/mod.rs`). The current goal is read at `analysis.current.context.objective`; the previous goal
   is the sibling `analysis.previous.<slice>.context.objective`. Contrast `R6-2`'s kickoff anchor, which is
   *session-level* and had to be threaded into `score_session` (`kickoff_anchor: Option<&StructuredObjective>`).
   `R6-3` threads nothing new: `score_semantic_goal_drift` keeps its current signature and reads
   `analysis.previous` internally.
3. **Rolling drift is scored only when both adjacent goals are present and confident (symmetric presence
   guard — resolved, see Resolved Decision 3).** Both the current and the immediately-previous checkpoint
   goals must pass the same `eligible_current_goal` bar the landed scorer already applies to the current
   goal: `objective_class == TaskStatement`, `Confidence::Medium`-or-higher, empty `unknowns`, and a
   non-empty `comparison_key`. First checkpoint (`analysis.previous` is `None`) → no rolling claim
   (conservative). Previous goal absent/unknown → no rolling claim. This mirrors `R6-2`'s conservative
   no-claim contract.
4. **Rolling and kickoff-anchored drift are computed independently inside the same scorer; the class flags
   if either fires (resolved — see Resolved Decision 6).** `score_semantic_goal_drift` computes the
   kickoff-anchored comparison (as landed) and the rolling comparison (new), and flags `SemanticGoalDrift`
   when either diverges and the checkpoint is not a sanctioned replan. Evidence names which comparison
   fired; both may co-fire on the same checkpoint with distinct evidence lines. The posture, `raw_score`
   (`DRIFT_RAW_SCORE`), and `confidence` shape are unchanged from `R6-2` — one class, one posture.
5. **The distance primitive is reused, but extraction is symmetric (resolved — see Resolved Decision 4).**
   Rolling drift reuses the same disjoint-set overlap over normalized structured-goal terms that `R6-2`'s
   `semantic_goal_diverged` uses (SPEC `R6-2` Resolved Decision 7). Because both the current and previous
   goals are full `ObjectiveSummary`s (each carrying its own `comparison_key`), rolling extracts terms
   **symmetrically** via `goal_specific_terms(structured, Some(summary))` on **both** sides. The
   anchor-side asymmetry that `R6-2` carries (its kickoff anchor is a bare `StructuredObjective` with no
   `ObjectiveSummary`, so it omits `comparison_key` segments) therefore does **not** apply to `R6-3`. This
   is a deliberate reuse-with-a-known-difference, not silent inheritance (MAP item 6).
6. **The sanctioned-replan exclusion is reused as-is.** A checkpoint whose `analysis.sanctioned_replan` is
   true is an authorized pivot and must not be flagged — for the rolling comparison exactly as for the
   kickoff-anchored one. `R6-3` reads the existing `CheckpointAnalysis.sanctioned_replan` field (SPEC `R6-2`
   Resolved Decision 4); it adds no new replan detection.
7. **Rule-based, interpretable, additive.** Rolling distance is a deterministic structured-term comparison,
   not a learned monitor. The packet is additive: it must not change `R6-2`'s kickoff-anchored outcomes,
   the `R6-1` `dead_end_thrash` posture, or regress `R5.75-3` (delegated stability) / `R5.75-4`
   (zero-verifier anti-flap).

If any of these assumptions drift, update this spec before implementation.

## Objective

Add a **rolling / previous-checkpoint** semantic-goal-drift signal that flags when a session's current
effective goal has lurched away from the *immediately-previous* checkpoint's goal — the "did we lurch this
checkpoint?" question — computed from the structured objective sidecar (`comparison_key` + typed goal
anchor) with a symmetric presence guard, surfaced as tagged evidence on the existing
`DriftClass::SemanticGoalDrift` class. This is the complement to `R6-2`'s cumulative kickoff-anchored
measure: rolling catches abrupt single-checkpoint pivots that a slow cumulative drift threshold can miss.

Primary users:

1. operators who need "the agent abruptly changed goal this checkpoint" surfaced as an explicit, tagged
   drift reason, distinct from a slow cumulative drift from the original ask;
2. the sentinel/replay path, which should render the rolling reason line on an already-supported
   `SemanticGoalDrift` posture with no sentinel code change;
3. the later full structured-native migration, which gains a second proven consumer of `comparison_key`
   with no additional schema surface.

This packet succeeds when:

1. a session with an abrupt single-checkpoint goal pivot (without an authorized replan) scores
   `SemanticGoalDrift`, with rolling-tagged evidence naming the previous goal and the lurched-to goal;
2. a slow legitimate evolution — adjacent checkpoints that still share a specific structured term — does
   **not** flag rolling drift;
3. rolling drift is scored **only** when both adjacent goals are present and confident; a first checkpoint
   (`analysis.previous` is `None`) or an absent/unknown previous goal yields no rolling claim, proven by
   tests;
4. a sanctioned explicit replan is **not** flagged as rolling drift;
5. the signal is derived from `comparison_key` / structured terms, not `task_frame.objective`;
6. `R6-2` kickoff-anchored outcomes, `R5.75-3`/`R5.75-4`, and the `R6-1` `dead_end_thrash` posture do not
   regress; **no** new `DriftClass` variant, `schema_version` bump, or sentinel/`export.rs` change is
   introduced.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer` (analyzer-local; **no** coordinated `agent-drift-sentinel` code
  change, because the `SemanticGoalDrift` class already exists and its operator mapping is landed)
- Live code seams for this packet:
  - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (extend: add the rolling comparison,
    symmetric term extraction, rolling evidence tag; the landed kickoff-anchored path is preserved)
  - `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (read: `CheckpointAnalysis`, `analysis.previous`,
    `analysis.sanctioned_replan`)
  - `crates/agent-drift-analyzer/src/context/objective.rs` (read: `ObjectiveSummary`, `StructuredObjective`,
    `comparison_key`)
  - `crates/agent-drift-analyzer/tests/` (new rolling regressions + acceptance fixture)
  - `crates/agent-drift-sentinel/tests/` (a rendering assertion that a rolling-tagged `SemanticGoalDrift`
    checkpoint surfaces its evidence — test-only, no sentinel source change)
- This packet does **not** touch `progress.rs` comparability/reset (that is the conditional `R6-4`), the
  `DriftClass` enum, `export.rs`, or sentinel `operator_surface.rs` source.

## Commands

Focused scorer + rolling regressions:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer rolling -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full analyzer wall + sentinel walls for closeout (the rolling evidence renders through the sentinel
surface even though no sentinel source changes):

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

## Project Structure

```text
docs/specs/r6/MAP.md
  Landing-order authority for the R6 family (item 6 is the R6-3 next-seam charter).
docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
  Packet Decomposition; the R6-3 rolling/previous-checkpoint charter and the "do not conflate them" guidance.
docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md
  The landed kickoff-anchored scorer this packet extends (esp. R6-2 Resolved Decisions 4, 6, 7).
docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-{spec,plan,tasks}.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
  The landed scorer; R6-3 extends it with the rolling comparison + rolling evidence tag (symmetric extraction).
crates/agent-drift-analyzer/src/checkpoint/mod.rs
  CheckpointAnalysis.previous (the previous-checkpoint slice) and CheckpointAnalysis.sanctioned_replan (read-only here).
crates/agent-drift-analyzer/src/context/objective.rs
  ObjectiveSummary + comparison_key: the typed state both sides consume (read-only here).
```

## Code Style

Rolling drift is a second, independent comparison inside the same scorer. It reads the previous goal from
`analysis.previous` (no threading), applies the same `eligible_current_goal` bar to **both** sides, reuses
the disjoint-set distance primitive with **symmetric** extraction, respects `analysis.sanctioned_replan`,
and attaches evidence under a distinct rolling reason prefix so operators can tell it apart from the
kickoff-anchored reason.

```rust
// Existing kickoff-anchored reason prefixes (R6-2), unchanged:
//   CURRENT_GOAL_REASON_PREFIX     = "semantic goal drift current goal:"
//   KICKOFF_ANCHOR_REASON_PREFIX   = "semantic goal drift kickoff anchor:"
// New rolling reason prefixes (R6-3), so the two comparisons stay distinguishable on the same class:
const ROLLING_CURRENT_REASON_PREFIX: &str  = "rolling semantic goal drift current goal:";
const ROLLING_PREVIOUS_REASON_PREFIX: &str = "rolling semantic goal drift previous goal:";

// No new parameter: the scorer already receives `analysis`, which carries `.previous`.
pub(crate) fn score_semantic_goal_drift(
    analysis: &CheckpointAnalysis,
    kickoff_anchor: Option<&StructuredObjective>,
) -> ScoredDrift {
    let current_goal = eligible_current_goal(&analysis.current.context.objective);

    // Kickoff-anchored (R6-2, cumulative) — unchanged.
    let kickoff_drift = current_goal.as_ref().is_some_and(|cur| {
        eligible_anchor_goal(kickoff_anchor)
            .is_some_and(|anchor| semantic_goal_diverged(cur, anchor)) // asymmetric: anchor has no summary
    });

    // Rolling (R6-3, step-size) — both sides are full ObjectiveSummary, so extraction is SYMMETRIC.
    let rolling_drift = match (current_goal.as_ref(), previous_eligible_goal(analysis)) {
        (Some(cur), Some(prev)) => rolling_goal_diverged(cur, &prev), // both via goal_specific_terms(.., Some(summary))
        _ => false, // first checkpoint / absent / unknown previous -> conservative no-claim
    };

    if analysis.sanctioned_replan || !(kickoff_drift || rolling_drift) {
        return no_claim(/* confidence as today */);
    }
    // One SemanticGoalDrift claim; attach evidence for whichever comparison(s) fired,
    // tagged kickoff-anchored vs rolling. Posture/raw_score/confidence unchanged from R6-2.
}
```

Conventions for this packet:

- Read the previous goal from `analysis.previous`; never thread a new session-level input, and never read
  `task_frame.objective` for the drift signal.
- Apply the same `eligible_current_goal` bar to both the current and previous goals (symmetric); extract
  terms with `Some(summary)` on both sides.
- Reuse the disjoint-set distance primitive; do not introduce a graduated/weighted distance here (that
  stays the deferred `R6` iteration named in `R6-2` Resolved Decision 7).
- Exclude sanctioned explicit replans via the existing `analysis.sanctioned_replan` field.
- Tag rolling evidence distinctly from kickoff-anchored evidence; do not add a new `DriftClass` variant.
- Keep the scorer rule-based and evidence-first; no learned monitor.

## Testing Strategy

1. **Rolling presence-guard regressions** (new `tests/...`): current+previous both present+confident and
   disjoint → rolling flagged; first checkpoint (`analysis.previous` is `None`) → no rolling claim;
   previous present-but-unknown (`ObjectiveUnknown` / non-`TaskStatement`) → no rolling claim. Each state
   asserted.
2. **Step-size-vs-evolution regressions**: an abrupt single-checkpoint pivot (adjacent goals fully
   disjoint) → flagged; a slow legitimate evolution (adjacent goals share a specific structured term) →
   not flagged.
3. **Rolling-vs-replan regression**: an abrupt pivot that is a sanctioned explicit replan
   (`analysis.sanctioned_replan == true`) → not flagged.
4. **Independence / co-fire regressions**: (a) a checkpoint that drifts from kickoff but not from the
   previous checkpoint still flags with kickoff-anchored evidence only (rolling silent), proving `R6-2`
   behavior is preserved; (b) a checkpoint that drifts from both surfaces both evidence tags on the one
   `SemanticGoalDrift` claim; (c) a checkpoint that lurches from the previous but shares a term with
   kickoff flags with rolling evidence only; **(d) a checkpoint with NO eligible kickoff anchor (anchor
   absent or below the `High` bar — the common early-session case, before a `High` anchor is captured) but
   an eligible, disjoint previous goal still flags with rolling evidence.** This is the load-bearing
   independence case: the landed scorer hard-returns `no_claim` on an absent anchor
   (`semantic_goal_drift.rs`, the `eligible_anchor_goal` early return) *before* any drift logic, so a naive
   edit that keeps that early return would silently kill rolling exactly when it is most valuable. The edit
   must restructure so an absent anchor does not short-circuit the rolling comparison.
5. **Structured-source proof**: rolling drift changes with `comparison_key`/structured state and is
   independent of the bridge-patched display string. Includes the **symmetric-extraction lock** (Resolved
   Decision 4): a previous goal whose only distinguishing term lives in its `comparison_key` (not in
   `target`/constraints) still participates in the rolling comparison, so a naive
   `semantic_goal_diverged(cur, prev_structured)` reuse — which passes the previous side as a bare
   `StructuredObjective` and drops its `comparison_key` — fails this test.
6. **Acceptance fixture**: a committed session with an abrupt mid-session pivot that scores rolling drift,
   locked like the `semantic_goal_drift_acceptance` corpus.
7. **Sentinel rendering (test-only)**: a rolling-tagged `SemanticGoalDrift` checkpoint renders its evidence
   through the operator surface (mirroring the `R6-2` `operator_surface_renders_flagged_semantic_goal_drift_*`
   coverage) with no sentinel source change.
8. **Non-regression**: `R6-2` kickoff-anchored witnesses, `R5.75-3`/`R5.75-4`, and the `R6-1`
   `dead_end_thrash` posture unchanged; full analyzer + sentinel walls green.

## Boundaries

- **Always:**
  - verify `R6-2` (the kickoff-anchored `semantic_goal_drift` scorer + `SemanticGoalDrift` class) is landed
    before editing; if it is missing or the scorer signature differs from this spec, stop and report rather
    than compensating here;
  - read the previous goal from `analysis.previous` and the current goal from `analysis.current`; read the
    drift signal from structured state only;
  - apply the symmetric eligibility bar to both adjacent goals and assert each guard state (flag / first-
    checkpoint-None / previous-unknown);
  - reuse the disjoint-set distance primitive with symmetric extraction; tag rolling evidence distinctly;
  - keep `DriftScore`'s output shape and the `SemanticGoalDrift` posture additive; preserve `R6-2`,
    `R5.75`, and `R6-1` behavior.
- **Ask first:**
  - any change that would make rolling drift a separate `DriftClass` variant or bump `schema_version` (that
    reverses Resolved Decision 1 and reopens the cross-crate lockstep + a `v0.8` forward-compat break —
    only do it if a flip-condition in Resolved Decision 1 is met);
  - the exact rolling eligibility/threshold tuning if the `R6-3.1` corpus check shows the symmetric
    `Medium+` bar under- or over-fires (Open Question 1).
- **Never:**
  - read `task_frame.objective` (the bridge-patched display string) for the drift signal;
  - migrate `progress.rs` comparability/reset here (that is the conditional `R6-4`);
  - flag a sanctioned explicit replan as drift;
  - score rolling drift when `analysis.previous` is `None` or either adjacent goal's fields are unknown;
  - add a learned/hybrid scorer; introduce a graduated distance; or change the `DriftClass` enum,
    `export.rs`, or sentinel `operator_surface.rs` source.

## Success Criteria

1. Rolling semantic goal drift is flagged only on an unauthorized abrupt pivot between two confident
   adjacent structured goals, with rolling-tagged evidence naming the previous and current goals.
2. The rolling guard states are each asserted: flag / first-checkpoint-no-claim / previous-unknown-no-claim.
3. A slow legitimate evolution that keeps one shared specific term is not flagged.
4. A sanctioned explicit replan is not flagged.
5. Rolling drift is provably derived from `comparison_key`/structured state, not the patched display string.
6. `R6-2` kickoff-anchored witnesses, `R5.75-3`/`R5.75-4`, and `R6-1` postures do not regress; **no** new
   `DriftClass` variant, `schema_version` bump, or sentinel/`export.rs` change lands; `cargo test -p
   agent-drift-analyzer -- --nocapture` and `cargo test -p agent-drift-sentinel -- --nocapture` are green.

## Resolved Decisions

1. **Resolved (2026-07-02, codex-consulted): surface rolling drift as tagged evidence on the existing
   `DriftClass::SemanticGoalDrift` class, not a new `DriftClass::RollingSemanticGoalDrift` variant.** A
   second-opinion `codex exec` consult (session `019f2399`) and this packet's author converged on evidence,
   not a variant. Rationale: kickoff-anchored and rolling are two subtypes of the *same* phenomenon (the
   goal moved) sharing one operator posture, `raw_score`, and handling path — unlike `R6-2`'s choice, which
   was between an existing *different* class and a new semantic-goal-drift class (real conflation). Distinct
   evidence reason prefixes preserve the cumulative-vs-step-size distinction the DESIGN's "do not conflate
   them" guidance cares about, without paying a second forward-compat break + `v0.8` schema bump for
   taxonomy not yet needed. A new variant would buy separate analytics/fingerprints but no separate posture
   or handling today. **Promotion path (the flip-conditions that would move this to a variant in a later
   `R6` iteration, recorded so the reuse is honestly reversible, not a silent default):** (i) rolling drift
   needs a different `raw_score` / threshold / debounce / warning policy than kickoff drift; or (ii)
   operators need separate labels/actions in the sentinel surface; or (iii) acceptance evidence shows
   aggregate `SemanticGoalDrift` reporting hides whether a failure was gradual vs abrupt in a way that
   matters operationally. Until one holds, evidence-on-the-existing-class is the shape. **Known cost of the
   evidence approach (recorded, not a blocker):** downstream consumers already key on `DriftClass` identity,
   not evidence subtype — analyzer export stats aggregate per class (`checkpoint/export.rs`) and the sentinel
   `warning_fingerprint` / debounce key joins `drift_class_name(score.class)`
   (`crates/agent-drift-sentinel/src/operator_surface.rs`). So a rolling-only and a kickoff-only
   `SemanticGoalDrift` checkpoint collapse together in aggregate counts and in the warning debounce
   fingerprint; the distinct evidence reason prefixes preserve the per-checkpoint reason for an operator
   reading the checkpoint, but **not** per-subtype rates or per-subtype debounce. This collapse is exactly
   the concrete form flip-condition (iii) would take — if per-subtype analytics or debounce become
   operationally load-bearing, that is the trigger to promote rolling to its own variant.
2. **Resolved (2026-07-02): no `schema_version` bump; the packet is analyzer-local.** Because Resolved
   Decision 1 adds no `DriftClass` variant and no new `DriftScore` field (only new evidence reason
   prefixes), the checkpoint schema is unchanged and stays `v0.7`. There is no cross-crate lockstep and no
   `R6-2.2`-style impact-gated variant packet; `R6-3.1` records a light impact confirmation that the change
   is analyzer-local (scoring module + evidence), not a schema/variant change.
3. **Provisionally resolved (2026-07-02), stop/go-gated by `R6-3.1`: symmetric eligibility bar — both
   adjacent goals at the current-goal bar.** The proposed bar: rolling drift scores only when the current
   **and** the immediately-previous checkpoint goals each pass the landed `eligible_current_goal` bar
   (`TaskStatement`, `Confidence::Medium`-or-higher, empty `unknowns`, non-empty `comparison_key`). There is
   no "origin" to hold to `High` as in `R6-2` Resolved Decision 5: both sides are ordinary per-checkpoint
   goals, so the bar is symmetric. First checkpoint (`analysis.previous` is `None`) or an ineligible previous
   goal → conservative no rolling claim. **This bar is not final until the `R6-3.1` corpus check (Open
   Question 1) confirms it.** `R6-3.1` is a hard stop/go: if the corpus shows the symmetric `Medium+` bar
   over-fires on ordinary checkpoint-to-checkpoint evolution (the reused disjoint-set primitive reads a
   legitimate narrowing as disjoint — `R6-2` Resolved Decision 7 / MAP item 5 debt, to which rolling is
   *more* exposed than kickoff-anchored), the scorer packet does **not** proceed on this bar until it is
   retuned (tighten eligibility, hold `previous` to a higher bar, or open the graduated-distance revisit).
   Unlike `R6-2` Resolved Decision 5 — which `R6-2.1` had already confirmed against the corpus (9/9 `High`
   anchors) before it was called resolved — this bar's confirming evidence is still pending, so it is marked
   provisional on purpose.
4. **Resolved (2026-07-02): reuse the `R6-2` disjoint-set distance primitive, with symmetric extraction.**
   Rolling drift reuses `semantic_goal_diverged`'s disjoint-set overlap over normalized structured-goal
   terms (`R6-2` Resolved Decision 7); it does **not** introduce a graduated/weighted distance (that stays
   the deferred later-`R6` iteration). The one deliberate difference: both adjacent goals are full
   `ObjectiveSummary`s carrying their own `comparison_key`, so rolling extracts terms symmetrically via
   `goal_specific_terms(structured, Some(summary))` on **both** sides. The `R6-2` anchor-side asymmetry (its
   kickoff anchor is a bare `StructuredObjective` with no `ObjectiveSummary`, so it omits `comparison_key`
   segments, per `R6-2` Resolved Decision 7) therefore does not apply to `R6-3`. This satisfies MAP item 6's
   requirement to reuse the primitive *deliberately*, not inherit it silently. The binary-distance v1 debt
   (misses partial-overlap drift; over-flags legitimate narrowing) carries over unchanged and is not
   re-litigated here. **Implementation constraint (locked by a regression in `R6-3.3`):** the landed
   `semantic_goal_diverged(current: &EligibleCurrentGoal, anchor: &StructuredObjective)` is asymmetric *by
   type* — its second argument is a bare `StructuredObjective`, so `goal_specific_terms` is called with
   `None` for that side and its `comparison_key` segments are dropped. Rolling must **not** reuse that
   signature with the previous goal passed as a bare `StructuredObjective`; doing so silently reintroduces
   the asymmetry and drops the previous side's `comparison_key`. Rolling needs both sides carrying their
   `ObjectiveSummary` (two `EligibleCurrentGoal`-like values, or an equivalent symmetric term-set helper),
   and a `R6-3.3` regression must force a case where the previous goal's only distinguishing term lives in
   its `comparison_key` (not in `target`/constraints), so a naive `semantic_goal_diverged(cur,
   prev_structured)` reuse fails the test.
5. **Resolved (2026-07-02): reuse the existing `analysis.sanctioned_replan` exclusion.** An abrupt pivot
   that is an authorized replan must not flag rolling drift, exactly as for kickoff-anchored drift. `R6-3`
   reads the existing `CheckpointAnalysis.sanctioned_replan` field (`R6-2` Resolved Decision 4); it adds no
   new replan detection and does not weaken the existing exclusion.
6. **Resolved (2026-07-02): rolling and kickoff-anchored are computed independently; the class flags if
   either fires.** `score_semantic_goal_drift` keeps returning a single `ScoredDrift` for
   `SemanticGoalDrift`; it flags when the kickoff-anchored comparison OR the rolling comparison diverges
   (and the checkpoint is not a sanctioned replan), attaching the evidence for whichever fired. Co-firing is
   allowed and surfaces both tagged evidence lines. This preserves `R6-2`'s single-posture contract while
   adding the step-size reason.
7. **Resolved (2026-07-02): `progress.rs` comparability/reset is not migrated in this packet.** As in
   `R6-2`, any structured-state migration of the reset surface is the conditional `R6-4`, evidence-gated, to
   honor Guardrail 4. `R6-3` reads `analysis.previous` for scoring only; it does not touch interval/reset
   history.

## Open Questions

1. **Open — hard stop/go gate in `R6-3.1` (corpus check).** Does the symmetric `Medium+` bar (Resolved
   Decision 3) fire on real abrupt single-checkpoint pivots in the committed corpus without over-firing on
   ordinary checkpoint-to-checkpoint evolution? Measure, across the fixture corpus, how often adjacent
   confident `TaskStatement` goals occur and how often they are fully disjoint under the reused distance
   primitive. **This is a stop/go gate, not a note-and-proceed:** the scorer packet (`R6-3.2`) does not
   proceed on the symmetric `Medium+` bar until this check comes back clean. If abrupt pivots are too rare
   to prove the signal, a synthetic acceptance fixture is added first (as `R6-2.4` did). If ordinary
   evolution frequently reads as disjoint (over-fire), the bar is retuned (tighten eligibility, hold
   `previous` to a higher bar) or the graduated-distance revisit is opened as a prerequisite — `R6-3.2`
   does **not** ship an over-firing signal on the un-retuned bar. This is the `R6-3` analogue of `R6-2.1`'s
   confidence-bar corpus check, except `R6-2.1` had already returned clean (9/9 `High` anchors) when its
   decision was called resolved; here the check is still pending, so Resolved Decision 3 is marked
   provisional.
2. **Open — resolve in `R6-3.2` (evidence ordering).** When both comparisons co-fire, in what order do the
   kickoff-anchored and rolling evidence lines render, and does the operator surface de-duplicate the shared
   current-goal line? Decide during scorer implementation; assert the rendered ordering in the sentinel
   rendering test (Testing Strategy item 7). This is a presentation detail, not a posture change.
