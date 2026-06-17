# Design: R5 Structured Objective Migration And Repo Integration

Status: draft design note created on 2026-06-13 from the objective-architecture dossier and live
repo inspection.

## Why This Doc Exists

The canonical architecture doc describes the target state, but the repo still depends on legacy
objective strings in multiple places. This doc explains how the structured-objective design should
be integrated incrementally without breaking the analyzer's existing contracts.

## Current Integration Seams

The following seams are the practical migration surface today.

### Seam 1: `context/objective.rs`

Current role:

- choose one objective row,
- return `ObjectiveSummary { text, verification_commands, evidence }`.

Why it matters:

- this is the earliest point where structured extraction can be introduced.

### Seam 2: `context/working_set.rs`

Current role:

- use `objective.text.contains(path)` when attributing truth-artifact origin.

Why it matters:

- path sourcing should eventually come from structured target/evidence, not just substring checks.

### Seam 3: `inference/mod.rs`

Current role:

- carry objective text into `TaskFrame.objective`.

Why it matters:

- this is where the compatibility string and the structured sidecar need to coexist.

### Seam 4: `checkpoint/mod.rs`

Current role:

- use string-level objective predicates for closeout/review/no-code semantics.

Why it matters:

- these should gradually migrate toward typed fields rather than brittle substring logic.

### Seam 5: `checkpoint/progress.rs`

Current role:

- compare older/newer checkpoint objectives through normalized strings and token overlap.

Why it matters:

- progress comparability is one of the highest-risk downstream consumers of lossy objective text.

### Seam 6: regression surfaces

Current role:

- many tests still assert exact objective strings.

Why it matters:

- the migration must update tests to validate structure plus compatibility, not structure or string
  alone.

## Recommended Migration Phases

### Phase 0: Docs and evaluation lock

Deliverables:

- architecture doc,
- evaluation/annotation doc,
- classifier taxonomy doc,
- this migration/integration doc.

Exit condition:

- doc stack agrees on field semantics, evidence rules, migration order, and acceptance scoring.

### Phase 1A: Schema types + additive sidecar

Primary change:

- add `structured: Option<StructuredObjective>` and `comparison_key: String` to
  `ObjectiveSummary`.

Rules:

1. keep current `text` field,
2. keep `text` as the display compatibility rendering,
3. add `comparison_key` as the deterministic semantic comparison key,
4. leave the sidecar optional,
5. do not allow downstream consumers to assume the sidecar is always present.

Recommended compatibility shape:

```rust
pub struct ObjectiveSummary {
    pub text: String,
    pub comparison_key: String,
    pub structured: Option<StructuredObjective>,
    pub verification_commands: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
}
```

Rationale:

- this is the smallest architecture-valid change,
- it creates a stable bridge away from one-string comparison,
- it preserves compatibility while richer fields are introduced.

### Phase 1B: Heuristic section/clause decomposition + evidence spans

Primary change:

- populate the sidecar heuristically with section-aware and clause-aware evidence spans.

Rules:

1. phase 1 remains deterministic and dependency-free,
2. evidence spans must include section kind and clause offsets when available,
3. unknown fields are valid outcomes,
4. fields without evidence must stay absent or unknown.

Rationale:

- this is the first phase that materially fixes the WDAP section-hierarchy failure,
- it establishes the structure that later classifier work may assist but not replace.

### Phase 1C: Compatibility rendering + objective acceptance tests

Primary change:

- render compatibility output from structured state when safe and validate it with the objective
  acceptance wall.

Rules:

1. `ObjectiveSummary.text` remains the public display string,
2. `comparison_key` becomes the preferred internal comparison surface,
3. acceptance tests must score structured fields, grounding, forbidden promotions, and acceptable
   compatibility rendering,
4. exact pretty-string equality is no longer the sole success criterion.

Exit condition:

- WDAP seed cases stop promoting checklist lines,
- preserved boilerplate-target cases remain intact,
- compatibility rendering stays acceptable for current consumers.

### Phase 2: TaskFrame coexistence

Primary change:

- add `structured_objective: Option<StructuredObjective>` and `objective_key: String` to
  `TaskFrame`.

Rules:

1. preserve `TaskFrame.objective` during the coexistence window,
2. `TaskFrame.objective` stays the public display string,
3. `TaskFrame.objective_key` becomes the preferred internal comparison source,
4. teach new tests to assert both structured correctness and compatibility rendering,
5. do not widen public schema versioning unless the sidecar must be serialized externally.

