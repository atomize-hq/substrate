# R6-3.X.2B TASKS — Graduated / Weighted Semantic Distance For Semantic Goal Drift

Status: OPEN (created 2026-07-06 from the next-packet planning request plus live `R6` repo truth,
tightened after review findings). This packet owns the **remaining** weighted-distance remainder inside the
analyzer scorer. It does not reopen extraction hardening, the containment first cut, or `R6-3.X.3`
eligibility loosening.

## R6-3.X.2B.0: Docs lock and live-truth framing

- [x] Task R6-3.X.2B.0.1: Lock the packet docs against live repo truth.
  - Acceptance:
    - `docs/specs/r6/R6-3.X.2B/` contains packet-local spec, plan, and tasks docs.
    - The docs state explicitly that `R6-3.5` and the containment first cut are already landed.
    - The docs state explicitly that the next blocker is weighted relation grading, not eligibility loosening.
    - The docs state explicitly that the relation taxonomy, not any numeric score band, is the routing authority.
  - Verify:
    - manual audit against `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - manual audit against `docs/specs/r6/MAP.md`
    - manual audit against `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Dependencies: none
  - Files likely touched:
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-spec.md`
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-plan.md`
    - `docs/specs/r6/R6-3.X.2B/agent-drift-analyzer-semantic-goal-drift-graduated-weighted-distance-tasks.md`
  - Estimated scope: S
  - Result (2026-07-06): packet-local SPEC/PLAN/TASKS now explicitly lock the live repo truth that
    `R6-3.5` target hygiene and the `R6-3.X.2` containment first cut are already landed, that the
    remaining blocker is weighted relation grading rather than eligibility loosening, and that the
    relation taxonomy — not a numeric score band — is the routing authority. `MAP.md` and the
    parent `R6-3` ledger now route the remainder through `R6-3.X.2B` with the same constraints.

- [x] Task R6-3.X.2B.0.2: Record the implementation boundary and deferred surfaces.
  - Acceptance:
    - The docs lock the analyzer-local boundary: scorer, analyzer tests/fixtures, batch tooling, findings/map/ledger.
    - The docs explicitly fence off `crates/agent-drift-analyzer/src/context/objective.rs`, sentinel, compactor,
      `TaskFrame`, `working_set`, `progress`, and export/schema seams.
    - The docs define the default `R6-3.X.3` defer rule.
    - The docs mark repo-relative cwd-strip equivalence as ask-first/deferred unless current scorer-seam
      reachability is proven from live repo truth.
  - Verify: manual review of the packet spec boundaries and non-goals.
  - Dependencies: `R6-3.X.2B.0.1`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS
  - Result (2026-07-06): the packet docs now fence the work to analyzer-local scorer logic,
    analyzer tests/fixtures, batch tooling, and findings/map/ledger updates; they explicitly keep
    `context/objective.rs`, checkpoint export/schema/progress/working-set seams, sentinel,
    compactor, and full delegated-subagent semantics out of scope unless reopened. Repo-relative
    cwd-strip equivalence is recorded as conditional/deferred unless the scorer seam proves it can
    already read the needed cwd/session-root truth analyzer-locally.

- [x] Task R6-3.X.2B.0.3: Lock the shared-helper preflight requirements.
  - Acceptance:
    - The docs require GitNexus impact analysis before editing shared scorer helpers such as
      `eligible_current_goal`, `goal_specific_terms`, and any new relation helper shared by kickoff and rolling.
    - The docs require `gitnexus detect-changes` (or repo-qualified equivalent) before commit.
    - The docs pin the prior-witness non-regression wall (`R6-1`, `R6-2`, `R6-3.X.2`, `R6-3.6`, carried `R5.75` witnesses).
  - Verify: manual review of packet-local spec/plan/tasks docs.
  - Dependencies: `R6-3.X.2B.0.2`
  - Files likely touched:
    - packet-local spec/plan/tasks docs
  - Estimated scope: XS
  - Result (2026-07-06): the packet docs now require GitNexus impact analysis before editing shared
    scorer helpers, require `gitnexus detect-changes` before commit, and lock the prior-witness
    non-regression wall (`R6-1`, `R6-2`, `R6-3.X.2`, `R6-3.6`, carried `R5.75`). Live preflight was
    run before scorer work: the stale index was refreshed with `npx gitnexus analyze --name
    97a0-substrate`, then upstream impact was recorded for `eligible_current_goal` (LOW),
    `goal_specific_terms` (MEDIUM), `goal_sets_diverged` (LOW), and `score_semantic_goal_drift`
    (HIGH within the scoring/test surface), so the scorer-edit slice remains sign-off gated.

