# SEAM-1 — Lift Vector schema + human rubric (threaded decomposition)

## Seam Brief (Restated)

- **Seam ID**: SEAM-1
- **Name**: Lift Vector v1 schema + rubric
- **Goal / value**: Make Lift Vector v1 fillable/parseable in a deterministic way, with a single human-readable rubric that aligns with the machine schema and avoids “invented precision”.
- **Type**: integration
- **Scope**
  - In:
    - Create `docs/project_management/system/schemas/work_lift_vector.schema.json`.
    - Create `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`.
    - Document the `PM_LIFT_VECTOR` marker convention and the required JSON fenced block.
  - Out:
    - Enforcing presence of lift blocks across all packs/intakes.
    - Auto-rewriting existing docs to insert lift blocks.
- **Touch surface**:
  - New: `docs/project_management/system/schemas/work_lift_vector.schema.json`
  - New: `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`
  - Reference-only input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D3/D6 canonical fields + examples)
- **Verification**:
  - Validate the schema against at least:
    - a “fully specified” vector,
    - a vector with `null` numeric fields (`touch.crates_touched`, `touch.boundary_crossings`),
    - a vector with invalid types (e.g., bool for number) to ensure clear failures.
  - Rubric includes at least one end-to-end example block that passes schema validation.
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: SEAM-3, SEAM-5
  - Contracts produced (owned):
    - `CONTRACT-1:work_lift_vector_block_v1` (markers + embedded JSON block + schema file)
  - Contracts consumed: none (source-of-truth is `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`)

## Slice index

- `S1` → `slice-1-contract-1-schema.md`: publish CONTRACT-1 as a concrete JSON Schema with null semantics + versioning hooks
- `S2` → `slice-2-human-rubric-and-conformance.md`: publish a field-by-field rubric + examples + a conformance checklist to prevent drift

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `CONTRACT-1:work_lift_vector_block_v1`
    - Definition: `<!-- PM_LIFT_VECTOR:BEGIN -->` / `<!-- PM_LIFT_VECTOR:END -->` wrapping a fenced ` ```json { ... } ``` ` object that validates against `docs/project_management/system/schemas/work_lift_vector.schema.json`.
    - Produced by: S1 (schema) and S2 (rubric documents the marker convention + examples).
- **Contracts consumed**:
  - None (this seam is anchored on the decision log’s canonical field list and examples).
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-3`: S1/S2 produce an authoritative schema/rubric so `pm_lift` can validate and explain lift inputs deterministically.
  - (Non-blocking but important) `SEAM-1 feeds SEAM-5`: SEAM-5 references the same contract to integrate advisory workflow.
- **Parallelization notes**:
  - What can proceed now:
    - All work in this seam can land independently (new files only under the seam’s touch surface).
  - What must wait:
    - Nothing for this seam; consumers (SEAM-3/SEAM-5) MUST NOT rely on CONTRACT-1 until S1/S2 land.
