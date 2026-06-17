# Design: R5 Structured Objective Map

Status: draft map created on 2026-06-14 to provide a top-down overview of the structured-objective
design stack without introducing a second competing architecture authority.

## Objective

Provide one routing and authority document that explains:

1. why the structured-objective work exists,
2. how the four design docs fit together,
3. which doc answers which class of question,
4. what the intended implementation sequence is,
5. which decisions are already locked vs still open.

This document is a navigation layer, not a replacement for the underlying design docs.

## Problem Summary

The current analyzer still allows one extracted objective string to become canonical task truth.
The confirmed WDAP kickoff failure shows why that abstraction is too weak: a subordinate checklist
line can outrank the real mission in `## Scope`, and that lossy string then leaks into
working-set attribution, task framing, checkpoint predicates, and progress comparability.

The structured-objective design stack exists to replace:

```text
one chosen row -> one normalized objective string -> task truth
```

with a staged, grounded, typed objective representation whose compatibility string is only a view.

## Design Stack Overview

The structured-objective work is intentionally split across four primary design docs.

### 1. Architecture

File:

- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-architecture.md`

Owns:

- the target abstraction,
- the phase-1 `StructuredObjective` shape,
- evidence span requirements,
- section/clause decomposition stages,
- frame assembly precedence,
- semantic-vs-execution metadata boundary,
- the rule that compatibility text is downstream-only.

Use this doc when asking:

- what is the new objective model,
- what fields exist in phase 1,
- what counts as semantic state,
- what the WDAP bug means architecturally,
- what rules govern field assembly and grounding.

### 2. Evaluation and annotation

File:

- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md`

Owns:

- the acceptance wall,
- corpus families,
- annotation units and labels,
- expected fixture schema,
- required metrics,
- forbidden-promotion scoring,
- unknown-field correctness expectations.

Use this doc when asking:

- how we prove the new design is better,
- what benchmark cases exist,
- what `expected.json` should contain,
- what counts as a forbidden promotion,
- how acceptance runs should be summarized.

### 3. Classifier taxonomy

File:

- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-classifier-taxonomy.md`

Owns:

- the future advisory classifier shape,
- Head A / Head B / Head C separation,
- model comparison direction,
- data thresholds for experiments,
- invariants preventing classifier output from becoming ungrounded truth.

Use this doc when asking:

- whether classifier help belongs in phase 1,
- how many heads there should be,
- what models are reasonable future experiments,
- when classifier work is allowed to start,
- what safety boundary classifier output must obey.

### 4. Migration and integration

File:

- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md`

Owns:

- repo landing order,
- sidecar coexistence plan,
- `comparison_key` / `objective_key` migration role,
- downstream seam order,
- sidecar presence guards,
- migration guardrails and risks.

Use this doc when asking:

- what lands first in the repo,
- which file seams migrate first,
- how `ObjectiveSummary` and `TaskFrame` should coexist,
- when `context/working_set.rs` should move,
- when `checkpoint/progress.rs` is allowed to consume structured state.

## Locked Cross-Doc Decisions

The following decisions should be treated as adopted across the whole design stack:

1. this is an **architecture problem**, not just a missing-heuristic bug,
2. the repo should add a **structured sidecar first** rather than replacing every string consumer
   immediately,
3. compatibility text is a **view**, not the semantic authority,
4. phase 1 should be **deterministic, dependency-free, and evidence-grounded**,
5. `comparison_key` / `objective_key` should become the preferred internal comparison bridge
   before richer structured comparison fully lands,
6. tool-choice instructions belong in execution metadata unless they change semantic task meaning,
7. weak evidence should produce **unknowns**, not fabricated structured fields,
8. no classifier or LLM should be added to analyzer-time objective assembly before the acceptance
   wall exists,
9. `R5.75-1` objective condensation is a **stopgap front-end filter**, not the target
   architecture.

## Recommended Reading Order

For a fresh agent or reviewer, the intended reading order is:

1. this map doc,
2. architecture,
3. evaluation and annotation,
4. migration and integration,
5. classifier taxonomy.

Reason:

