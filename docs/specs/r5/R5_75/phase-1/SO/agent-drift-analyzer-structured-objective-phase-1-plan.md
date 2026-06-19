# Plan: Agent Drift Analyzer Structured Objective Phase 1

Status: draft plan created on 2026-06-14 from the structured-objective design stack; reconciled
on 2026-06-17 against the live crate snapshot plus the grounding follow-on family through `SO-G6`,
then reconciled again after the landed `SO-3` through `SO-6.1` packets so this plan reflects the
live phase-1 closeout and the remaining post-phase-1 debt explicitly. The map doc was used only as
a routing overview; architecture owns semantics, evaluation owns acceptance, migration owns landing
order, and classifier taxonomy remains deferred for this phase.

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
8. Phase-1 packet prompts must verify any named earlier packet prerequisites against live
   code/tests before editing; later packets stop/report missing prerequisite work instead of
   silently absorbing it.

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
   claims should route through structured state plus a derived `comparison_key`. That semantic-key
   derivation is now landed; any remaining follow-on work should treat downstream migration, not
   `comparison_key` derivation itself, as the open seam. The checkpoint bridge must preserve the
   richer `assemble_context(...)` objective when structured state already exists; legacy narrowing
   is fallback/display-only, not semantic truth replacement.
5. **Defer risky downstream edits.** `working_set`, `checkpoint/mod.rs`, and `checkpoint/progress.rs`
   should stay untouched in Phase 1 unless a later approved follow-on explicitly promotes them.

## Dependency Graph

```text
phase-1 docs lock
  -> schema bridge (`StructuredObjective`, `comparison_key`, optional sidecar) [landed]
  -> deterministic decomposition and preliminary structured assembly [landed]
  -> grounding identifier restoration + adversarial heading proof [landed later via `SO-G1`/`SO-G2`]
  -> bridge-honesty corrective packet (`SO-2.3B-refine`) [landed]
  -> compatibility rendering from structured state + deterministic `comparison_key` derivation [landed]
  -> objective-acceptance harness + committed fixtures [landed]
  -> full analyzer regression closeout [landed]
  -> explicit follow-on seam capture in the phase-1 docs [this packet]

explicitly deferred:
  -> TaskFrame coexistence (`checkpoint/schema.rs`, `inference/mod.rs`)
  -> working-set path-attribution migration (`context/working_set.rs`)
  -> checkpoint semantic predicate migration (`checkpoint/mod.rs`)
  -> progress comparability migration (`checkpoint/progress.rs`)
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

### Reconciled Live Status

This packet family is now landed in the live crate snapshot. `ObjectiveSummary.text` is rendered
from structured state when the evidence is strong enough, and `comparison_key` is derived from the
structured frame rather than mirroring display text. The remaining migration work is downstream of
Phase 1 and is now tracked explicitly under the deferred follow-on list instead of being left as an
implicit blocker here.

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

### Reconciled Live Status

This packet family is now landed in the live crate snapshot. The `objective_acceptance` harness,
fixture loader/support, and expected-shape contract are committed, and the locked corpus is part of
the validation wall that `SO-6.1` ran green. The remaining gaps are not harness gaps; they are the
explicit downstream follow-on seams captured at the end of this plan.

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

### Verification

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

### Historical Ordering Note

This packet originally followed `SO-2.3B-refine` and `SO-3` so the acceptance harness would validate the intended Phase-1 semantics (including compatibility rendering and semantic comparison) instead of defining them after the fact. That sequencing dependency is already satisfied in the live repo snapshot because `SO-3`, `SO-4`, `SO-5`, and `SO-6.1` are now landed; keep this note only as historical rationale, not as still-open backlog.

## SO-5: Seed The Locked Acceptance Wall

### Reconciled Live Status

This packet family is now landed in the live crate snapshot. The locked acceptance corpus includes
the WDAP kickoff seeds, preserved instruction-surface controls, concise `/goal` controls, and the
review/no-code plus planning/docs controls needed for the phase-1 acceptance wall.

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

### Reconciled Live Status

The analyzer-local validation wall is now landed and was rerun green after the `SO-5.3` review-fix
loop. Phase 1 can now close honestly because the remaining non-phase-1 work is explicit: the docs
record the downstream migration seams and classifier deferrals instead of leaving them implied by
stale “remaining gap” language or half-wired code comments.

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

These are intentionally out of scope for this plan unless a later approval promotes them. They are
not hidden phase-1 TODOs; they are the explicit post-phase-1 seam list.

1. **TaskFrame coexistence (`checkpoint/schema.rs`, `inference/mod.rs`)**
   - live repo truth: `TaskFrame` still carries only the legacy `objective` string plus companion
     evidence/working-set fields; it does not yet expose `objective_key` or
     `structured_objective`.
   - follow-on requirement: add the coexistence bridge only after an approved Phase 2 promotion so
     downstream code can consume structured semantics without deleting the compatibility surface.
2. **Working-set path attribution migration (`context/working_set.rs`)**
   - live repo truth: path attribution still uses `objective.text.contains(&path)` as the bridge.
   - follow-on requirement: migrate to structured target/evidence-driven attribution with explicit
     sidecar-presence guards.
3. **Checkpoint predicate migration (`checkpoint/mod.rs`)**
   - live repo truth: closeout/review/no-code predicates still inspect objective text directly.
   - follow-on requirement: port those predicates to typed structured fields only after TaskFrame
     coexistence is available and validated.
4. **Progress comparability migration (`checkpoint/progress.rs`)**
   - live repo truth: comparability still depends on legacy TaskFrame/objective/working-set
     surfaces rather than a richer structured-objective continuity contract.
   - follow-on requirement: migrate progress reasoning only after TaskFrame coexistence plus the
     earlier downstream seams are stable; do not move this seam first.
5. **Classifier work**
   - live repo truth: no classifier/runtime dependency is part of the landed phase-1 slice.
   - follow-on requirement: keep classifier experiments, training pipelines, or model dependencies
     as ask-first work after the deterministic baseline has already proved itself.
