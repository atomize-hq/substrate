### S2 — Human rubric published (field meaning + null semantics + examples)

- **User/system value**: Make Lift Vector v1 consistently fillable by humans/agents by documenting what each field means, what counts as “unknown”, and how to write a valid block.
- **Scope (in/out)**:
  - In:
    - Publish `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`.
    - Include the marker convention and at least one end-to-end example block that passes schema validation.
    - Include a conformance checklist to prevent schema/rubric drift.
  - Out:
    - Canonical scoring weights/thresholds (these live in `CONTRACT-2:work_lift_model_v1`).
    - Workflow enforcement rules (strict-mode gates are owned by downstream seams).
- **Acceptance criteria**:
  - Rubric exists at `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`.
  - Rubric documents:
    - the marker convention (`PM_LIFT_VECTOR` begin/end comments),
    - the required fenced JSON block,
    - per-field meaning and counting guidance aligned to D6,
    - what `null` means and where it is allowed,
    - at least one complete example block that validates against `work_lift_vector.schema.json`,
    - a “Conformance” checklist enumerating canonical fields + null allowances + marker conventions.
- **Dependencies**:
  - Consumes: `CONTRACT-1:work_lift_vector_block_v1` (produced by S1) for the schema path and canonical field list.
  - Reference-only input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D3/D6).
- **Verification**:
  - Validate the rubric’s end-to-end example against the schema.
  - Spot-check that rubric text doesn’t diverge from schema field names/types.
- **Rollout/safety**:
  - Advisory-first: rubric is guidance; enforcement posture is handled elsewhere.

#### S2.T1 — Author WORK_LIFT_RUBRIC.md aligned to canonical vector

- **Outcome**: A human-readable rubric that defines the Lift Vector v1 contract for authors and reviewers.
- **Inputs/outputs**:
  - Inputs:
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D3 (block format) + D6 (field list)
    - `docs/project_management/system/schemas/work_lift_vector.schema.json` (from S1)
  - Output: `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`
- **Implementation notes**:
  - Include sections:
    - “How to add a Lift Vector block” (markers + fenced JSON)
    - “Field-by-field rubric” grouped by: `touch`, `contract`, `qa`, `docs`, `ops`, `risk`, `notes`
    - “Null / unknown guidance”:
      - explicitly call out `touch.crates_touched` / `touch.boundary_crossings` as `null`-allowed,
      - explain that `null` means “unknown” (discovery-time) and should not be replaced with invented precision.
    - “Example”:
      - include one end-to-end example block matching the schema (can reuse the D3 example).
    - “Conformance checklist” (from S1.T2)
  - Where scoring/mapping is mentioned, reference the model config contract rather than embedding weights:
    - `docs/project_management/system/schemas/work_lift_model.v1.json` (`CONTRACT-2:work_lift_model_v1`)
- **Acceptance criteria**:
  - Field names in the rubric match the schema exactly.
  - Rubric’s example validates against the schema.
- **Test notes**:
  - Validate the embedded example JSON with a JSON Schema validator.
- **Risk/rollback notes**:
  - Keep rubric language non-prescriptive where discovery-time uncertainty is expected; avoid turning guidance into unintended hard requirements.

Checklist:
- Implement:
  - Draft rubric sections and cross-link to schema + model config location.
  - Add end-to-end example block.
  - Add conformance checklist.
- Test:
  - Validate the example JSON against the schema.
- Validate:
  - Ensure rubric doesn’t define the scoring weights (leave that to CONTRACT-2).
- Cleanup:
  - Ensure references and paths are correct and stable.

#### S2.T2 — Conformance pass: schema/rubric/example stay aligned

- **Outcome**: A quick, repeatable manual review path that prevents drift at PR time.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_vector.schema.json`
    - `WORK_LIFT_RUBRIC.md`
    - D6 field list
  - Outputs:
    - Rubric “Conformance checklist” verified and updated if needed.
    - A short note in the rubric about how to validate examples (tool-agnostic).
- **Implementation notes**:
  - Checklist should include a mechanical comparison:
    - “Rubric field list == schema properties”
    - “Null allowances == schema union types”
    - “Markers == CONTRACT-1 definition”
- **Acceptance criteria**:
  - A reviewer can verify conformance without reading any other documents.
- **Test notes**:
  - N/A beyond example validation.
- **Risk/rollback notes**:
  - None.

Checklist:
- Implement:
  - Add/verify the conformance checklist items.
  - Add/verify a brief “How to validate the example” note.
- Test:
  - Re-validate example after any edits.
- Validate:
  - Confirm no new coupling to other seams’ touch surfaces.
- Cleanup:
  - None.