- architecture defines the target,
- evaluation defines the proof wall,
- migration defines how the target lands safely,
- classifier taxonomy is future-facing and should not be mistaken for the immediate plan.

## Authority And Precedence

If two docs appear to disagree, use this precedence table.

### For semantic model questions

Canonical authority:

- `DESIGN-r5-structured-objective-architecture.md`

Examples:

- what fields exist,
- what target means,
- what section/role/evidence must be preserved,
- what counts as semantic state.

### For benchmark and acceptance questions

Canonical authority:

- `DESIGN-r5-structured-objective-evaluation-and-annotation.md`

Examples:

- fixture layout,
- allowed compatibility-rendering variance,
- forbidden promotions,
- scorer output requirements.

### For classifier questions

Canonical authority:

- `DESIGN-r5-structured-objective-classifier-taxonomy.md`

Examples:

- head split,
- model candidates,
- minimum dataset thresholds,
- classifier guardrails.

### For repo landing-order and coexistence questions

Canonical authority:

- `DESIGN-r5-structured-objective-migration-and-integration.md`

Examples:

- whether `working_set` or `progress` moves first,
- when `TaskFrame` gains `objective_key`,
- what must be guarded when sidecar data is missing.

### Role of this map doc

This map doc wins only for:

- top-down routing,
- document purpose summary,
- reading order,
- cross-doc overview.

It should not silently override detailed rules from the underlying design docs.

## Implementation Sequence Overview

The intended landing order, as currently designed, is:

### Phase 0: doc and acceptance lock

- architecture is stable enough to implement against,
- evaluation wall is explicit,
- migration order is explicit,
- classifier work remains deferred.

### Phase 1A: additive schema bridge

- add `structured: Option<StructuredObjective>` to `ObjectiveSummary`,
- add `comparison_key`,
- preserve `text` as compatibility display.

### Phase 1B: deterministic section/clause/evidence extraction

- introduce section-aware and clause-aware decomposition,
- preserve `source_kind`, `section_kind`, and evidence spans,
- allow unknown fields when support is weak.

### Phase 1C: compatibility rendering plus acceptance fixtures

- render compatibility text from structured state when safe,
- validate against the objective acceptance wall,
- stop treating exact pretty-string equality as the only test oracle.

### Phase 2: TaskFrame coexistence

- add `structured_objective` and `objective_key`,
- preserve public display behavior during migration,
- start moving internal comparison off raw strings.

### Phase 3: downstream consumer migration

Move in this order:

1. `context/working_set.rs`
2. `checkpoint/mod.rs`
3. `checkpoint/progress.rs`

This order is deliberate because path attribution is the first high-value seam, while progress
comparability is one of the highest-risk downstream consumers.

### Phase 4: optional classifier augmentation

Only after the heuristic structured pipeline and acceptance wall are both stable:

- add advisory classifier support,
- compare against heuristic baseline,
- keep grounded assembly as the final authority.

## What This Stack Is Trying To Prevent

Across all four docs, the main failure classes being prevented are:

1. subordinate checklist or boilerplate lines becoming canonical task truth,
2. tool-choice metadata being mistaken for semantic mission,
3. fabricated fields filling weakly supported slots,
4. compatibility display text remaining the hidden authority forever,
5. classifier experiments starting before the repo can even score the deterministic baseline.

## Open Questions That Still Remain Legitimate

The map does not close the remaining open questions, but it does bound them:

1. the smallest sufficient deterministic clause-segmentation helper,
2. how aggressive early `comparison_key` normalization should be,
3. which exact predicate family after `working_set` should migrate first once coexistence is
   stable,
4. how much classifier help, if any, proves worthwhile after the acceptance wall exists.

## Non-Goals

This map doc should not become:

- a second architecture spec,
- a second migration spec,
- a benchmark fixture schema,
- a classifier proposal in disguise,
- a packet plan or implementation checklist.

If any of those need refinement, update the canonical underlying doc instead.

## Source Map

Primary local authority summarized here:

- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-architecture.md`
- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md`
- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-classifier-taxonomy.md`
- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md`
- `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/.codex/handoffs/2026-06-13-objective-architecture-decision-dossier.md`
