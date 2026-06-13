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

## Phase 0: Docs and evaluation lock

Deliverables:

- architecture doc,
- evaluation/annotation doc,
- classifier taxonomy doc,
- this migration/integration doc.

Exit condition:

- doc stack agrees on field semantics, evidence rules, and migration order.

## Phase 1: Heuristic structured sidecar introduction

Primary change:

- add `structured: Option<StructuredObjective>` to `ObjectiveSummary`.

Rules:

1. keep current `text` field,
2. populate the sidecar heuristically first,
3. attach evidence spans,
4. render the current string from structured fields only when safe,
5. leave sidecar absent when evidence is too weak.

Rationale:

- this is the smallest architecture-valid change,
- it allows fixture and evaluation work before classifier work,
- it preserves compatibility for all current consumers.

## Phase 2: TaskFrame coexistence

Primary change:

- add `structured_objective: Option<StructuredObjective>` to `TaskFrame`.

Rules:

1. preserve `TaskFrame.objective` during the coexistence window,
2. teach new tests to assert both structured correctness and compatibility rendering,
3. do not widen public schema versioning unless the sidecar must be serialized externally.

## Phase 3: Downstream consumer migration

Suggested migration order:

1. `context/working_set.rs`
   - prefer target/evidence-driven path attribution over raw substring checks,
2. `checkpoint/mod.rs`
   - port closeout/review/no-code predicates to typed fields,
3. `checkpoint/progress.rs`
   - port continuity and overlap logic away from raw objective strings,
4. export and rendering surfaces
   - keep legacy strings available until all major downstream logic can consume the structured form.

This order reduces the risk of changing progress reasoning too early.

## Phase 4: Optional classifier augmentation

Only after the evaluation wall exists and the heuristic sidecar is stable:

- add Head A/B/C classifier assistance,
- compare against heuristic-only baseline,
- keep evidence-grounded assembly as the final authority.

## Compatibility Rules

During migration, the repo should obey these rules:

1. the legacy objective string remains available,
2. the sidecar is additive,
3. no downstream module may assume the sidecar is always present in phase 1,
4. unknowns are allowed and expected,
5. tests must validate both structure and compatibility.

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
6. only then `context/working_set.rs`, `checkpoint/mod.rs`, and `checkpoint/progress.rs`

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
