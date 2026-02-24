# Work Lift v1 — seam extraction (WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md)

Source: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` and `.codex/handoffs/2026-02-22-175724-workstream-triage-lift.md`

This directory contains **seam** artifacts extracted to make the Work Lift v1 rollout owner-assignable and parallelizable without hiding coupling.

## Normativity / source of truth (read this first)

This folder has a specific role:

- **Before Work Lift v1 is implemented**: `work_lift_v1_seams/` is the source of truth for **what to implement** (deliverables, contract specs, acceptance criteria, and sequencing). Implementers MUST follow the threaded slices under `threaded-seams/**/slice-*.md`.
- **After Work Lift v1 is implemented**: `docs/project_management/system/*` becomes the source of truth for **what the system is** (published schemas/configs/scripts/standards and their semantics). At that point, `work_lift_v1_seams/` is historical planning context and MUST NOT be treated as the canonical behavioral spec.

Conflict rule during implementation:

- If a statement in `work_lift_v1_seams/` conflicts with `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`, the decision log wins.
- If two files in `work_lift_v1_seams/` conflict, `threading.md` wins for contract IDs/paths, and threaded slices win for execution/acceptance criteria.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading (contracts + dependencies + workstreams): `threading.md`
- Execution plan (threaded decompositions): `threaded-seams/README.md`

## Contract artifact map (cutover targets)

These paths are the intended *published* homes for the contracts after implementation lands:

- `CONTRACT-1:work_lift_vector_block_v1` → `docs/project_management/system/schemas/work_lift_vector.schema.json`
- `CONTRACT-2:work_lift_model_v1` → `docs/project_management/system/schemas/work_lift_model.v1.json`
- `CONTRACT-3:pm_lift_emit_json_v1` → `docs/project_management/system/scripts/planning/pm_lift.py` (`--emit-json` output)
- `CONTRACT-4:impact_map_emit_json_v1` → `docs/project_management/system/scripts/planning/validate_impact_map.py` (`--emit-json` output)

## Reading order (canonical navigation)

1. `scope_brief.md` — what “Work Lift v1” is and what is explicitly out-of-scope.
2. `seam_map.md` — what seams exist and why they were cut.
3. `threading.md` — canonical contract IDs + dependency edges + workstreams.
4. `threaded-seams/README.md` — per-seam `seam.md` + `slice-*.md` files (the executable slice/task breakdowns).

Notes:
- The **threaded seam slices** (`threaded-seams/**/slice-*.md`) are the canonical execution plan artifacts for implementers.
- When a doc references a `CONTRACT-*`, treat `threading.md` as the canonical registry for the exact artifact path and the producing/consuming seams.
