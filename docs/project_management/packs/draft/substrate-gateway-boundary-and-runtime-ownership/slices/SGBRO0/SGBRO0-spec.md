# SGBRO0-spec - lock canonical slice order and planning-support surfaces

## Behavior delta (single)
- Existing: the pack has partial planning support artifacts, but the canonical slice order and task-graph support paths are not all enforced from one execution-ready slice spec.
- New: `SGBRO0` becomes the authoritative boundary-order slice that keeps the accepted `SGBRO0`..`SGBRO4` order and its prompt/spec support paths coherent across the planning pack.
- Why: the rest of the planning graph cannot stay deterministic if the first slice does not pin the shared ordering and support-artifact rules.

## Scope
- Lock the accepted slice ordering in the planning pack.
- Keep the task graph, spec manifest, checkpoint plan, and kickoff prompt paths aligned.
- Do not restate or alter contract, policy, runtime, or docs-validation behavior from later slices.

## Behavior (authoritative)

### Canonical slice-order lock
- `SGBRO0` keeps the accepted order `SGBRO0`, `SGBRO1`, `SGBRO2`, `SGBRO3`, `SGBRO4` consistent across the planning pack.
- The slice keeps populated task entries pointed at real support-artifact paths.
- The slice does not introduce or move the checkpoint boundary.

## Acceptance criteria
- AC-SGBRO0-01: the planning pack names `SGBRO0` through `SGBRO4` as the accepted slice order.
- AC-SGBRO0-02: the planning pack includes a real kickoff prompt path for every populated task.
- AC-SGBRO0-03: the slice-order and checkpoint-bounding docs agree on `CP1` after `SGBRO4`.

## Out of scope
- Status-schema details.
- Policy-evaluation details.
- Runtime-parity details.
- Docs-validation closeout evidence.
