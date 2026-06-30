# R6 Map: Drift Scorer Cutover To Context-Aware Semantics (Rescope Scaffold)

Status: scaffold created on 2026-06-27 after `R5.75` closed. This is a **rescope** of the original
`R6` packet, not its execution plan. The original `R6` line was written in the pre-`R5.75` landing
order (`HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`) before the structured-objective (`SO`)
work under `docs/specs/r5/R5_75/phase-1/SO/` expanded into a full objective architecture. That work
changed the foundation `R6` was assumed to stand on, so `R6` must be re-specced with knowledge of
where `SO` actually landed before any scorer code is touched. This document frames that rescope. It
deliberately left the central scope decision (`Decision Gate 0`) open in its first draft. That gate is
now **resolved** in `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` (scoped
Option C); this map defers to that doc for the resolution and the packet decomposition.

## Why This Rescope Exists

`R5.75` validated test-green and smoke-proven at HEAD, but it closed honestly on two facts that bear
directly on `R6`:

1. The structured objective is an additive, observational sidecar. `comparison_key` is computed but
   has **no live consumer**: `progress.rs` comparability and `working_set.rs` path attribution still
   run off the legacy `task_frame.objective` string. The full structured-native consumer migration
   (Phases 2-3 of `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`)
   was never started.
2. `R5.75-6` made the effective checkpoint objective faithful to the structured goal anchor with a
   bounded, analyzer-local **heuristic bridge** in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
   (`grounded_structured_goal_anchor_text` + `should_prefer_grounded_goal_anchor`, keyed on hardcoded
   needle lists in `anchor_text_looks_grounded_goal` / `narrowed_objective_looks_subordinate`). It
   holds on the proven repros and is correct as a bounded intermediate, but it is exactly the
   "compatibility text stays the real truth forever" debt the migration design doc names (Guardrail 5).

If `R6` retunes scorers on top of that bridge without a conscious decision, it builds scoring on a
patched legacy string whose correctness on novel phrasings rests on those needle lists. The point of
this rescope is to make that decision explicit and evidence-backed, not implicit.

## Original R6 Intent (Preserved)

From the landing-order authority, the original `R6` intent is kept as a hard target for the rescope,
not discarded:

- re-score `dead_end_thrash` and related drift classes using typed outcome evidence, turn context,
  archetype, and progress modules — scorers become consumers of deeper analyzer modules rather than
  home-grown heuristic islands;
- cut over `dead_end_thrash` **first**; revisit `truth_grounding_gap` only if new context exposes an
  obvious improvement; keep `wrong_plan_branch` unchanged unless context surfaces one;
- add explicit scoring dimensions: `stall without frontier movement`,
  `semantic drift from kickoff/plan/docs`, and `expected debugging churn with positive progress`;
- keep the first version rule-based and interpretable; consider learned/hybrid monitors only after the
  rule-based signals and acceptance wall are stable;
- acceptance: troubleshooting sessions tolerate expected failures when the frontier advances, long
  autonomous turns score differently from multi-turn conversational sessions, and flagged sessions
  become materially more honest on known replay artifacts.

Note the one dimension that was always going to depend on objective quality: `semantic drift from
kickoff/plan/docs`. You cannot honestly score drift *away from the goal* if the goal is a
heuristic-patched string. That dimension is the sharpest reason the `SO` foundation and `R6` must be
reconciled rather than run past each other.

## Preliminary Investigation Findings (This Pass)

A focused read of the live scorer surface (`crates/agent-drift-analyzer/src/scoring/`) — enough to
de-risk and frame `Decision Gate 0`, not an exhaustive audit. The planning session should finish it
(see Task 1) before locking the decision.

- `score_session` (`scoring/mod.rs`) runs three rule-based scorers over `CheckpointAnalysis`:
  `wrong_plan_branch`, `truth_grounding_gap`, `dead_end_thrash`.
- `dead_end_thrash` (`scoring/dead_end_thrash.rs`) consumes **only** repetition/recovery signals
  (`analysis.repetition.repeated_*_loops`, `analysis.recovery.active_repeated_*`,
  `command_observations`). It reads no objective text and no truth artifacts. It is **fully
  independent** of the structured-objective question.