## R6-3.X.2B.1: Stable-anchor relation model

- [ ] Task R6-3.X.2B.1.1: Define the stable-anchor comparison record and ordered relation taxonomy.
  - Acceptance:
    - The scorer design names the relation taxonomy: exact, structural containment, conditional repo-relative
      equivalence after cwd stripping, same artifact family, same work-item family, same doc family,
      plan-doc ↔ code role shift, review/findings ↔ fix/verify role shift, shared-constraint-only,
      weak/generic-only, unrelated, unknown.
    - The taxonomy states which relations may suppress and which may not.
    - The taxonomy states explicitly that role-shift relations are **pairwise** only and do not require any hidden
      temporal state/history beyond current/kickoff/previous.
    - The design preserves multi-anchor symmetry.
  - Verify:
    - scorer-local unit tests for relation precedence
    - spec/plan/tasks docs reference the same taxonomy names without drift
  - Dependencies: `R6-3.X.2B.0.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - packet-local docs
  - Estimated scope: M

- [ ] Task R6-3.X.2B.1.2: Tighten suppressive-family semantics so weak overlap cannot suppress pivots.
  - Acceptance:
    - `SameArtifactFamily` requires decisive same-lineage evidence and does **not** suppress on sibling-stem overlap,
      same extension alone, or other weak family residue.
    - `SameWorkItemFamily` requires the same numbered/dotted lineage token and does **not** suppress on generic
      `spec/plan/tasks` vocabulary alone.
    - `WeakOrGenericOnly` explicitly includes generic `spec/plan/tasks` overlap and remains non-suppressive.
    - The docs and tests preserve multi-anchor symmetry and prevent one related pair from masking an unrelated one.
  - Verify:
    - scorer-local regression tests for sibling-stem and generic-spec/plan/tasks false-negative guards
    - packet docs reflect the tightened wording without drift
  - Dependencies: `R6-3.X.2B.1.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - fixture directories as needed
  - Estimated scope: M

- [ ] Task R6-3.X.2B.1.3: Handle repo-relative normalization only if seam reachability is proven.
  - Acceptance:
    - If live implementation proof shows the scorer can already read the needed cwd/session-root truth analyzer-locally,
      absolute-vs-repo-relative spellings of the same subtree can be classified as related after safe cwd stripping.
    - If that proof is absent, the relation is recorded as deferred / ask-first instead of guessed into scope.
    - The existing raw containment helper remains unchanged except for calling the later-stage relation path when
      the seam proof exists.
    - Case-sensitive symbol/path protections remain intact.
  - Verify:
    - scorer-local regression tests only if the seam proof exists
    - otherwise docs explicitly record the deferral
  - Dependencies: `R6-3.X.2B.1.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (conditional)
    - packet-local docs
  - Estimated scope: S

## R6-3.X.2B.2: Weighted assessment and drift routing

- [ ] Task R6-3.X.2B.2.1: Add the weighted relation assessment shape.
  - Acceptance:
    - The scorer has an analyzer-local assessment object with relation, score, confidence, decisive evidence,
      and counter-evidence.
    - The code/comments/docs make explicit that relation + confidence drive routing; the numeric score is
      explanatory only.
    - Kickoff and rolling comparisons use the same assessment surface after exact/containment checks.
  - Verify:
    - scorer-local tests for relation-authoritative routing
    - focused semantic-goal-drift test target
  - Dependencies: `R6-3.X.2B.1.3`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.2.2: Route shared-constraint-only, weak/generic-only, and ambiguous cases correctly.
  - Acceptance:
    - shared-constraint-only overlap no longer suppresses an unrelated pivot.
    - weak/generic-only overlap no longer suppresses an unrelated pivot.
    - ambiguous/unknown middle-band cases prefer conservative no-claim over speculative suppression.
    - family-style suppressions have explicit counter-evidence paths so unmatched anchors can still escalate to
      `Unrelated` or `Unknown`.
  - Verify:
    - scorer-local tests covering shared-constraint and weak/generic-only paths
    - acceptance-level positive control for shared-constraint false-negative guard
  - Dependencies: `R6-3.X.2B.2.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M