Recommended coexistence shape:

```rust
pub struct TaskFrame {
    pub objective: String,
    pub objective_key: String,
    pub structured_objective: Option<StructuredObjective>,
    // existing fields remain during migration
}
```

### Phase 3: Downstream consumer migration

Suggested migration order:

1. `context/working_set.rs`
   - first downstream migration target,
   - prefer target/evidence-driven path attribution over raw substring checks,
2. `checkpoint/mod.rs`
   - port closeout/review/no-code predicates to typed fields only after TaskFrame coexistence is
     stable,
3. `checkpoint/progress.rs`
   - port continuity and overlap logic away from raw objective strings only after TaskFrame
     coexistence and working-set migration have landed,
4. export and rendering surfaces
   - keep legacy strings available until all major downstream logic can consume the structured form.

This order reduces the risk of changing progress reasoning too early.

### Phase 4: Optional classifier augmentation

Only after the evaluation wall exists and the heuristic sidecar is stable:

- add Head A/B/C classifier assistance,
- compare against heuristic-only baseline,
- keep evidence-grounded assembly as the final authority.

## Compatibility Rules

During migration, the repo should obey these rules:

1. the legacy objective string remains available,
2. the sidecar is additive,
3. no downstream module may assume the sidecar is always present until its seam is explicitly
   migrated,
4. unknowns are allowed and expected,
5. tests must validate both structure and compatibility,
6. internal comparison should prefer `comparison_key` / `objective_key` before richer structured
   comparison is fully wired.

## Sidecar Presence Guard

No downstream migration should land without an explicit sidecar-presence guard.

That means migrated consumers must define what they do when:

- structured sidecar is present and high-confidence,
- structured sidecar is absent,
- structured sidecar is present but key fields remain unknown.

Silent fallback from missing structure back to brittle string assumptions must be visible in tests.

## Failure-Mode Guardrails

### Guardrail 1: Do not break WDAP while improving easy cases

Every migration phase must keep the WDAP family in the acceptance wall.

### Guardrail 2: Do not regress deliberate boilerplate-target requests

Preserved-target cases must remain first-class migration tests.

### Guardrail 3: Do not over-upgrade weak evidence into fake structure

If the extractor is uncertain, keep fields unknown. Compatibility text may still fall back to legacy
behavior when necessary.

### Guardrail 4: Do not move progress reasoning first

`checkpoint/progress.rs` is downstream of several other seams and should not be the first migration
step.

### Guardrail 5: Do not let compatibility display text remain the hidden authority

Any newly migrated downstream consumer should prefer typed fields or `comparison_key` /
`objective_key` rather than reintroducing string truth under a new name.

## Risks

### Risk: schema sprawl before value

Mitigation:

- keep phase 1 small,
- do not add every conceivable field immediately.

### Risk: compatibility text stays the real truth forever

Mitigation:

- every new downstream consumer should prefer structured fields when present,
- string-based logic should be treated as migration debt, not the target state.

### Risk: classifier work starts before the benchmark exists

Mitigation:

- keep evaluation/annotation work ahead of model work.

### Risk: evidence spans are omitted in early prototypes

Mitigation:

- make evidence spans mandatory for all nontrivial fields in the sidecar contract.

## Suggested Future File Touch Order

When implementation eventually begins, the safest initial touch order is:

1. `crates/agent-drift-analyzer/src/context/objective.rs`
2. `crates/agent-drift-analyzer/src/context/mod.rs`
3. `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
4. `crates/agent-drift-analyzer/src/inference/mod.rs`
5. tests for context/task frame/objective acceptance
6. only then `context/working_set.rs`
7. only then `checkpoint/mod.rs`
8. only then `checkpoint/progress.rs`

## Non-Goals

This design does not define:

- packet breakdown,
- commit order,
- final schema versioning,
- a date for retiring the legacy string.

## Source Map

Primary local authority used for this design:

- `.codex/handoffs/2026-06-13-objective-architecture-decision-dossier.md`
- `.codex/handoffs/2026-06-13-081405-r5-75-objective-failure.md`
- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/src/context/working_set.rs`
- `crates/agent-drift-analyzer/src/context/mod.rs`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
