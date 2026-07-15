# R6 Map: Drift Scorer Cutover To Context-Aware Semantics (Rescope Scaffold)

Status: **CLOSED — R6-REPLAY, R6-CLOSE, AND CTX-R6-17 COMPLETE; R7-PROMOTE COMPLETE; R7 AUTHORITY IMPLEMENTATION-READY; R7-0 ACTIVE AT ENTRY ONLY; ACTIVE PACKET NONE; R7-0.1 NEXT; NO R7 TASK STARTED** as of 2026-07-15. Promotion series `455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`; transition series `6bf0ac6ad` + `4a887ee0c` is landed with fresh transition review pending. Preserved review-clean R6 proof series remain authoritative. The 2026-07-15 exact replay controls pass `4 x 1 / 1`; family filters pass `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`; full analyzer completes with all suites green; diff check is green. Sticky `CTX-R6-06` current authority remains `HistoricalOnly / 20`, unflagged, while `Recovered / 20` remains historical baseline evidence only. `CTX-R6-17` assigns `dead_end_thrash` and `semantic_goal_drift` **Cutover complete** plus `truth_grounding_gap`, `wrong_plan_branch`, and `scoring/mod.rs` **Fit-for-purpose exception**. No R7 task has started. Closure authority is
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. This map began as a scaffold created on
2026-06-27 after `R5.75` closed. It is a **rescope** of the original
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

## Historical Preliminary Investigation Findings (Superseded)

> **Historical / superseded status:** this section records the 2026-06-27 pre-cutover investigation
> that framed Decision Gate 0. It is preserved for provenance, not as current scorer inventory or
> closure posture. In particular, its three-scorer dispatcher description predates
> `semantic_goal_drift`. Current behavior and proof authority live in the closure finding linked at
> the top of this map.

A focused read of the then-live scorer surface (`crates/agent-drift-analyzer/src/scoring/`) — enough
to de-risk and frame `Decision Gate 0`, not an exhaustive audit. The original planning session was
expected to finish it (see Task 1) before locking the decision.

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