## R6-3.X.2B.3: Proof wall expansion

- [ ] Task R6-3.X.2B.3.1: Add the required positive controls.
  - Acceptance:
    - the acceptance corpus includes at least the following new true-positive families:
      - crate/package pivot
      - workspace-ref pivot
      - verification-target pivot
      - repo/work-item pivot
      - shared-constraint false-negative guard
      - sibling-stem false-negative guard
      - generic `spec/plan/tasks` false-negative guard
    - every new positive control clears the current eligibility bar and uses stable target anchors.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Dependencies: `R6-3.X.2B.2.2`
  - Files likely touched:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.3.2: Add the required negative controls.
  - Acceptance:
    - the acceptance corpus includes at least the following legitimate non-pivot families:
      - same artifact family progression
      - same work-item family progression
      - plan-doc ↔ code role shift within one workstream
      - review/findings ↔ fix/verify role shift within one workstream
      - absolute-vs-repo-relative subtree only if the seam proof exists; otherwise the deferral is explicit
    - existing containment negatives remain intact.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - focused scorer-local regressions for each suppressive relation family
  - Dependencies: `R6-3.X.2B.3.1`
  - Files likely touched:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - Estimated scope: M

- [ ] Task R6-3.X.2B.3.3: Pair every new suppressive relation with at least one false-negative guard.
  - Acceptance:
    - same-artifact-family, same-work-item-family, same-doc-family, plan-doc ↔ code role shift, and
      review/findings ↔ fix/verify role shift each have at least one counter-example proving an unrelated pivot
      still fires.
    - the packet docs call out any residual over-fire or undecidable relation honestly instead of silently widening.
    - weak/generic overlap is never promoted into a suppressive family just to make a test pass.
  - Verify:
    - scorer-local tests
    - acceptance corpus assertions
  - Dependencies: `R6-3.X.2B.3.2`
  - Files likely touched:
    - scorer tests and acceptance fixtures
    - packet docs if residual limits remain
  - Estimated scope: M

- [ ] Task R6-3.X.2B.3.4: Preserve prior witnesses without rebaseline.
  - Acceptance:
    - the existing `R6-3.6` 10-case acceptance allowlist remains bounded and intact.
    - the existing `R6-1` dead-end-thrash posture / acceptance witnesses remain intact.
    - the existing `R6-2` kickoff-anchor witnesses remain intact.
    - the existing `R6-3.X.2` containment negatives remain intact.
    - the carried `R5.75-3` delegated-stability / `R5.75-4` zero-verifier anti-flap surfaces remain intact.
    - any witness change is treated as regression, not as an allowed rebaseline.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Dependencies: `R6-3.X.2B.3.3`
  - Files likely touched:
    - acceptance harness/fixtures
    - scorer-local regressions as needed
    - packet docs if a limit must be called out
  - Estimated scope: M

## R6-3.X.2B.4: Corpus tooling, rerun, and findings update

