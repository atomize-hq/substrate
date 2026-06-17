# Tasks: Agent Drift Analyzer Structured Objective Phase 1 Grounding Follow-On

Status: draft task ledger created on 2026-06-16 after validating the live structured-objective
crate state and the narrow harness-fix landing; reconciled on 2026-06-17 after the grounding
follow-on family landed through `SO-G6`. Keep this ledger as the historical closeout record for the
bounded grounding family rather than as an active implementation queue.

## SO-G0: Docs Lock

- [x] Task SO-G0.1: Add the grounding follow-on SPEC.
  - Acceptance:
    `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md`
    exists and captures assumptions, objective, commands, project structure, code style, testing
    strategy, boundaries, success criteria, and open questions for the bounded follow-on seam.
  - Verify: Manual review against the architecture, evaluation, migration, and existing phase-1 docs.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md`

- [x] Task SO-G0.2: Add the grounding follow-on PLAN and TASKS.
  - Acceptance: the follow-on plan and task ledger exist, keep the harness fix intact, and bound
    the next code seam to additive grounding restoration plus docs reconciliation.
  - Verify: Manual review against the live crate state and this follow-on spec.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md`

## SO-G1: Restore Additive Grounding Identifiers

- [x] Task SO-G1.1: Add optional `section_index` / `clause_index` back to `ObjectiveEvidenceSpan`.
  - Acceptance: `ObjectiveEvidenceSpan` again exposes additive optional section/clause identifiers
    with serde defaults, and older artifacts remain loadable without requiring the new fields.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/context/objective.rs`

- [x] Task SO-G1.2: Populate section/clause identifiers from the live decomposition path.
  - Acceptance: the clause-backed evidence path emits `section_index` / `clause_index` for goal,
    constraint, verification, and context spans when that information exists, while `start_char` /
    `end_char` remain optional and may stay `None`.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## SO-G2: Focused Grounding And Heading Regressions

- [x] Task SO-G2.1: Add focused regressions proving section/clause identifiers are populated.
  - Acceptance: checkpoint tests assert that at least goal and verification evidence spans carry
    populated `section_index` / `clause_index` values when clause-backed evidence exists.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task SO-G2.2: Add the duplicate-ish grounding ambiguity control.
  - Acceptance: a control case with similar scope/checklist wording proves the supporting span comes
    from the correct section/clause rather than only matching excerpt text.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task SO-G2.3: Add adversarial heading-classification controls.
  - Acceptance: focused regressions show `Task constraints`, `Verification task`, `Output request`,
    `Questions to ask`, and `Implementation steps` do not collapse into generic mission matching,
    while `What I need` still resolves as a mission-like heading when it contains the real goal.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## SO-G3: Reconcile Existing Phase-1 Docs To Landed Reality

- [x] Task SO-G3.1: Update the phase-1 task ledger so landed work is not still presented as open.
  - Acceptance: the existing phase-1 tasks doc explicitly distinguishes landed schema bridge,
    sidecar exposure, section/clause decomposition, and preliminary structured assembly from the
    remaining grounding hardening, acceptance-harness, and comparison-key work.
  - Verify: Manual review against the live crate state and `git diff --stat`.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

- [x] Task SO-G3.2: Reconcile the phase-1 plan with the real next seams.
  - Acceptance: the existing phase-1 plan no longer implies that the entire sidecar stack is still
    hypothetical, and it names the real remaining seams: grounding restoration, comparison-key
    derivation, acceptance harness, and deferred downstream migration.
  - Verify: Manual review against the architecture and migration authorities.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md`

## SO-G4: Lock The Comparison-Key Guardrail

- [x] Task SO-G4.1: Record that current `comparison_key` is provisional and not the approved
      semantic bridge.
  - Acceptance: the docs explicitly state that current `comparison_key` still mirrors display text
    and must not become the basis for downstream migration until the dedicated derivation packet
    lands.
  - Verify: Manual review of the phase-1 spec/plan/tasks plus this follow-on family.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md`
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

## SO-G5: Prepare The Next Structured-Objective Packet Seam

- [x] Task SO-G5.1: Make `SO-2.3B-refine` the explicit next packet after grounding.
  - Acceptance: the reconciled docs name `SO-2.3B-refine` as the next packet boundary after
    grounding restoration, then `SO-3.1` / `SO-3.2`, and only then `SO-4.1` / `SO-4.2` for the
    objective-acceptance harness.
  - Verify:
    - Manual review of the updated docs set.
    - `git diff --stat`
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md`
    - `docs/specs/r5/R5_75/MAP.md`

## SO-G6: Validation And Closeout

- [x] Task SO-G6.1: Run the focused checkpoint wall and the full analyzer wall on the grounding patch.
  - Acceptance: the additive grounding restoration lands with both the focused checkpoint suite and
    the full analyzer test wall green.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no source changes required unless validation exposes a packet-scoped defect

## Deferred / Ask-First

- [ ] Task SO-GX.1: Implement real `start_char` / `end_char` offsets only after the section/clause
      identifiers are restored and the future packet scope is approved.
  - Acceptance: a later approved follow-on defines the localization contract and keeps the patch
    distinct from this additive grounding restoration.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`

- [ ] Task SO-GX.2: Migrate downstream consumers to `comparison_key` or richer structured state only
      after the dedicated derivation packet lands.
  - Acceptance: a later approved follow-on proves that internal comparison no longer depends on the
    pretty display string and that downstream migration stays additive.
  - Verify: to be defined only when the follow-on is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/context/working_set.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
