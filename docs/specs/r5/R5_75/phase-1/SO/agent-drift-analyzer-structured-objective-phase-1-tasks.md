# Tasks: Agent Drift Analyzer Structured Objective Phase 1

Status: draft task ledger created on 2026-06-14 from the structured-objective design stack;
reconciled on 2026-06-17 against the live crate snapshot plus the grounding follow-on family
through `SO-G6`. Architecture owns semantics, evaluation owns acceptance, migration owns landing
order, and the classifier taxonomy remains deferred for this phase. Keep the original packet
structure for auditability, but do not treat already-landed SO-1 / SO-2 seams as still-open
implementation debt.

Keep each task as close as possible to five touched files or fewer. Do not advance to the next task
until the current task's verification commands are green or the failure is explicitly captured in
packet notes.

Packet prerequisite rule: when a phase-1 packet prompt names earlier packets as already landed,
verify those prerequisite tasks in live code/tests before editing. If a prerequisite is missing,
stop and report it instead of letting the later packet absorb the earlier bug.

## SO-0: Docs Lock

- [x] Task SO-0.1: Add the phase-1 SPEC.
  - Acceptance: `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md` exists
    and covers assumptions, objective, commands, project structure, code style, testing strategy,
    boundaries, success criteria, and open questions.
  - Verify: Manual review against the five structured-objective design docs and `AGENTS.md`.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md`

- [x] Task SO-0.2: Add the phase-1 PLAN and TASKS.
  - Acceptance: the plan and task ledger exist, preserve the authority precedence, and keep Phase 1
    bounded to additive sidecar + acceptance work.
  - Verify: Manual review against the architecture, evaluation, and migration authorities.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

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
    acceptance is about additive exposure only; semantic `comparison_key` derivation remains open
    under SO-3.2, and the current key is still only a provisional display-text mirror rather than
    the approved semantic bridge for downstream migration.
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
landed here.

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

Remaining work note: this section is still open. The live implementation exposes structured state,
but `comparison_key` still mirrors display text and compatibility text is not yet rendered from the
structured frame as the semantic authority. Treat the current key as a stopgap only; no downstream
migration may anchor on it until SO-3.2 lands. After the grounding follow-on closes, the next
immediate packet is `SO-2.3B-refine`, which preserves structured state through checkpoint
narrowing, tightens target honesty, and aligns verification grounding before the compatibility/key
projection work begins. Once `SO-2.3B-refine` lands, `SO-3.1` and `SO-3.2` are the next packet
boundary before `SO-4` and `SO-5`.

- [ ] Task SO-3.1: Render compatibility text from structured state when safe.
  - Acceptance: `ObjectiveSummary.text` becomes a compatibility view over structured state when the
    evidence is sufficient, but falls back conservatively when the sidecar is absent or key fields
    remain unknown.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task SO-3.2: Derive deterministic `comparison_key` from the structured frame.
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

Remaining work note: this harness has not landed yet. The planned `objective_acceptance` test file
and fixture families are still absent from the current crate snapshot. Do not start `SO-4` until
`SO-2.3B-refine` lands and `SO-3.1` / `SO-3.2` have completed; the harness should validate those
corrected semantics rather than stand in for them. Do not skip directly to SO-5 fixture seeding or
later migration work.

- [ ] Task SO-4.1: Add the objective-acceptance test harness and fixture loader.
  - Acceptance: `tests/objective_acceptance.rs` exists, can load committed objective-acceptance
    fixtures deterministically, and asserts the fixture directory contract for the three evaluation
    families. This packet starts only after `SO-2.3B-refine` and `SO-3.1` / `SO-3.2` are landed.
  - Verify: `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`

- [ ] Task SO-4.2: Encode the objective-acceptance expected-shape contract.
  - Acceptance: the harness checks structured-field expectations, role spans, grounding refs,
    forbidden promotions, compatibility rendering, and unknown-field correctness from committed
    fixture metadata rather than relying on one exact objective string, and it defines the concrete
    acceptance wall that later SO-3 and SO-5 work must satisfy.
  - Verify: `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`

## SO-5: Seed The Locked Acceptance Wall

Packet-ordering note: do not start SO-5 until `SO-2.3B-refine`, `SO-3.1`, `SO-3.2`, and then
SO-4.1 / SO-4.2 have landed as the explicit pre-fixture acceptance boundary.

- [ ] Task SO-5.1: Add WDAP linux and macOS locked-acceptance seeds.
  - Acceptance: the committed objective-acceptance corpus includes the required WDAP kickoff seeds,
    and their expected metadata explicitly forbids subordinate checklist lines from being promoted
    to `goal`.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`

- [ ] Task SO-5.2: Add preserved boilerplate-target controls.
  - Acceptance: the locked acceptance set contains deliberate `AGENTS.md` / `<skill>` /
    instruction-surface cases where the real task is to analyze or edit that surface, and the
    structured extractor preserves that target instead of filtering it away.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`

- [ ] Task SO-5.3: Add concise-goal, review/no-code, and planning/docs controls.
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

Remaining work note: honest closeout still depends on finishing SO-3 and SO-4 / SO-5 first; do
not mark this packet family done while compatibility/comparison-key work or the acceptance wall is
still missing.

- [ ] Task SO-6.1: Run the full phase-1 validation wall.
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

- [ ] Task SO-6.2: Capture the deferred follow-on seam list explicitly.
  - Acceptance: any remaining TaskFrame coexistence, working-set migration, checkpoint predicate
    migration, progress comparability migration, or classifier work is recorded as explicit follow-on
    debt rather than left implicit in comments or half-wired code.
  - Verify: Manual review of the landed phase-1 docs and code diff.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

## Deferred / Ask-First (Not Phase 1)

- [ ] Task SO-X.1: Add `TaskFrame.objective_key` and `structured_objective` only after the phase-1
      sidecar and objective-acceptance wall are green.
  - Acceptance: a later approved follow-on explicitly promotes migration Phase 2, and the work
    stays additive instead of silently widening this phase.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/inference/mod.rs`

- [ ] Task SO-X.2: Migrate downstream consumers (`working_set`, `checkpoint/mod.rs`,
      `checkpoint/progress.rs`) only after TaskFrame coexistence lands.
  - Acceptance: a later approved follow-on preserves sidecar presence guards and does not re-create
    raw-string truth under a new name. That migration must remain blocked while the current
    `comparison_key` still mirrors display text instead of the approved derived semantic key.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/context/working_set.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