- [x] Task R6-3.X.2B.4.1: Add validation strata reporting when inferable.
  - Acceptance:
    - batch tooling reports delegation topology (mandatory) plus any additional best-effort splits for
      language/repo type, workflow type, and tooling type that can be supported honestly.
    - any stratum that remains `100% unknown` is recorded as unresolved/blocked and does **not** count as satisfying
      the validation gate.
    - the docs name the heuristic source for each reported non-unknown stratum.
  - Verify:
    - `python3 scripts/dev/drift-batch-scan/tabulate.py --help`
    - sample or real run output inspection
  - Dependencies: `R6-3.X.2B.3.4`
  - Files likely touched:
    - `scripts/dev/drift-batch-scan/README.md`
    - `scripts/dev/drift-batch-scan/tabulate.py`
    - `scripts/dev/drift-batch-scan/run_batch.py` (only if the smallest tag addition is required)
    - `scripts/dev/drift-batch-scan/inspect_targets.py` (only if residue labeling needs it)
  - Estimated scope: S
  - Result (2026-07-06):
    - `scripts/dev/drift-batch-scan/tabulate.py` now reports three additional best-effort checkpoint-level
      strata alongside the existing mandatory delegation split: `language / repo type`
      (`rust` / `js_ts` / `python` / `docs_only` / `mixed` / `unknown`), `workflow type`
      (`implementation` / `docs_planning` / `verification` / `review_fix` / `mixed` / `unknown`), and
      `tooling type` (`cargo_rust` / `node_npm` / `python_pytest` / `generic_filesystem_doc` /
      `unknown`).
    - Each new table prints its heuristic source explicitly from current export only: language/repo uses
      `task_frame.working_set_paths` plus `structured_objective.target` values with command/tool and file-
      extension hints; workflow uses `session_archetype.label` plus `session_progress.dimension` with
      `structured_objective.primary_intent` / verification-command fallback; tooling uses
      `task_frame.command_families`, `task_frame.tools`, and `task_frame.verification_commands` with a file-
      extension fallback when command evidence is absent.
    - `tabulate.py` now fail-closes the gate when an added stratum resolves to `100% unknown`, printing an
      explicit unresolved warning instead of silently counting that table as satisfied.
    - `scripts/dev/drift-batch-scan/README.md` documents the added strata, their buckets, and the
      reporting-only / best-effort contract.
    - Narrow TDD proof was added in `scripts/dev/drift-batch-scan/test_tabulate.py`, covering Rust,
      docs-only, mixed, workflow, and Python/tooling inference before the script changes landed.
    - Verification:
      - `python3 -m unittest scripts/dev/drift-batch-scan/test_tabulate.py`
      - `python3 scripts/dev/drift-batch-scan/tabulate.py --help`
      - synthetic output inspection via a 3-checkpoint temp corpus exercising the new language/workflow/tooling
        tables and the existing funnel output

