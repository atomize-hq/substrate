# Tasks: SO-2.3B-refine Structured Objective Bridge Honesty

Status: TASKS artifact created on 2026-06-17 after the packet-local SPEC and PLAN were written;
closeout reconciled on 2026-06-18 after `B1.1` through `B5.2` were verified against live code,
tests, and packet docs. This remains the audit ledger for the bounded corrective packet inside
`R5.75-1`; `SO-3` is next only because structured state now survives checkpoint narrowing, target
honesty is fixed, verification grounding is role-backed, the packet verification wall is green,
and residual risks are explicit.

Keep each task as close as possible to five touched files or fewer. Stay analyzer-local unless a
small compile-safe bridge proves unavoidable.

Packet prerequisite rule: when a packet prompt names earlier `B*` tasks as already landed, verify
those prerequisite tasks in live code/tests before editing. If one is missing, stop and report it
instead of compensating inside the later packet.

## Packet Boundary

This packet covers only:

- checkpoint narrowing preservation,
- target honesty,
- verification-role grounding alignment,
- targeted checkpoint regressions,
- final combined-case regression audit,
- packet closeout and handoff to `SO-3`.

This packet does **not** cover:

- compatibility rendering from structured state,
- deterministic `comparison_key` derivation,
- `objective_acceptance` harness/fixtures,
- TaskFrame coexistence,
- working-set / downstream progress migration,
- classifier/runtime work.

## B0: Docs Lock