- `wrong_plan_branch` (`scoring/wrong_plan_branch.rs`) and `truth_grounding_gap`
  (`scoring/truth_grounding_gap.rs`) consume `task_frame.truth_artifacts` (a path set) plus
  command observations. They depend on the **expected-path set**, not on objective-comparability
  strings. The path set is sourced upstream by `working_set.rs` (Seam 2), which still uses
  `objective.text.contains(path)` substring attribution.
- All three sit downstream of `progress.rs` comparability/reset. `comparable_window_boundary`
  (`checkpoint/progress.rs`) compares `normalize_task_text(task_frame.objective)` strings with `!=`
  plus keyword `contains`, which sets interval/window boundaries and history carry
  (`previous_truth_grounding_gap`, repetition/recovery continuity). This is the single indirect path
  by which the `R5.75-6` bridge-patched objective string reaches every scorer (Seam 5, the
  design doc's "highest-risk downstream consumer of lossy objective text").

What this evidence implies for the gate (framing, not a decision):

- `R6`'s stated first step — `dead_end_thrash` cutover — is objective-independent and can proceed on
  the legacy surface regardless of how the gate resolves.
- The structured-objective question becomes load-bearing specifically for (a) the **new**
  `semantic drift from kickoff/plan/docs` dimension, which would be the first direct structured
  consumer, and (b) the fidelity of comparability/reset (Seam 5) and truth-artifact path attribution
  (Seam 2) that feed `wrong_plan_branch` / `truth_grounding_gap`.
- Therefore the gate is **not** binary "migrate everything first vs. nothing." A surgical middle path
  exists: migrate only Seam 2 and the Seam 5 reset/comparability surface onto structured state,
  without the full `TaskFrame` coexistence (Phase 2) the design doc describes. The session should
  weigh that option alongside the broader ones.

## Decision Gate 0 (RESOLVED 2026-06-27) — The Objective-Consumption Boundary

**Question:** Does progress comparability / scorer input migrate onto the structured objective
(`comparison_key` + typed target/evidence) before or alongside scorer retuning, and at what depth?

**Resolution: scoped Option C — "new-consumer-first, conditional-reset-migration."** The
objective-independent scorer cutover proceeds on the legacy surface; the one dimension that needs the
structured objective (`semantic drift from kickoff/plan/docs`) is built as a new direct consumer of the
structured sidecar with a presence guard; and the high-risk `progress.rs` reset migration is conditional
and evidence-gated. Full evidence, rationale, guardrail compliance, and packet decomposition are in
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`. The options below are retained for
provenance.

> **Naming caution:** this resolved "scoped Option C" is a *composition* (new-consumer-first +
> conditional reset migration). It is **not** the same as candidate "C — Surgical middle" listed below,
> which migrates Seam 2 + Seam 5 wholesale. The candidate letters A/B/C below are provenance only; the
> binding resolution is the composition named here and in the DESIGN doc, not candidate C.

Candidate resolutions to weigh (illustrative, non-exhaustive — the session may compose them):

- **A — Legacy-surface retune.** Retune/redesign scorers on the existing string surface; keep the
  `R5.75-6` bridge as the honesty layer; treat the `SO` migration as a parallel/later track. Cheapest;
  but the bridge's needle lists become load-bearing for scoring correctness, and
  `semantic drift from kickoff/plan/docs` cannot be built honestly under it.
- **B — Full migration first.** Land Phase 2 (`TaskFrame.objective_key` coexistence) + the
  `progress.rs` seam of Phase 3, then retune on typed comparability. Heaviest; aligns scoring with the
  architecture `SO` was building toward; retires/demotes the bridge.
- **C — Surgical middle.** Migrate only Seam 2 (working-set path attribution) and the Seam 5
  reset/comparability surface onto structured state, leaving full `TaskFrame` coexistence deferred.
  Unblocks honest `semantic drift` scoring and improves `wrong_plan_branch` / `truth_grounding_gap`
  fidelity without paying for the whole migration.
- **Split:** in all cases, the objective-independent `dead_end_thrash` cutover can begin immediately;
  the gate governs only the objective-coupled scope.

The decision must be recorded in this file (status flips to a chosen resolution) before any
objective-coupled `R6` code lands.

## Inherited Guardrails (Non-Negotiable)

`R6` inherits these from the migration/integration design doc regardless of which resolution it picks:

- **Guardrail 4 — do not move progress reasoning first.** `progress.rs` is the highest-risk seam; any
  migration touching it must come after (or land jointly with explicit guards for) the seams it
  depends on, never as a careless first step.
- **Guardrail 5 — no compatibility-string-as-hidden-authority as the permanent foundation.** Whatever
  `R6` does, it must not leave the heuristic-patched legacy objective string as the *permanent* basis
  for scoring. If `R6` chooses Option A, it must (a) put the `R5.75-6` bridge needle lists under
  acceptance coverage, and (b) treat any novel-phrasing scoring miss as a bridge gap, not a scorer
  bug — and record string-truth as explicit migration debt, not target state.
- **Preserve earlier `R5.75` behavior.** Any comparability change must keep delegated-stability
  (`R5.75-3`) and zero-verifier anti-flap (`R5.75-4`) outcomes intact.

## Task Order

Tasks 1-3 are **complete** as of 2026-06-27 — the consumer/failure-mode investigation and the
`Decision Gate 0` resolution landed in
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` (which also fixes the packet
decomposition: `R6-1`, `R6-2`, conditional `R6-3`), and the `R6-1` and `R6-2` SPEC/PLAN/TASKS sets are
written (see Packet Documents below). Tasks 4-5 are the remaining execution sequence.

### Packet Documents

- `R6-1` (objective-independent scorer cutover): `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-{spec,plan,tasks}.md`
- `R6-2` (structured-objective consumer): `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md`
- `R6-3` (conditional reset migration): not written — evidence-gated per the DESIGN doc and `R6-2.5`.

1. **[done] Complete the consumer/failure-mode investigation.** Finish the scorer-input audit (turn context,
   archetype inputs, interval/reset history carry, how `truth_artifacts` are sourced), and enumerate
   the concrete `R6` failure modes from the known replay artifacts. Output: the evidence that makes
   `Decision Gate 0` non-arbitrary.
2. **[done] Resolve `Decision Gate 0`.** Resolved to scoped Option C with full evidence and rationale in
   the `R6` DESIGN doc.
3. **[done] Lock `R6` scope + acceptance wall.** The `R6-1` and `R6-2` SPEC/PLAN/TASKS sets are written
   under `docs/specs/r6/R6-1/` and `docs/specs/r6/R6-2/` (Packet Documents above).
4. **`dead_end_thrash` cutover (`R6-1`, gate-independent).** Implement per the `R6-1` SPEC/PLAN/TASKS:
   frontier-aware decisive-step scoring + the two objective-independent process dimensions, rule-based.
5. **Objective-coupled work (`R6-2`, then conditional `R6-3`).** Implement the
   `semantic drift from kickoff/plan/docs` dimension as a new structured-sidecar consumer per `R6-2`; open
   the conditional `R6-3` reset migration only if `R6-1`/`R6-2` evidence warrants. Guardrails enforced in
   tests throughout.

## Non-Goals For This Rescope

- re-litigating `Decision Gate 0` (it is **resolved** to scoped Option C in the DESIGN doc; do not
  reopen A/B/C selection here);
- the full structured-native consumer migration as a foregone conclusion (the resolution defers it);
- `R7` full delegated-session semantics;
- learned/hybrid scoring monitors (explicitly after the rule-based wall is stable);
- scheduler/sentinel tuning beyond touched spot-checks.

## Open Questions For The Planning Session (RESOLVED — see DESIGN "Answered Open Questions")

These four questions framed `Decision Gate 0` and are **answered** in
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` ("Answered Open Questions"). They are
retained here for provenance, not as open work:

- Which concrete replay artifacts define `R6` success, and do any of them fail *because* of objective
  quality vs. purely repetition/path signals? → the chartered `dead_end_thrash` artifacts are
  repetition/recovery failures, not objective-quality failures; objective quality is decisive only for the
  new `semantic drift` dimension (`R6-2`).
- Does `semantic drift from kickoff/plan/docs` ship in the first `R6` cut, or wait behind the objective
  migration? → ships in `R6-2` as a direct sidecar consumer; does not wait behind a `TaskFrame` migration.
- For the resolution, can Seam 5 reset/comparability consume `comparison_key` without `TaskFrame` Phase 2,
  given a sidecar-presence guard? → yes in principle, but deferred to the conditional `R6-3`, evidence-gated.
- Minimum acceptance coverage to keep the legacy surface honest under Guardrail 5? → `R6-1` exercises the
  `R5.75-6` bridge needle lists directly so a missed phrasing fails at the analyzer wall.
