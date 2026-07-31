# Tasks: Agent Drift Analyzer Structured Objective Phase 1

Status: draft task ledger created on 2026-06-14 from the structured-objective design stack;
reconciled on 2026-06-17 against the live crate snapshot plus the grounding follow-on family
through `SO-G6`, then updated on 2026-06-18 after `SO-2.3B-refine` closeout and reconciled again
after the landed `SO-3` through `SO-6.1` packets so this ledger reflects the live phase-1 closeout
plus explicit post-phase-1 debt. Architecture owns semantics, evaluation owns acceptance, migration
owns landing order, and the classifier taxonomy remains deferred for this phase. Keep the original
packet structure for auditability, but do not treat already-landed SO-1 / SO-6 seams as still-open
implementation debt.

Keep each task as close as possible to five touched files or fewer. Do not advance to the next task
until the current task's verification commands are green or the failure is explicitly captured in
packet notes.

Packet prerequisite rule: when a phase-1 packet prompt names earlier packets as already landed,
verify those prerequisite tasks in live code/tests before editing. If a prerequisite is missing,
stop and report it instead of letting the later packet absorb the earlier bug.

## SO-0: Docs Lock

- [x] Task SO-0.1: Add the phase-1 SPEC.
  - Acceptance: `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md` exists
    and covers assumptions, objective, commands, project structure, code style, testing strategy,
    boundaries, success criteria, and open questions.
  - Verify: Manual review against the five structured-objective design docs and `AGENTS.md`.
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md`

- [x] Task SO-0.2: Add the phase-1 PLAN and TASKS.
  - Acceptance: the plan and task ledger exist, preserve the authority precedence, and keep Phase 1
    bounded to additive sidecar + acceptance work.
  - Verify: Manual review against the architecture, evaluation, and migration authorities.
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

## SO-1: Schema Bridge And Additive Sidecar

Historical note: this packet's schema bridge and sidecar exposure are already landed in the live
crate snapshot. The open work now starts later in the ledger; keep these tasks for auditability
rather than deleting the original packet structure.

- [x] Task SO-1.1: Define the phase-1 structured-objective DTOs and evidence-span enums.
  - Acceptance: shared schema types exist for `StructuredObjective`, field-level supporting enums,
    `ObjectiveEvidenceSpan`, and `ObjectiveUnknown`, and they match the architecture authority's
    minimum semantic coverage.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/context/objective.rs`

- [x] Task SO-1.2: Add `comparison_key` and optional structured sidecar to `ObjectiveSummary`.
  - Acceptance: `ObjectiveSummary` now exposes `text`, `comparison_key`,
    `structured: Option<StructuredObjective>`, `verification_commands`, and `evidence`, while all
    current callers still compile and no consumer assumes the sidecar is always present. This
    acceptance is about additive exposure only; the later semantic `comparison_key` derivation that
    was originally deferred to SO-3.2 is now landed, so this task should be read as the bridge-
    creation packet rather than the final semantic-key packet.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/src/context/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## SO-2: Deterministic Decomposition And Structured Assembly

Historical note: this packet family is already materially landed in the live crate snapshot. The
follow-on grounding family (`SO-G1` / `SO-G2`) hardened section/clause identifiers and adversarial
heading coverage without erasing the fact that the original decomposition/assembly seam already
landed here. `SO-2.3B-refine` is the later corrective continuation of this already-landed
preliminary structured assembly and is now itself landed as the bridge-honesty refinement; do not
read `SO-2.1` / `SO-2.2` / `SO-2.3` below as untouched greenfield backlog or as a competing
implementation plan.

- [x] Task SO-2.1: Add section-aware decomposition for long directive rows.
  - Acceptance: objective extraction can distinguish mission/scope, checklist, verification,
    constraints, deliverables, context, boilerplate, and tooling-instruction sections well enough
    to stop whole-row flattening from hiding the real mission.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task SO-2.2: Add clause-role labeling and evidence-span grounding.
  - Acceptance: clause/sentence units are labeled with `goal`, `constraint`, `verification`,
    `context`, or `other_role`, and each nontrivial structured field records supporting evidence
    spans with source kind and section kind.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task SO-2.3: Assemble phase-1 structured fields and preserve ambiguity as unknowns.
  - Acceptance: phase-1 extraction populates objective class, primary intent, target, constraints,
    success conditions, deliverables, verification commands, confidence, evidence spans, and
    unknowns only when evidence supports them; weak evidence remains unknown.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## SO-3: Compatibility Rendering And Deterministic Comparison Key

