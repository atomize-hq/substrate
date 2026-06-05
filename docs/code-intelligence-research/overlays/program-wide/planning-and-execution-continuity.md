Status: synthesis
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: possible

# Planning And Execution Continuity

This note records the cross-crate pattern that the same Lift-derived structural
signals can add value twice:

1. during static planning in `effort`
2. during runtime enforcement and sanity checking in `exec`

## Durable takeaways

1. Planning-time and execution-time consumers should not reinvent structural
   intelligence independently.
2. Reuse of structural signals across phases improves operator discipline and
   output quality.
3. Continuity does not erase the ownership split:
   `effort` plans, `exec` runs, and neither takes over Lift's extraction role.

## Planning-time use

The most useful planning-time uses are:

- lane boundary decisions
- dependency and conflict reasoning
- freeze and ownership hints
- confidence-aware suppression of unsafe parallelism
- handoff constraints and likely verification surfaces

## Execution-time reuse

The most useful runtime uses are:

- drift detection against the planning baseline
- sanity checks on touched scope and ownership overlap
- revalidation of stale assumptions before closeout
- richer blocked-condition evidence when reality diverges from the plan

## Relationship to `LiftEffortSignalV1`

The current strongest framing is:

> `LiftEffortSignalV1` should be treated as a reusable structural-evidence
> surface, not only as a one-shot planner blob.

That means the planner can consume it first, while later runtime artifacts may
preserve or reference the same family of signals.

## Multi-layer artifact contract

The strongest repo-local pattern remains a layered handoff shape rather than
one ambiguous planning blob:

1. code-intelligence facts:
   Lift-owned typed entities, typed dependency structure, typed conflict-risk
   structure or conflict evidence, source mappings, confidence or missing-signal
   annotations, and deterministic history-derived features
2. plan and lane structures:
   planner-owned work graph, lane boundaries, ordering, ownership, freeze, and
   blocked-condition intent
3. runtime and handoff structures:
   worker-visible packets, validation surfaces, runtime evidence attachments,
   and blocked or closeout state

This layering keeps explicit-external schemas and artifact-centric handoffs
intact without forcing one crate to absorb the entire flow.

## Boundary caution

This note does not require runtime to import Lift internals or to silently
replan work graphs.
The reuse should happen through stable artifact contracts and referenced
evidence, not crate-boundary collapse.
