Status: synthesis
Scope: effort+exec
Authority: non-canonical
Artifact-boundary impact: possible

# Parallel Planning And Conflict

This note distills the planning and conflict-management lessons that cut across `effort` and `exec`.

Primary inputs:

- [planner-safe-parallel-lanes.md](../../raw/2026-06-04/planner-safe-parallel-lanes.md)
- [merge-conflict-prediction.md](../../raw/2026-06-04/merge-conflict-prediction.md)
- [deterministic-planning-uncertainty.md](../../raw/2026-06-04/deterministic-planning-uncertainty.md)
- [hf-papers-modern-reference-addendum.md](./hf-papers-modern-reference-addendum.md)

## Durable takeaways

1. Safe parallelism requires both a dependency graph and a conflict graph.
2. Conflict classes should distinguish textual, structural, and semantic interference.
3. Low-confidence or missing signals should reduce parallelism rather than merely lower an aggregate score.
4. Cheap conflict predictors should gate expensive ones rather than replace them.
5. Final safety should be judged at merged-tree time even when lane-local checks stay green.

## Implications for `effort`

- plan deterministically from explicit edges and risk flags
- mark low-confidence shared surfaces as non-parallelizable by default
- partition lanes only after hard conflict elimination and dependency ordering
- emit ownership, forbidden-surface, and freeze information in lane outputs

## Implications for `exec`

- do not silently replan when runtime evidence contradicts the plan
- preserve blocked-condition evidence when hidden coupling appears
- add speculative merge/build/test validation later as a stronger verification layer

## Boundary caution

This note supports embedded ownership/freeze semantics inside planning artifacts.
It does not by itself justify promoting `OwnershipMapV1`, `FreezeMapV1`, or a standalone `ParallelWindowV1` to canonical top-level artifacts yet.

## Modern reference note

For newer repository-planning and graph-oriented references, see:

- [hf-papers-modern-reference-addendum.md](./hf-papers-modern-reference-addendum.md)