Historical note: this section is now landed in the live crate snapshot. `ObjectiveSummary.text`
renders from structured state when safe, and `comparison_key` is now derived from structured
semantic state rather than mirroring display text. Keep the packet entries below for auditability,
but do not treat `SO-3.1` / `SO-3.2` as still-open backlog.

- [x] Task SO-3.1: Render compatibility text from structured state when safe.
  - Acceptance: `ObjectiveSummary.text` becomes a compatibility view over structured state when the
    evidence is sufficient, but falls back conservatively when the sidecar is absent or key fields
    remain unknown.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task SO-3.2: Derive deterministic `comparison_key` from the structured frame.
  - Acceptance: `comparison_key` is derived from structured semantic state rather than raw pretty
    text, and focused regressions prove checklist or boilerplate wording changes do not silently
    become the new truth source.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## SO-4: Objective Acceptance Harness

Historical note: this harness is now landed in the live crate snapshot. The
`objective_acceptance` test file, fixture loader/support, and expected-shape contract are committed
and part of the green `SO-6.1` validation wall. Keep the packet entries below for auditability,
but do not treat `SO-4.1` / `SO-4.2` as still-open backlog.

- [x] Task SO-4.1: Add the objective-acceptance test harness and fixture loader.
  - Acceptance: `tests/objective_acceptance.rs` exists, can load committed objective-acceptance
    fixtures deterministically, and asserts the fixture directory contract for the three evaluation
    families. This packet starts only after the already-landed `SO-2.3B-refine` plus `SO-3.1` / `SO-3.2`.
  - Verify: `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`

- [x] Task SO-4.2: Encode the objective-acceptance expected-shape contract.
  - Acceptance: the harness checks structured-field expectations, role spans, grounding refs,
    forbidden promotions, compatibility rendering, and unknown-field correctness from committed
    fixture metadata rather than relying on one exact objective string, and it defines the concrete
    acceptance wall that later SO-3 and SO-5 work must satisfy.
  - Verify: `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`

## SO-5: Seed The Locked Acceptance Wall

Historical note: this packet family is now landed in the live crate snapshot. The locked
acceptance corpus includes the WDAP kickoff seeds, preserved instruction-surface controls, concise
`/goal` controls, and the review/no-code plus planning/docs controls needed for the phase-1
acceptance wall. Keep the packet entries below for auditability, but do not treat `SO-5.*` as
still-open backlog.

- [x] Task SO-5.1: Add WDAP linux and macOS locked-acceptance seeds.
  - Acceptance: the committed objective-acceptance corpus includes the required WDAP kickoff seeds,
    and their expected metadata explicitly forbids subordinate checklist lines from being promoted
    to `goal`.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`

- [x] Task SO-5.2: Add preserved boilerplate-target controls.
  - Acceptance: the locked acceptance set contains deliberate `AGENTS.md` / `<skill>` /
    instruction-surface cases where the real task is to analyze or edit that surface, and the
    structured extractor preserves that target instead of filtering it away.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`