Tasks 1-5 are **complete** as of 2026-07-01 — the consumer/failure-mode investigation and the
`Decision Gate 0` resolution landed in
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` (which also fixes the packet
decomposition: `R6-1`, `R6-2`, `R6-3`, conditional `R6-4`), the `R6-1` and `R6-2` SPEC/PLAN/TASKS sets are
written (see Packet Documents below), and `R6-1` closed review-clean with the full analyzer wall plus
touched sentinel spot-checks green (`cargo test -p agent-drift-analyzer -- --nocapture`,
`cargo test -p agent-drift-sentinel warning_policy -- --nocapture`,
`cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`). `R6-2` then closed with the full
analyzer wall plus the full sentinel wall green on 2026-07-01 (`cargo test -p agent-drift-analyzer -- --nocapture`,
`cargo test -p agent-drift-sentinel -- --nocapture`), and no `R6-1`/`R6-2` replay evidence showed
`progress.rs` reset errors caused by objective-string quality, so `R6-4` remains deferred to the later
full-migration phase. `R6-3` then closed on 2026-07-03 with the same full analyzer + sentinel walls green,
no `DriftClass` / `schema_version` / `checkpoint/export.rs` / sentinel `operator_surface.rs` source
changes across the committed `R6-3` range, and still no `R6-1`/`R6-2`/`R6-3` replay evidence showing a
`progress.rs` reset error caused by objective-string quality.

### Packet Documents

- `R6-1` (objective-independent scorer cutover): `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-{spec,plan,tasks}.md`
- `R6-2` (structured-objective consumer, kickoff-anchored first cut): `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md`
- `R6-3` (rolling / previous-checkpoint semantic drift): `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-{spec,plan,tasks}.md` — landed on 2026-07-03. The scorer now adds rolling / previous-checkpoint semantic drift as **tagged evidence on the existing `SemanticGoalDrift` class, not a new variant**, and the closeout proof confirmed no `schema_version` bump and no sentinel/`export.rs` lockstep source change.
- `R6-4` (conditional reset migration): not written — still deferred after the `R6-3` closeout because no `R6-1`/`R6-2`/`R6-3` replay evidence showed `progress.rs` reset errors caused by objective-string quality; remains evidence-gated per the DESIGN doc.

1. **[done] Complete the consumer/failure-mode investigation.** Finish the scorer-input audit (turn context,
   archetype inputs, interval/reset history carry, how `truth_artifacts` are sourced), and enumerate
   the concrete `R6` failure modes from the known replay artifacts. Output: the evidence that makes
   `Decision Gate 0` non-arbitrary.
2. **[done] Resolve `Decision Gate 0`.** Resolved to scoped Option C with full evidence and rationale in
   the `R6` DESIGN doc.
3. **[done] Lock `R6` scope + acceptance wall.** The `R6-1` and `R6-2` SPEC/PLAN/TASKS sets are written
   under `docs/specs/r6/R6-1/` and `docs/specs/r6/R6-2/` (Packet Documents above).
4. **[done] `dead_end_thrash` cutover (`R6-1`, gate-independent).** Landed per the `R6-1`
   SPEC/PLAN/TASKS: frontier-aware decisive-step scoring + the two objective-independent process
   dimensions, rule-based, with closeout confirmed by the green analyzer wall and touched sentinel
   spot-checks on 2026-06-30. `R6-1` is promoted history.
5. **[done] Objective-coupled first cut (`R6-2`) + closeout routing.** Landed per the `R6-2`
   SPEC/PLAN/TASKS as the kickoff-anchored `semantic drift from kickoff/plan/docs` dimension, wired as a
   structured-sidecar consumer. Closeout is confirmed by the green full analyzer wall plus the green full
   sentinel wall on 2026-07-01 (`cargo test -p agent-drift-analyzer -- --nocapture`,
   `cargo test -p agent-drift-sentinel -- --nocapture`). `R6-4` stays **deferred** because no `R6-1`/`R6-2`
   replay evidence showed `progress.rs` reset errors caused by objective-string quality. A follow-up codex
   second-opinion review (2026-07-01) found and fixed one real bug (the kickoff anchor was being applied to
   checkpoints ordinally before the one that established it) and one sentinel coverage hole; see the `R6-2`
   TASKS ledger "Post-Closeout Codex Review Fixes" section.
   **Known limitation, carried as debt, not fixed in `R6-2`:** the semantic-distance check is disjoint-set
   overlap over normalized structured-goal terms, not a graduated distance (SPEC Resolved Decision 7). It
   can over-flag legitimate narrowing (e.g. a target path narrowing from a crate root to one file inside it
   reads as fully disjoint and gets flagged as drift) and can miss real drift whenever the two goals still
   share any single `PlatformBoundary`/`ScopeBoundary` constraint term (which masks it) or the anchor has no
   concrete target and no boundary constraint (an empty anchor term set suppresses the claim). Anyone
   building `R6-3` or a future distance-metric pass should read that Resolved Decision before assuming the
   current check is graduated.
6. **[done] `R6-3` (rolling / previous-checkpoint semantic drift), with `R6-4` still conditional.**
   Landed per the `R6-3` SPEC/PLAN/TASKS on 2026-07-03. The analyzer now scores **rolling /
   previous-checkpoint** semantic drift (current checkpoint's structured goal vs the immediately-previous
   checkpoint's, reached through `analysis.previous` with no new plumbing) as tagged evidence on the
   existing `SemanticGoalDrift` class, complementing the kickoff-anchored cumulative signal from `R6-2`.
   Closeout is confirmed by the green full analyzer wall plus the green full sentinel wall on 2026-07-03
   (`cargo test -p agent-drift-analyzer -- --nocapture`, `cargo test -p agent-drift-sentinel -- --nocapture`),
   with explicit grep/diff proof that no new `DriftClass` variant, no `schema_version` bump, and no
   `checkpoint/export.rs` / sentinel `operator_surface.rs` source change landed across the committed `R6-3`
   range. `R6-4` stays **deferred** because no `R6-1`/`R6-2`/`R6-3` replay evidence showed a `progress.rs`
   reset error caused by objective-string quality. The disjoint-set semantic-distance limitation from item 5
   remains accepted debt; any future reset migration or distance revisit stays evidence-gated and deliberate.
   **Open validation debt (rolling):** the `R6-3.1` corpus check found `0 of 23` eligible adjacent
   `Medium+` `TaskStatement` pairs disjoint, so rolling drift never fired on the committed corpus — its
   positive firing rests entirely on the synthetic `synthetic-rolling-mid-session-pivot` fixture, and its
   real-corpus firing rate and over-fire rate are **unvalidated**. Measure both against real replay sessions
   with genuine mid-session pivots before trusting rolling operationally or building the graduated-distance
   revisit (`R6-3.X.2`) on top of it; real over-fire is the trigger to open that revisit.
   **Real-session probe (2026-07-03):** two live sessions run through the `agent-session-compactor` ->
   `agent-drift-analyzer` pipeline (`019e9864-…` exploratory/review; `019f2837-…` concrete-goal work) both
   produced `0` eligible checkpoints and `0` rolling evidence. The dominant gate is upstream of the distance
   metric: `eligible_current_goal` requires `unknowns.is_empty()`, and neither session cleared it —
   `019e9864` never resolved a `target` (`unknowns=[target,deliverables]`), and `019f2837` had a concrete
   high-confidence stable target on all 9 checkpoints but left `unknowns=[success_conditions,deliverables]`.
   So rolling's real-world non-firing is dominated by **objective-decomposition coverage**, not by the
   disjoint-set distance debt. A codex second-opinion (consult `019f2927`, 2026-07-03) sharpened the
   mechanism: empty `success_conditions` / `deliverables` do **not** disqualify on their own — they become
   `unknowns` only when the extractor saw those cues *off the goal surface* and rejected them (normal
   scaffolding like "Verify…" / "Return with…"), and neither field participates in divergence anyway. The
   load-bearing lever is therefore to **loosen the shared `eligible_current_goal` bar to a target-resolved
   gate** (grounded target present + confidence, not all-fields-resolved), not to chase `success_conditions`
   / `deliverables` extraction (the weaker lever, since those fields do not drive divergence). That bar is
   shared with `R6-2`'s kickoff-anchored path, so it is a deliberate cross-signal change whose payoff is
   gated by the disjoint-set over-fire risk — tracked as deferred task `R6-3.X.3`.
   **Diagnostic batch scan (2026-07-03, done):** 110 real sessions across 43 repos and 11 analyzable months
   (2025-09 → 2026-07; pre-`session_meta` rollouts cannot be analyzed), 882 checkpoints. Current-bar
   eligibility is `17.7%` (not inert), a target-resolved bar would reach `39.5%` (2.2×), and rolling's
   firing surface would grow from `1` real fire to ~`12` disjoint adjacent-pair candidates. But the decisive
   finding is that the firings are **false positives**: the current bar's only `6` flags are one session's
   garbage target extraction, and all `12` hypothetical target-only disjoint pairs are garbage/fragment
   targets or legitimate narrowing/progression — no genuine "goal A abandoned for unrelated goal B" pivot
   appeared. **Revised verdict:** do **not** loosen the bar in isolation; the strict `unknowns.is_empty()`
   gate currently suppresses over-fire, and loosening would multiply false positives from garbage extraction
   and the disjoint-set narrowing debt. Gate any loosening behind BOTH objective-extraction robustness
   (`context/objective.rs`) and the graduated-distance metric (`R6-3.X.2`). **Junk-filter gate re-count
   (2026-07-03):** an analysis-only junk-target filter over the same batch confirmed extraction is the
   dominant lever — `0/6` fires survive and the `12` disjoint pairs drop to `9`, none a real pivot. The
   batch + pipeline scripts are preserved at `scripts/dev/drift-batch-scan/`. Full data in the `R6-3` TASKS
   ledger, `R6-3.X.3` "Batch scan outcome," and `FINDINGS-r6-3-real-world-drift-validation.md` "Gate Result."
   **Next packet — `R6-3.5` Objective Target Hygiene (kickoff 2026-07-04):** the charter's Step 1
   extraction-hardening lands here — a shared deterministic target-anchor classifier
   (`TargetAnchorQuality{Stable,Weak,Junk}`) in `context/objective.rs` plus a scorer stable-term backstop and
   a bounded opaque-delegated-parent guardrail in `semantic_goal_drift.rs`. See
   `docs/specs/r6/R6-3.5/` (spec/plan/tasks) and the `FINDINGS` "R6-3.5 Result" section. **Landing order stays
   locked:** `R6-3.5` (extraction) → `R6-3.X.2` (graduated/weighted distance) → `R6-3.X.3` (eligibility-bar
   revisit, only if evidence supports it). Do not reorder; do not loosen the eligibility bar before the
   graduated-distance work lands.
   **`R6-3.X.2` containment first cut (landed 2026-07-05):** with `R6-3.5` closed, the first bounded cut of
   the distance work landed — divergence in `scoring/semantic_goal_drift.rs` now recognizes structural
   path/symbol containment as relatedness, so the canonical narrowing (crate/directory root → one file
   inside it, and `foo::bar` → `foo::bar::baz`) no longer flags on either the kickoff-anchored or rolling
   comparison. Containment is computed on the **raw structured-target strings** split only on real
   structural separators (`/`, `\`, `::`), never on the normalized term set — a codex review of the first
   draft caught that a normalized-prefix test would collapse `/`, `-`, and `.` to the same `_` boundary and
   silently drop a real pivot (`docs/specs/r6-map` vs `docs/specs/r6/map.md`); the raw-path split keeps
   `-`/`.` inside a segment so that pivot still fires. Sibling artifacts sharing a stem still fire, and all
   pinned true-positive fixtures are preserved (new acceptance case
   `synthetic-kickoff-narrowing-into-anchored-subtree` pins the suppression end-to-end; a scorer-level
   regression guard pins the hyphen/slash collision). Multi-anchor containment is symmetric, not any-pair
   (codex re-review §P2, two rounds: a multi-target goal that adds an unrelated target — narrowing or
   broadening — still fires), and segment comparison is case-sensitive (codex re-review §P3), with
   identity-preserving spellings canonicalized first (codex re-review round 4: a `./` current-dir prefix
   and a trailing `:line`/`:line:col` reference are no-ops and are stripped before comparison, so
   `exec.rs` → `exec.rs:1537` reads as the same file, not a pivot; the line strip runs on the leaf
   segment, not the whole string, so a Windows drive colon — `C:/repo/src/lib.rs:42` — does not defeat
   it [codex re-review round 5]). The graduated / weighted remainder of `R6-3.X.2` was subsequently
   closed through the bounded `R6-3.X.2B` / `2C` / `2D` chain; the eligibility-bar revisit
   (`R6-3.X.3`) remains deferred. **R6-3.6 validation follow-on (2026-07-05):** the
   acceptance corpus became an explicit 10-case wall with **`5/5` true-positive controls firing** and
   **`5/5` legitimate non-pivot controls staying quiet**, and the fresh seed-42 corpus rerun
   (`110` sessions / `44` repos / `938` checkpoints) again showed `0` fires under the shipping bar.
   **R6-3.X.2B / R6-3.X.2C / R6-3.X.2D closeout chain (2026-07-08):** `R6-3.X.2B` landed the relation taxonomy,
   but `R6-3.X.2C` was required to replace relation-only `claims_drift()` routing with an explicit
   internal `Suppress` / `Fire` / `NoClaim` contract. At the public `semantic_goal_drift` emission
   boundary today, only `Fire` emits while both `Suppress` and `NoClaim` stay non-fire. `R6-3.X.2D`
   committed the exact root-level README doc-bundle narrowing as a scorer-local and live-analyzer
   negative control, kept the unrelated root-doc addition as a positive control, pinned non-`src` role
   witnesses (`build.rs` / `examples/*.rs` as `Code`; `tests/*.rs` / `benches/*.rs` as `Verify`), and
   added continuity-threshold edge tests. Live truth after the fresh default seed-42 rerun from the
   current store: `110/110` sessions clean / `45` repos / `884` checkpoints, `146` current-bar eligible
   checkpoints, `245` target-resolved eligible checkpoints, `206` adjacent target-eligible pairs, `199`
   same-target suppressions, `7` changed-target candidates, `6` remaining disjoint pairs, and **`0`
   emitted `semantic_goal_drift` fires**. **Routing decision updated:** `R6-3.X.2D` is landed and closed;
   keep `R6-3.X.3` deferred and do not expand `semantic_goal_drift` without new evidence. The next R6
   planning pass should select a non-`semantic_goal_drift` scorer or closeout item. Sentinel, compactor,
   schema, delegation, and `RepoRelativeEquivalentAfterCwdStrip` work remain outside this closeout. See
   the `R6-3` TASKS ledger and the `FINDINGS` `R6-3.X.2D Closeout` result note.

## R6 Closure Reconciliation (2026-07-12)

The scoped `R6-1`, `R6-2`, `R6-3`, and `R6-3.X.2B` / `2C` / `2D` packets are landed. That packet-level
fact does **not** prove the broader scorer-context charter closed:

- `dead_end_thrash` consumes analyzer-owned `SessionProgress` and has focused proof for advancing
  frontier suppression, no-frontier stall, typed failure evidence, recovery, and bounded replay
  posture. The frozen four-case corpus proves posture invariance, not comparative integrated
  improvement, so broad replay honesty remains partially / bounded proven. Scorer-level proof is
  still missing for regression, opaque delegated-parent activity, and the long-autonomous versus
  many-short-conversational acceptance sentence.
- `semantic_goal_drift` is context-aware in the way appropriate to its responsibility: structured
  current/kickoff/previous goals, stable targets, sanctioned replans, delegation visibility, and
  checkpoint history. It is **cutover complete**; typed outcomes, archetype, and progress are not
  semantically required, and it must not be reopened absent new failing evidence.
- `truth_grounding_gap` and `wrong_plan_branch` have reasonable fit-for-purpose input boundaries,
  but their context-exception claims lack the behavioral controls required for closure. Truth
  grounding still needs truth-path-touching action-before-read and actionful-planning/research
  controls; wrong-branch scoring still needs an empty-authority path-bearing action control.
- `scoring/mod.rs` has an explicit deterministic order in source. That exact order is source-proven,
  not behavior-tested, and needs a focused assertion only if it remains a closure contract.
- `R6-3.X.3` and conditional `R6-4` remain separately evidence-gated; this audit does not
  manufacture their triggers.

As of the 2026-07-12 audit, R6 was therefore **PARTIAL / CLOSURE AUDIT REQUIRED**, not closed for
sequencing, and the next authorized work was the acceptance-first `R6-C.1` packet. That statement is
historical; the current disposition is recorded below. At that historical boundary, R7 design
documents remained preserved as **DRAFT / BLOCKED ON R6 CLOSURE DECISION** until the finding was
updated to `CLOSED`, every material scoring surface had one of the four terminal dispositions
(**Cutover complete**, **Fit-for-purpose exception**, **Merged/deprecated**, or **Explicitly deferred
outside R6 with justification**), and the authority stack agreed. The later `CTX-R6-17` receipt
satisfied that entry gate; it did not itself promote the R7 drafts.

## R6-C.1 Control Disposition Update (2026-07-13)

`R6-C.1-CONTROLS` is **COMPLETE** against the `5618f7864` controls-wall receipt. The thirteen
synthetic controls resolved as `10 PASS / 3 preserved RED`, checkpoints passed, and no production
code changed. In matrix order the preserved routes are:

1. `R6-GAP-DET-OPAQUE-PARENT` for `CTX-R6-04`, witness `87409b39a` — **COMPLETE** after production
   series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN`;
2. `R6-GAP-TGG-TRUTH-PATH-ACTION` for `CTX-R6-12` — **COMPLETE** after its final proof-receipt
   series `fee9c2b16` + `6674a8316` received fresh independent built-in `default` `REVIEW CLEAN`;
   and
3. `R6-GAP-WPB-EMPTY-AUTHORITY` for `CTX-R6-15`, witness `59f098b35` — **COMPLETE** after
   implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh
   independent built-in `default` `REVIEW CLEAN`; exact target is `0 / Low / Cleared`, unflagged,
   with empty evidence, and its protected/family/checkpoint/full-analyzer/static walls are green.

R6 is **CLOSED**. `R6-REPLAY`, `R6-CLOSE`, and `CTX-R6-17` are complete with active packet `none`.
Historical `CTX-R6-02` witness `60cde3dd7` is preserved;
review-clean commit
`6eda87e60` completes `CTX-R6-02` and `R6-GAP-DET-REPLAY-STALL` with exact true-stall proof, the
complete ordered packet wall, full analyzer `402 / 402`, and static gates. Packet transition series
`1ff592823` + `7839a7f47` is fresh independent built-in `default` `REVIEW CLEAN`. Phase-owned exact
replay controls pass `4 x 1 / 1`; the R6 family wall passes `21 / 21`, `58 / 58`, `22 / 22`,
`6 / 6`, `169 / 169`, full analyzer `402 / 402`, and diff check. Current sticky authority is
`HistoricalOnly / 20`, unflagged, while `Recovered / 20` is historical baseline only. Proof/fix
series `b1791c1e3` + `e6d43eee9` + `61c9d5074` received fresh independent built-in `default`
`REVIEW CLEAN`; no ordinary replay gap remains. The 2026-07-15 `CTX-R6-17` receipt reconfirms the
exact replay controls, the five Manifest E family filters, the full analyzer all-suites wall, and
`git diff --check`, then completes the terminal table. Promotion series `455d0ed90` + `876ac55de`
received fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-PROMOTE` and making
the R7 authority family implementation-ready. Transition series `6bf0ac6ad` + `4a887ee0c` is
landed with fresh transition review pending; `R7-0` is active at entry only with packet `none`,
`R7-0.1` is next, and no R7 task has started.

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
  given a sidecar-presence guard? → yes in principle, but deferred to the conditional `R6-4`, evidence-gated.
- Minimum acceptance coverage to keep the legacy surface honest under Guardrail 5? → `R6-1` exercises the
  `R5.75-6` bridge needle lists directly so a missed phrasing fails at the analyzer wall.