- [x] Task R6-3.X.2B.4.2: Run the focused wall, full analyzer wall, and 110-session corpus rerun.
  - Acceptance:
    - focused semantic-goal-drift tests pass.
    - full analyzer wall passes.
    - the rerun is seed-pinned with `--seed 42`, or the docs explain why the exact prior sample cannot be reused.
    - the rerun is baseline-compared against the published `R6-3.6` seed-42 counts.
    - the remaining disjoint residue is labeled by relation family.
    - the validation-strata output is not presented as complete if it is all-unknown outside delegation.
  - Verify:
    - the full verification wall from the packet spec
    - the full batch command sequence from the packet spec
  - Dependencies: `R6-3.X.2B.4.1`
  - Files likely touched:
    - docs only unless a tiny tooling/report fix is required during rerun
  - Estimated scope: M
  - Result (2026-07-06):
    - The packet reran the committed seed-42 batch harness on a fresh manifest from the current
      `~/.codex/sessions` store: `110` sessions / `44` repos / `892` checkpoints, with `110/110`
      sessions analyzed clean. This is **directionally** baseline-compared against the published
      `R6-3.6` rerun (`938` checkpoints) rather than claiming the exact prior manifest was reusable.
    - The first rerun exposed a narrow scorer-local false positive family still inside packet scope:
      three current-bar fires in one docs session were just **doc-bundle → anchored member-doc narrowing**
      (`architecture-overview.md|README.md|…` → `README.md`). A failing scorer regression
      (`semantic_goal_drift_suppresses_doc_bundle_member_narrowing`) was added first, then the scorer gained a
      bounded `SameDocFamily` exact-member bundle suppression so that bundle-member narrowing/broadening no
      longer claims drift.
    - After that narrow fix, the **same seed-42 manifest** reran cleanly: `0` flagged checkpoints, `0`
      rolling evidence lines, `0` kickoff-anchor evidence lines, `137` current-bar eligible checkpoints,
      `240` target-resolved eligible checkpoints, `201` adjacent target-eligible pairs, `194` same-target
      exact-match suppressions, `7` changed-target candidates, and `6` remaining disjoint pairs.
    - Remaining disjoint residue was hand-labeled as non-pivot relation families:
      1. docs-survey / docs-root progression (`docs/ideas/...` → `docs/README.md` family)
      2. planning-doc progression (`status.md,risks.md` → `sprint-planning.md`)
      3. generic `spec/plan/tasks` bundle collapse (`...spec.md|...plan.md|...tasks.md` → `spec/plan`)
      4. artifact-family narrowing (`audit-trio.report.json` → `audit-trio.model-selection/...report.json`)
      5. sibling docs under one workstream directory (`...handoff-boundary.md` → `...threading.md`)
      6. plan → code → plan cycle residue (`async_repl.rs:497` → `llm-last-mile/PLAN-04.md`)
    - Validation strata are now reported without any `100% unknown` table outside the mandatory delegation
      split: delegation stays explicit (`single_agent` = `760` cp / `186` target-resolved / `0` fires /
      `2` disjoint; `delegated_child_visible` = `132` cp / `54` target-resolved / `0` fires / `4`
      disjoint), while best-effort language/workflow/tooling tables are populated from the exported heuristic
      sources documented in `scripts/dev/drift-batch-scan/README.md` and printed by `tabulate.py`.
    - Verification:
      - `cargo test -p agent-drift-analyzer semantic_goal_drift_suppresses_doc_bundle_member_narrowing -- --nocapture`
      - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
      - `cargo test -p agent-drift-analyzer -- --nocapture`
      - `cargo clippy --workspace --all-targets -- -D warnings`
      - `cargo fmt --all -- --check`
      - `python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2b_selected.jsonl --batch-dir /tmp/r6_3_x_2b_batch_after_fix`
      - `python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2b_batch_after_fix/checkpoints`
      - `python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2b_batch_after_fix/checkpoints`

- [ ] Task R6-3.X.2B.4.3: Update findings, map, and routing decision.
  - Acceptance:
    - `FINDINGS`, `MAP`, and the `R6-3` task ledger record:
      - relation families landed
      - relation-authoritative routing (score explanatory only)
      - pairwise role-shift semantics
      - repo-relative-cwd-strip proof or explicit deferral
      - positive/negative control results
      - prior-witness non-regression status
      - rerun totals, baseline comparison, and strata
      - remaining residue
      - final yes/no decision on whether `R6-3.X.3` stays deferred
    - default expected decision remains deferred unless the rerun proves otherwise.
  - Verify: manual audit against the final test and rerun outputs.
  - Dependencies: `R6-3.X.2B.4.2`
  - Files likely touched:
    - `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
  - Estimated scope: S

## Final checkpoint

Before implementation is considered packet-complete:
- [ ] the relation taxonomy is explicit and relation-authoritative
- [ ] any numeric score is marked explanatory only
- [ ] pairwise role-shift relations are explicit and do not rely on hidden temporal history
- [ ] repo-relative cwd-strip equivalence is either proven reachable or explicitly deferred
- [ ] shared-constraint-only no longer masks real pivots
- [ ] weak/generic overlap no longer masks sibling or generic-spec/plan/tasks pivots
- [ ] new progression families are absorbed without widening containment
- [ ] every suppressive family has a false-negative guard
- [ ] prior witnesses stay locked with no rebaseline
- [ ] the 110-session rerun is seed-pinned or baseline-compared honestly
- [ ] validation strata are not closed out via all-unknown reporting
- [ ] closeout docs make an explicit `R6-3.X.3` defer/reopen decision