- [x] Task SO-5.3: Add concise-goal, review/no-code, and planning/docs controls.
  - Acceptance: the committed corpus includes concise `/goal` controls, review/no-code semantics,
    and planning/research/docs prompts so Phase 1 proves it can distinguish implementation from
    non-implementation intent without overfitting to WDAP alone.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`

## SO-6: Full Phase-1 Validation And Honest Closeout

Historical note: the analyzer-local validation wall is now green in the live repo snapshot, and
the remaining non-phase-1 seams are explicit follow-on debt rather than implied blockers. Keep the
packet entries below for auditability, but do not treat `SO-6.1` / `SO-6.2` as still-open
implementation backlog.

- [x] Task SO-6.1: Run the full phase-1 validation wall.
  - Acceptance: formatting, clippy, focused checkpoint regressions, objective acceptance, and the
    full analyzer suite are all green on the landed sidecar implementation.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no source changes required unless validation exposes a packet-scoped defect

- [x] Task SO-6.2: Capture the deferred follow-on seam list explicitly.
  - Acceptance: any remaining TaskFrame coexistence, working-set migration, checkpoint predicate
    migration, progress comparability migration, or classifier work is recorded as explicit follow-on
    debt rather than left implicit in comments or half-wired code.
  - Verify: Manual review of the landed phase-1 docs and code diff.
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

## Post-SO-6 R5.75-1 Follow-On (tracked in `docs/specs/r5/R5_75/MAP.md`)

`SO-1` through `SO-6` are landed, and the two additional `R5.75-1` packets below are now landed too.
`R5.75-1` closes once the named promotion smoke is re-run; the MAP routing note and `R5.75-1` gate are
the live authority. The remaining open R5.75 work is `R5.75-2` onward.

- [x] Task SO-Observability: Make the structured objective observable in analyzer output and rewrite
      the `R5.75-1` gate to assert structured semantics.
  - Acceptance: each exported `Checkpoint` carries an additive optional `structured_objective`
    (`checkpoint/schema.rs`, populated in `checkpoint/mod.rs`), `summary.md` renders a per-checkpoint
    `objective:` line (`checkpoint/export.rs`), the `v0.6` schema is **not** bumped and no consumer is
    migrated, and the `R5.75-1` gate smoke asserts intent/target/success/deliverable/unknown semantics.
  - Note: this exports a Checkpoint-level projection only. It is **distinct from** `SO-X.1` below,
    which migrates `TaskFrame` itself (`objective_key` / `structured_objective`) plus its consumers
    and stays deferred.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`;
    `cargo test -p agent-drift-sentinel warning_policy live_end_to_end -- --nocapture`.
- [x] Task SO-2.3D: Semantic-honesty fix for `#4` (intent classification) and `#6`
      (success/deliverable pooling + asymmetric unknowns).
  - Acceptance: review prompts no longer classify as `Implement` on the noun "implementation"
    (`intent_for_text` derives intent from the request action via whole-word matching; this also
    corrected the WDAP fixtures `plan`→`validate` to match the evaluation authority);
    `success_conditions`/`deliverables`/`constraints` are scoped to the active goal surface
    (`clause_is_on_active_goal_surface`) with symmetric `unknowns` when off-surface cues are rejected;
    the two regressions are live (no longer `#[ignore]`d); and the `review-implementation-noun-review-intent`
    + `orchestration-scaffolding-field-honesty` locked acceptance cases were added.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture` (including the now-live regressions).

## Deferred / Ask-First (Not Phase 1)

- [ ] Task SO-X.1: Add `TaskFrame.objective_key` and `structured_objective` only after the phase-1
      sidecar and objective-acceptance wall are green.
  - Acceptance: a later approved follow-on explicitly promotes migration Phase 2, and the work
    stays additive instead of silently widening this phase. The live repo still carries only the
    legacy `TaskFrame.objective` string, so this coexistence bridge remains explicit debt rather
    than hidden future work.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/inference/mod.rs`

- [ ] Task SO-X.2: Migrate `context/working_set.rs` path attribution only after TaskFrame
      coexistence lands.
  - Acceptance: a later approved follow-on replaces `objective.text.contains(&path)` with
    structured target/evidence-driven attribution, preserves sidecar-presence guards, and does not
    re-create raw-string truth under a new name.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/context/working_set.rs`

- [ ] Task SO-X.3: Migrate `checkpoint/mod.rs` closeout/review/no-code predicates only after
      TaskFrame coexistence lands.
  - Acceptance: a later approved follow-on ports the string-level objective predicates to typed
    structured fields with explicit fallback behavior instead of keeping semantic truth in
    substring checks.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`

- [ ] Task SO-X.4: Migrate `checkpoint/progress.rs` comparability only after the earlier
      downstream seams are stable.
  - Acceptance: a later approved follow-on ports progress continuity/comparability away from the
    legacy TaskFrame/objective/working-set bridge to an explicit structured-objective continuity
    contract. Do not move this seam first.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`

- [ ] Task SO-X.5: Keep classifier work ask-first and outside the deterministic phase-1 slice.
  - Acceptance: any classifier experiment, training pipeline, or model dependency is proposed as a
    separate approved follow-on instead of being smuggled into the landed deterministic baseline.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - to be defined only when the classifier follow-on is approved