- [x] Task B0.1: Create the packet-local SPEC / PLAN / TASKS set.
  - Acceptance: this directory contains a self-consistent `SPEC`, `PLAN`, and `TASKS` set that
    makes the packet boundary, assumptions, and next-packet order explicit before any code changes.
  - Verify: manual review of the three packet docs.
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md`
    - `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md`
    - `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md`

## B1: Preserve Structured Objective Through Checkpoint Narrowing

- [x] Task B1.1: Stop checkpoint narrowing from erasing structured state and prove it in the same packet.
  - Acceptance: `checkpoint_analyses(...)` preserves the richer `context.objective` already
    produced by `assemble_context(&window)` when that summary contains `structured`, instead of
    overwriting it with `ObjectiveSummary::compatibility(...)`; any legacy narrowed summary remains
    fallback-only and may layer display text only if `structured`, `verification_commands`,
    `unknowns`, and evidence spans survive intact; the packet does not default to re-running
    extraction over a different row slice unless it proves equivalence and preservation; and a
    targeted regression demonstrates that `analysis.current.context.objective.structured.is_some()`
    survives checkpoint narrowing while the grounded evidence still carries section/clause
    identifiers.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## B2: Stop Target Fabrication

Shared explicit-target anchor contract for `B2.1` / `B2.2`:

- Accepted explicit target anchors for this packet:
  - repo-relative file or directory paths
  - crate/package names with crate/package cues
  - spec/design/doc names or markdown/doc paths
  - test/verifier target names when the task is about the test/verifier itself
  - instruction surfaces such as `AGENTS.md`, `<skill>`, `Available skills`, or profile/plugin
    instructions
  - workspace refs such as `@shared-cab-app`
  - named packet/work item identifiers only when directly tied to the requested task
  - specific named conceptual artifact/topic spans only when the span itself is explicit
- Not enough by itself:
  - `this`
  - `it`
  - `the above`
  - `what landed`
  - `the current issue`
  - the entire goal sentence copied as target

Never derive `ObjectiveTargetKind::ConceptualTopic` from the whole goal clause alone; the
conservative fallback for weak review/analyze/fix clauses is `target == None` unless a specific
named conceptual artifact/topic span is present.

- [x] Task B2.1: Separate grounded goal selection from explicit target extraction and prove vague-target unknown behavior.
  - Acceptance:
    - vague review/analyze/fix prompts can keep a real goal while leaving `target == None`
    - `ObjectiveUnknown { field_name: "target", ... }` is present
    - at least one obvious explicit target case still survives, such as a file path or
      instruction-surface target
    - pronoun-only or copied-whole-goal fallbacks do not become `target` unless a separate
      accepted explicit anchor is present
    - `ObjectiveTargetKind::ConceptualTopic` is not synthesized from the whole goal clause; weak
      review/analyze/fix clauses fall back to `target == None` unless a specific named conceptual
      artifact/topic span is present
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task B2.2: Preserve explicit concrete targets and prove the preservation matrix in the same packet.
  - Acceptance: explicit file/directory, instruction-surface, spec/doc, test/verifier,
    crate/package, workspace-ref, directly tied packet/work-item targets, and specific named
    conceptual artifact/topic spans still survive extraction after the honesty tightening; and the
    packet lands the explicit-target preservation regression matrix proving those target families
    remain intact without reopening vague-target fabrication.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## B3: Align Verification Grounding With Verification Extraction

- [x] Task B3.1: Make verification-bearing clauses discoverable without a dedicated heading and prove it in the same packet.
  - Acceptance: bullet-only or inline verifier clauses that contain command-like verification work
    can produce grounded verification evidence even when the section is `Mission`, `Scope`, or
    `UnknownSection`; when `has_explicit_verification_cue(...)` is true, the clause receives an
    `ObjectiveRole::Verification` role candidate with evidence instead of merely suppressing
    `ObjectiveRole::Goal`; and the packet lands a bullet-only / inline verifier-role regression
    proving that behavior.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task B3.2: Prefer clause-grounded verification extraction over whole-row fallback and prove it in the same packet.
  - Acceptance: `verification_commands` come from any clause carrying a verification role
    candidate when such clauses exist, not only from clauses where `Verification` is the top role,
    while whole-candidate fallback remains only a conservative backup path; and the packet lands a
    clause-grounded extraction regression proving the preference directly, including a mixed-role
    sentence such as `Review the objective extractor, run make test, and return concrete fixes.`
    that resolves to goal / verification / deliverable spans while still capturing `make test`.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/context/objective.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## B4: Final Coverage Audit

- [x] Task B4.1: Audit final coverage and add any missing combined-case regressions.
  - Acceptance: after B1-B3 land, the packet audits the remaining proof surface and adds only the
    missing combined-case or residual regressions needed to make the family fully reviewable and
    durable; B4 does not carry the core proof load for B1-B3.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - minimal packet-scoped code files only if required to make the final combined-case audit honest

## B5: Packet Closeout

- [x] Task B5.1: Run the packet verification wall.
  - Acceptance: focused checkpoint regressions and the full analyzer suite are green on the landed
    packet.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no source changes required unless validation exposes a packet-scoped defect

- [x] Task B5.2: Capture the next-packet handoff honestly.
  - Acceptance:
    - packet closeout notes record that `SO-2.3B-refine` refined already-landed preliminary
      structured assembly rather than starting a competing greenfield plan
    - `SO-3.1` / `SO-3.2` are clearly next only because structured state survives checkpoint
      narrowing, target honesty is fixed, verification grounding is role-backed, the packet
      verification wall is green, and residual risks are documented
    - `SO-4` / `SO-5` remain blocked until `SO-3` lands
    - the older phase-1 task ledger no longer reads as if `SO-2.1` / `SO-2.2` / `SO-2.3` are all
      still greenfield work
  - Verify: manual review of the packet docs and any touched routing note.
  - Files:
    - `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`
    - no additional files required unless closeout notes are updated during landing

## Closeout Notes (2026-06-18)

- This packet closed as a corrective refinement over already-landed preliminary structured
  assembly from `SO-2.1` / `SO-2.2` / `SO-2.3`; it did not replace that earlier seam with a
  competing greenfield plan.
- Structured preservation is now explicit in live behavior: `checkpoint_analyses(...)` preserves
  the richer structured objective during narrowing, and the regression wall includes
  `checkpoints_preserve_structured_objective_when_narrowing_runs`.
- Target honesty is now explicit in live behavior: vague review/analyze/fix asks keep a grounded
  goal while leaving `target` unknown, and the explicit-target preservation matrix remains green.
- Verification grounding is now explicit in live behavior: verification commands are backed by
  clause-level verification-role evidence across unheaded, inline, and mixed-role cases.
- Packet verification wall rerun on 2026-06-18:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- Next packet boundary: `SO-3.1` / `SO-3.2` only.
- Still blocked: `SO-4` / `SO-5` remain blocked until `SO-3` lands.
- Residual risks:
  - `comparison_key` still mirrors display text until `SO-3.2` derives it from structured state.
  - compatibility text is still not rendered from structured state as the semantic authority until
    `SO-3.1`.
  - the committed `objective_acceptance` harness and locked fixture wall are still absent until
    `SO-4` / `SO-5` after `SO-3`.
