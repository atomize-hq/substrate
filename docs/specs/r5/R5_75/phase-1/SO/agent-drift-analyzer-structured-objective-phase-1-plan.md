# Plan: Agent Drift Analyzer Structured Objective Phase 1

Status: draft plan created on 2026-06-14 from the structured-objective design stack; reconciled
on 2026-06-17 against the live crate snapshot plus the grounding follow-on family through `SO-G6`.
The map doc was used only as a routing overview; architecture owns semantics, evaluation owns
acceptance, migration owns landing order, and classifier taxonomy remains deferred for this phase.

## Objective

Land the first implementation-valid structured-objective slice as an additive, deterministic
analyzer-local sidecar with a committed acceptance wall, while explicitly deferring classifier work
and broader downstream migration until the sidecar proves itself.

## Planning Decisions Locked For This Draft

1. Phase 1 is limited to migration phases **1A, 1B, and 1C** from the migration authority:
   additive schema bridge, deterministic structured extraction, and compatibility/acceptance
   wiring.
2. Phase 1 does **not** include migration Phase 2 (`TaskFrame` coexistence) or Phase 3 downstream
   consumers (`working_set`, `checkpoint/mod.rs`, `checkpoint/progress.rs`) except for the minimum
   compile-safe wiring needed to keep current behavior intact.
3. The architecture authority wins any semantic dispute about fields, evidence spans, section kinds,
   role labeling, or compatibility meaning.
4. The evaluation authority wins any dispute about fixture shape, corpus families, metrics,
   forbidden-promotion scoring, or acceptance proof.
5. The migration authority wins any dispute about landing order, sidecar coexistence rules, and
   what must remain additive during this phase.
6. The classifier taxonomy is explicitly non-blocking for this phase. No analyzer-time classifier,
   no model runtime, and no dependency work should be folded into this landing.
7. Unknowns are a success condition. If the extractor is unsure, it should preserve evidence and
   leave the field unknown rather than guess.

## Why This Phase Is Separate From The Current Stopgap

`R5.75-1` objective condensation is still the current narrow stopgap for giant pasted prompts.
The structured-objective phase should not be treated as “keep extending `normalized_objective_text`.”
It is a new architecture-valid seam with a different proof wall:

- decomposition before assembly,
- grounded fields instead of one canonical string,
- compatibility text as projection,
- objective acceptance as the promotion gate.

## Implementation Principles

1. **Add structure before migrating consumers.** The sidecar must exist and prove itself before any
   downstream module starts depending on it.
2. **Prefer analyzer-local isolation.** Keep the first landing centered in `context/objective.rs`,
   `context/mod.rs`, `checkpoint/schema.rs`, and analyzer-local tests/fixtures.
3. **Keep acceptance close to the design.** The objective-acceptance suite should be committed and
   reviewable, not left as an aspirational future TODO.
4. **Preserve compatibility honestly.** The legacy string remains available, but new correctness
   claims should route through structured state plus a derived `comparison_key`. Until SO-3.2
   lands, the live key remains provisional because it still mirrors display text.
5. **Defer risky downstream edits.** `working_set`, `checkpoint/mod.rs`, and `checkpoint/progress.rs`
   should stay untouched in Phase 1 unless a later approved follow-on explicitly promotes them.

## Dependency Graph

```text
phase-1 docs lock
  -> schema bridge (`StructuredObjective`, `comparison_key`, optional sidecar) [landed]
  -> deterministic decomposition and preliminary structured assembly [landed]
  -> grounding identifier restoration + adversarial heading proof [landed later via `SO-G1`/`SO-G2`]
  -> compatibility rendering from structured state + deterministic `comparison_key` derivation [remaining]
  -> objective-acceptance harness + committed fixtures [remaining]
  -> full analyzer regression closeout [remaining]

explicitly deferred:
  -> TaskFrame coexistence
  -> working-set migration
  -> checkpoint semantic predicate migration
  -> progress comparability migration
  -> classifier experiments
```

## Recommended Landing Sequence

## SO-0: Docs Lock (This Spec / Plan / Tasks)

### Scope

- add phase-1 SPEC/PLAN/TASKS docs
- record the authority precedence explicitly
- keep the immediate implementation scope bounded to Phase 1 only

### Why First

The architecture, acceptance, and landing-order rules are spread across multiple design docs. The
implementation-facing packet needs one stable authority set before code begins.

### Verification

Manual review only.

## SO-1: Schema Bridge And Additive Sidecar

### Reconciled Live Status (2026-06-16)

This seam is already landed in the live crate snapshot: `StructuredObjective`, the supporting
semantic enums, additive `ObjectiveEvidenceSpan` / `ObjectiveUnknown` schema, and
`ObjectiveSummary.{comparison_key, structured}` all exist today. Keep this section as historical
packet structure, but do not treat the schema bridge or sidecar exposure as open backlog anymore.

### Scope

