# Design: R6 Drift Scorer Cutover And Objective-Consumption Boundary

Status: design created on 2026-06-27 to resolve `Decision Gate 0` from `docs/specs/r6/MAP.md` before
any `R6` spec/plan/tasks are written. This doc does the deeper consumer/failure-mode research the MAP
named as Task 1, resolves the objective-consumption boundary from that evidence, answers the MAP's
open questions, and fixes the `R6` packet decomposition. It is the gating design artifact: the
per-packet spec/plan/tasks below are written only after this doc is reviewed and approved.

## Relationship To The MAP

`docs/specs/r6/MAP.md` framed `Decision Gate 0` as **open** and deliberately did not pre-commit an
answer. This doc closes it. The MAP remains the family routing authority; this doc is the architecture
and decision record it defers to. When this doc is approved, the MAP's `Decision Gate 0` section flips
to "resolved — see this doc."

## Decision Gate 0 — Resolved

**Question (from the MAP):** Does progress comparability / scorer input migrate onto the structured
objective (`comparison_key` + typed target/evidence) before or alongside scorer retuning, and at what
depth?

**Resolution: a scoped Option C — "new-consumer-first, conditional-reset-migration."**

Concretely:

1. The objective-**independent** scorer cutover proceeds on the existing legacy surface with no
   objective migration. This covers `dead_end_thrash` redesign and the two process dimensions that
   read frontier/verifier/edit signals (`stall without frontier movement`,
   `expected debugging churn with positive progress`).
2. The one dimension that genuinely needs the structured objective —
   `semantic drift from kickoff/plan/docs` — is built as a **new, direct consumer of the structured
   objective sidecar** (`StructuredObjective` goal anchor + `comparison_key`), with an explicit
   sidecar-presence guard. It does **not** read the `R5.75-6` bridge-patched display string. This
   gives `comparison_key` its first live consumer without touching the high-risk `progress.rs` reset.
3. Migrating the existing `progress.rs` reset/comparability surface (Seam 5) onto `comparison_key` is
   **conditional and deferred**: pursued only if `R6-1`/`R6-2` replay evidence shows reset errors
   actually caused by objective-string quality. Absent that evidence it folds into the later
   full-migration phase, and Guardrail 5 is satisfied for the legacy-surface portion by acceptance
   coverage of the `R5.75-6` bridge.

