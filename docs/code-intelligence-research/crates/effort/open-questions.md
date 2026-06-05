Status: exploratory
Scope: effort
Authority: non-canonical
Artifact-boundary impact: possible

# Effort Open Questions

These questions remain open on purpose and should not be treated as settled boundaries.

## Open questions

1. Which `LiftEffortSignalV1` fields need to be mandatory for MVP versus optional for later refinement?
2. Should ownership and freeze semantics live only in `LanePlanV1`, or also in `WorkGraphV1` as reusable upstream summaries?
3. How far should conflict prediction go in MVP before deferring to later runtime validation?
4. When should a low-confidence signal fully block parallelization versus merely constrain merge order?
5. Does `ParallelWindowV1` need to exist as a standalone artifact, or is a `LanePlanV1` substructure sufficient?

## Keep provisional

Do not answer these by directory structure alone.
They should be resolved only when repeated implementation pressure or repeated orchestration patterns justify promotion.
