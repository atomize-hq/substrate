Status: synthesis
Scope: effort
Authority: non-canonical
Artifact-boundary impact: possible

# Effort Implications

This note records the strongest planner-specific implications from the current research set.

## Immediate implications

1. `LiftEffortSignalV1` should include confidence, missing-input, and conflict-risk information alongside touched surfaces and evidence refs.
2. `effort` should explicitly reason over both dependency edges and conflict edges.
3. Lane extraction should be conservative by default and should serialize low-confidence shared surfaces.
4. Ownership, forbidden-surface, and freeze semantics should appear in `LanePlanV1` and `HandoffPacketV1` even if top-level maps remain provisional.
5. Planner outputs should carry enough summary information for later runtime validation without requiring deep Lift imports.

## MVP design bias

Preferred early planner order:

1. normalize candidate surfaces
2. build work nodes
3. add hard dependencies
4. add hard conflicts
5. classify medium and soft conflicts
6. suppress unsafe parallelism
7. emit deterministic lane outputs

## Promotion caution

This note supports stronger content inside existing planning artifacts.
It does not require promoting new standalone top-level artifact families yet.