**Why this and not full migration (Option B) or pure legacy retune (Option A):** the research below
shows existing scoring is only weakly and indirectly coupled to objective-text quality, while the
*strong* dependency is exclusively the new semantic-drift dimension. Option B pays for a full
`TaskFrame` coexistence migration that current scorers do not need. Option A ships the new
semantic-drift dimension on a heuristic-patched string, which cannot be honest (it is "drift from the
goal" scored against a goal that is itself a needle-list guess). The scoped Option C spends structured
consumption exactly where it is load-bearing and nowhere else.

## Evidence: How Objective Quality Actually Reaches Scoring

Read of the live scorer and comparability surface (`crates/agent-drift-analyzer/src/scoring/`,
`context/working_set.rs`, `checkpoint/progress.rs`). The three rule-based scorers run by
`score_session` (`scoring/mod.rs`) couple to objective text as follows.

`score_dead_end_thrash` (`scoring/dead_end_thrash.rs`) — **zero coupling.** Inputs are entirely
`analysis.repetition.repeated_*_loops`, `analysis.recovery.active_repeated_*`, and
`command_observations`. No objective text, no truth artifacts. The cutover `R6` cares about most is
fully objective-independent.

`score_wrong_plan_branch` (`scoring/wrong_plan_branch.rs`) and `score_truth_grounding_gap`
(`scoring/truth_grounding_gap.rs`) — **weak coupling.** Both key off `task_frame.truth_artifacts` (a
path set) plus command observations. The truth-artifact set is collected by `collect_truth_artifacts`
(`context/working_set.rs`) from `focusable_directive_rows` + path hints; `objective.text.contains(path)`
only sets the `source` *label* (`objective_literal` vs `directive_literal`), it does not gate set
membership. The scorers consume the path strings, not the source label, so the bridge-patched objective
string has no first-order effect on which paths count as in-scope.

Reset / comparability (Seam 5, `checkpoint/progress.rs`) — **indirect coupling, the only real channel.**
`comparable_window_boundary` resets on five conditions: `strong_archetype_boundary`,
`explicit_replan_boundary`, `delegation_visibility_changed`, `objective_or_truth_artifacts_shifted`,
and `working_set_pivoted`. Two of the five read the objective string:
`explicit_replan_boundary` compares `normalize_task_text(task_frame.objective)` plus a
replan/pivot/switch/instead keyword, and `material_objective_delta` (via
`objective_or_truth_artifacts_shifted`) compares `objective_terms(task_frame.objective)` token sets
gated by `task_frame_transitioned`. Reset boundaries set interval bounds and history carry
(`previous_truth_grounding_gap`, repetition/recovery continuity), so the patched string reaches all
three scorers here — but only as token/keyword heuristics, and only as 2 of 5 boundary signals.

The new dimension `semantic drift from kickoff/plan/docs` — **strong, direct, and not yet built.** This
is "is the agent still working the stated goal?" There is no honest way to compute it from a
heuristic-patched display string; it must read the typed goal anchor and `comparison_key`. This is the
single place the structured objective is load-bearing for `R6`.

Conclusion: the migration the design-arch doc describes is real future work, but it is **not** a
prerequisite for the scorer cutover `R6` was chartered to do. It is a prerequisite only for the one
new dimension — which we satisfy as a fresh direct consumer, not a migration.

## Guardrail Compliance

`R6` inherits the migration design's guardrails. Under this resolution:

- **Guardrail 4 (do not move progress reasoning first):** honored. The high-risk `progress.rs` reset
  surface is not migrated; the structured objective enters via a new, additive scorer dimension, not
  by re-keying comparability. The optional Seam 5 migration (`R6-4`) stays gated behind evidence.
- **Guardrail 5 (no compatibility-string-as-permanent-authority):** honored. The new semantic-drift
  dimension reads typed structured state directly, never the patched string. For the legacy-surface
  scorers (`R6-1`), the `R5.75-6` bridge needle lists (`anchor_text_looks_grounded_goal`,
  `narrowed_objective_looks_subordinate`) are placed under acceptance coverage so a novel-phrasing
  miss is caught as a bridge gap, and string-truth is recorded as migration debt, not target state.
- **Preserve `R5.75` behavior:** any change must keep delegated-stability (`R5.75-3`) and zero-verifier
  anti-flap (`R5.75-4`) outcomes intact; the named smoke/acceptance witnesses carry forward.

## Architecture: The New Semantic-Drift Consumer

The `semantic drift from kickoff/plan/docs` dimension is the structured-objective integration point. Its
contract:

- read the structured objective sidecar from `analysis.current.context.objective` (its
  `structured: Option<StructuredObjective>` goal anchor and the already-computed `comparison_key`; exported
  per-checkpoint as `structured_objective`) — **not** `task_frame`, which has no structured field — plus
  the kickoff/anchor objective for the session;
- compute drift as semantic distance between the current effective goal and the anchored kickoff goal,
  using `comparison_key` / structured terms rather than raw display-string token overlap;
- **sidecar-presence guard (mandatory):** define behavior for all three states the migration doc
  requires — sidecar present and high-confidence (score drift), sidecar absent (no drift claim; stay
  conservative), sidecar present but key fields unknown (no drift claim). Silent fallback to brittle
  string assumptions must be visible in tests;
- keep it rule-based and interpretable in the first cut; learned monitors stay deferred.

This is intentionally a *read* of the sidecar, not a re-plumb of `TaskFrame`. If a later phase migrates
`TaskFrame` (design Phase 2), this consumer migrates with it, but it does not block on it now.

## Packet Decomposition

This resolves the MAP's "how many spec/plan/tasks sets" question. `R6` is **three committed packets plus
one conditional packet**. Two of the committed packets (`R6-1`, `R6-2`) have their spec/plan/tasks sets
written now; the third committed packet (`R6-3`, rolling drift — added 2026-06-30 after the decision to
ship `R6-2` kickoff-anchored first) is written when `R6-2` lands. The conditional `R6-4` set is **not**
written unless replay evidence opens it (see below); until then it has no spec/plan/tasks.

**`R6-1` — `dead_end_thrash` cutover + objective-independent process dimensions.** Redesign
`dead_end_thrash` toward decisive-bad-step semantics (AgentRx "critical failure step" framing) and add
`stall without frontier movement` and `expected debugging churn with positive progress`. Objective-
independent; may start immediately and in parallel with `R6-2` design. Primary files:
`src/scoring/dead_end_thrash.rs`, `src/scoring/mod.rs`, `tests/dead_end_thrash.rs`,
`tests/acceptance_fixtures.rs`. Also lands the Guardrail-5 acceptance coverage of the `R5.75-6` bridge
needle lists. Acceptance: troubleshooting sessions tolerate expected failures when the frontier
advances; the known `dead_end_thrash` replay artifacts (e.g. the cleared controls
`019e93fa`/`019e940c`/`019e943c` and the recovered sticky `019e894a`) keep their honest posture.

**`R6-2` — `semantic drift from kickoff/plan/docs` as a new structured-objective consumer.** Build the
dimension per the architecture above; gives `comparison_key` its first live consumer with a sidecar-
presence guard. Depends on `R6-1` only for shared scorer scaffolding, not semantically. Primary files:
a new `src/scoring/semantic_goal_drift.rs` + `scoring/mod.rs`, `src/checkpoint/schema.rs`
(only if a new `DriftClass` variant is needed — additive at the enum, but see the serde-compat note in the
`R6-2` spec), tests + acceptance fixtures. Acceptance: a
session that pivots away from its kickoff goal scores drift only when the sidecar is present and
confident; absent/unknown sidecar stays conservative; `R5.75` witnesses do not regress.

**`R6-3` — rolling / previous-checkpoint semantic drift (committed follow-up to `R6-2`, added
2026-06-30).** `R6-2` ships the **kickoff-anchored** first cut (drift of the current structured goal from
the session's kickoff/anchor goal — the "still on the original ask?" question). `R6-3` adds the
complementary **rolling** signal: drift of the current checkpoint's structured goal from the
*immediately-previous* checkpoint's structured goal — the "did we lurch this checkpoint?" question.
Rationale (the two comparisons catch different failure shapes — do not conflate them): kickoff-vs-current
is the *cumulative* measure — a slow drift accumulates distance from the origin and eventually flags — so
it is the one that catches gradual drift from the original ask. Previous-vs-current is a *step-size*
measure: it flags an abrupt single-checkpoint pivot immediately, and does not false-positive on slow
legitimate evolution — but on its own it *misses* slow cumulative drift when each step stays under
threshold. `R6-3`'s distinctive value is therefore catching abrupt pivots cheaply: the previous-checkpoint
goal is already reachable via `analysis.previous.context.objective.structured` with **no new plumbing**
(the kickoff anchor, by contrast, is session-level and must be threaded — see `R6-2`). Whether it lands as
a second `DriftClass` variant or as additional evidence on the `R6-2` class is an impact-gated decision
deferred to the `R6-3` spec. Primary files (when written): `src/scoring/` (extends the `R6-2`
`semantic_goal_drift` module), tests + acceptance fixtures. Acceptance: a session with an abrupt
single-checkpoint goal pivot (without a sanctioned replan) scores rolling drift; a slow legitimate
evolution does not; `R5.75`/`R6-1`/`R6-2` witnesses do not regress.

**`R6-4` (conditional) — Seam 5 reset/comparability onto `comparison_key`.** Only opened if `R6-1`/`R6-2`
replay evidence shows reset/continuity errors caused by objective-string quality (e.g. a bridge miss
that wrongly resets or fails to reset a window). If opened, migrate `explicit_replan_boundary` /
`material_objective_delta` to consume `comparison_key` with a sidecar-presence guard, preserving
`R5.75-3`/`R5.75-4` behavior. If the evidence does not appear, `R6-4` is **not** written here; it folds
into the later full structured-native migration phase. Primary files (if opened):
`src/checkpoint/progress.rs`, `tests/checkpoints.rs`, `tests/progress_acceptance.rs`.

Sequencing: `R6-1` first (objective-independent, fastest, unblocks the chartered cutover), `R6-2` next
(the structured integration, kickoff-anchored first cut), `R6-3` after `R6-2` (the rolling /
previous-checkpoint follow-up), `R6-4` only on evidence. Each lands test-green and replay-honest before
the next, mirroring the `R5.75` one-packet-at-a-time discipline.

## Answered Open Questions (From The MAP)

- *Which concrete replay artifacts define `R6` success, and do any fail because of objective quality vs.
  repetition/path signals?* The chartered `dead_end_thrash` artifacts (`019e894a` sticky-recovered;
  `019e93fa`/`019e940c`/`019e943c` cleared controls) are repetition/recovery failures, **not** objective-
  quality failures — they are `R6-1`'s targets and need no objective migration. Objective quality is
  decisive only for the new semantic-drift dimension (`R6-2`). Per-packet specs will pin the exact
  fixture witnesses.
- *Does `semantic drift from kickoff/plan/docs` ship in the first cut, or wait behind the objective
  migration?* It ships in `R6-2` as a direct sidecar consumer — it does not wait behind a `TaskFrame`
  migration, because reading the sidecar does not require migrating it.
- *Can Seam 5 reset consume `comparison_key` without `TaskFrame` Phase 2, given a sidecar-presence guard?*
  Yes in principle (the sidecar is already on `task_frame`/context), but it is deferred to the
  conditional `R6-4` and only if evidence warrants — Guardrail 4 keeps it from being a first move.
- *Minimum acceptance coverage for Option A to be honest under Guardrail 5?* `R6-1` adds regression
  coverage that exercises the `R5.75-6` bridge needle lists directly, so a phrasing the bridge misses is
  caught at the analyzer acceptance wall rather than silently mis-scoring downstream.

## What This Design Does Not Decide

Left to the per-packet spec/plan/tasks (not pre-empted here): exact `DriftClass`/schema field names and
whether a new variant is needed; the precise semantic-distance function for `R6-2`; specific fixture
session ids and threshold numbers; and whether `R6-4` opens at all (evidence-gated). The full
structured-native `TaskFrame`/working-set/progress migration (design Phases 2-3 beyond the conditional
`R6-4` slice) remains a later phase, not part of `R6`.