- define phase-1 `StructuredObjective` DTOs, evidence-span enums, and unknown representation
- add `comparison_key` and `structured: Option<StructuredObjective>` to `ObjectiveSummary`
- preserve `ObjectiveSummary.text`, `verification_commands`, and `evidence`
- keep all new fields additive and optional

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/src/context/mod.rs
```

### Why Before Anything Else

The additive schema bridge is the smallest architecture-valid change and creates the compatibility
surface every later step depends on.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## SO-2: Deterministic Section/Clause Decomposition And Structured Assembly

### Reconciled Live Status (2026-06-16)

This seam is also already materially landed: the current `context/objective.rs` path performs
section-aware decomposition, clause-role labeling, evidence-span grounding, and preliminary
structured assembly with unknown preservation. Later grounding hardening was tracked in the
follow-on `SO-G1` / `SO-G2` family rather than by pretending original `SO-2` work never existed.

### Scope

- decompose candidate directive text into sections and clause/sentence units
- label clause roles (`goal`, `constraint`, `verification`, `context`, `other_role`)
- assemble grounded phase-1 fields from those labeled spans
- preserve weak evidence as `unknowns`
- prevent subordinate checklist lines from outranking broader mission/scope spans

### Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why After SO-1

Decomposition and assembly need the new DTOs and additive sidecar shape already in place.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## SO-3: Compatibility Rendering And Comparison-Key Derivation

### Current Remaining Gap

This packet family is still open. The live crate exposes `comparison_key`, but the current
implementation still mirrors display text instead of deriving a stable semantic key from the
structured frame, and compatibility text is still selected directly from decomposition rather than
rendered from structured state as the semantic authority. Treat the live key as a provisional
stopgap only; downstream migration must not anchor on it until the dedicated derivation work lands.
After the grounding follow-on closes, however, this is no longer the next packet to start: the
objective-acceptance harness boundary in SO-4 comes first so the remaining semantic work lands
against an explicit acceptance wall instead of another vague future TODO.

### Scope

- render `ObjectiveSummary.text` from structured state when the evidence is strong enough
- derive deterministic `comparison_key` from the structured frame
- keep safe fallback behavior when the sidecar is absent or key fields are unknown
- prove that compatibility rendering is a projection, not the semantic authority

### Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why Separate From SO-2

This isolates the “projection back to legacy behavior” logic from the richer extraction work and
makes compatibility regressions easier to review.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## SO-4: Objective Acceptance Harness And Fixture Contract

### Current Remaining Gap

This packet family is still open. `crates/agent-drift-analyzer/tests/objective_acceptance.rs` and
the `tests/fixtures/objective_acceptance/**` corpus are not present yet, so the committed
acceptance wall promised by Phase 1 remains future work. Once the grounding follow-on family lands,
SO-4.1 and SO-4.2 are the explicit next packet boundary even though SO-3 remains unfinished:
future packet agents should start by standing up this harness contract before widening into SO-5
fixture expansion or downstream migration.

### Scope

- add `tests/objective_acceptance.rs`
- add the fixture loader/support needed for the new corpus
- codify the fixture directory contract for `design-set`, `locked-acceptance`, and
  `stretch-external`
- assert the required summary metrics and forbidden-promotion surfaces
- require the harness contract to validate structured fields, role spans, grounding refs,
  forbidden promotions, compatibility rendering, and unknown-field correctness

### Primary Files

```text
crates/agent-drift-analyzer/tests/objective_acceptance.rs
crates/agent-drift-analyzer/tests/support/mod.rs
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/**
```

### Why Before Fixture Expansion

The harness and contract must exist before a bounded set of cases can be added safely and reviewed
for drift. This is also the explicit next acceptance packet after grounding restoration, so later
SO-3 and SO-5 work has a concrete wall to target instead of another implied sequence step.

### Verification

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

## SO-5: Seed The Locked Acceptance Wall

### Scope

Add a small committed corpus that covers the acceptance authority's required families in priority
order:

1. WDAP linux and macOS seeds first,
2. deliberate boilerplate-target preserved cases,
3. concise `/goal` control cases,
4. review/no-code cases,
5. planning/research/docs cases.

The design-set may stay small, but the locked acceptance set must include the WDAP seeds and the
preserved-target controls before Phase 1 can close.

### Primary Files

```text
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/**
crates/agent-drift-analyzer/tests/objective_acceptance.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## SO-6: Full Phase-1 Validation And Closeout Review

### Current Remaining Gap

This closeout packet is still blocked on the unfinished compatibility/comparison-key work and the
missing objective-acceptance harness. It should remain open until those remaining seams land and
the analyzer-local validation wall can be run honestly.

### Scope

- run the full analyzer gate set
- confirm no consumer assumes sidecar presence
- confirm the phase stayed analyzer-local and did not silently widen into downstream migration or
  classifier work
- capture any remaining follow-on debt as explicit post-phase-1 items, not hidden TODOs in code

### Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## Risk Register

### Risk: schema sprawl before the first useful landing

Mitigation:

- keep Phase 1 limited to the architecture's minimum semantic coverage
- defer TaskFrame and downstream consumer migration

### Risk: compatibility text stays the real truth source

Mitigation:

- derive `comparison_key` from structured state
- treat compatibility rendering as a projection with explicit fallback rules
- keep exact-string equality out of the main acceptance gate

### Risk: weak evidence gets promoted into fake structure

Mitigation:

- require evidence spans for all nontrivial fields
- encode ambiguity as `unknowns`
- keep forbidden-promotion checks in the acceptance wall

### Risk: objective acceptance corpus grows uncontrolled

Mitigation:

- add the harness and fixture contract before broadening the corpus
- keep WDAP and preserved-target cases as the required locked core

## Deferred Follow-On Work (Not Phase 1)

These are intentionally out of scope for this plan unless a later approval promotes them:

1. `TaskFrame.objective_key` and `structured_objective` coexistence (migration Phase 2)
2. `context/working_set.rs` path attribution migration
3. `checkpoint/mod.rs` typed closeout/review/no-code predicates
4. `checkpoint/progress.rs` comparability migration away from raw objective strings
   - do not start this migration while `comparison_key` still mirrors display text; wait for the
     approved SO-3.2 derivation first
5. any classifier experiment, training pipeline, or model dependency
