### S2 — Golden cases + conformance guardrails for v1

- **User/system value**: Provide a small set of verifiable golden cases and an audit checklist so future edits can detect drift without reverse-engineering the scoring model.
- **Scope (in/out)**:
  - In:
    - Publish a “golden cases” artifact (vector + expected outputs) that downstream seams can use for tests/docs.
    - Publish a conformance checklist that makes v1 immutability and traceability enforceable at review time.
  - Out:
    - Wiring goldens into `pm_lift.py` tests (owned by SEAM-3).
    - Editing `WORK_LIFT_RUBRIC.md` to embed a golden example (owned by SEAM-1); this seam produces goldens that SEAM-1 can embed.
- **Acceptance criteria**:
  - At least one golden case exists that specifies:
    - a minimal Lift Vector (v1),
    - any derived signals needed for confidence/prefix rules,
    - expected `lift_score`, `estimated_slices`, `confidence`, and `triggers`.
  - Golden cases are small enough to be manually verified.
  - Conformance checklist exists and includes:
    - “v1 immutable → changes require v2 file”,
    - “constants trace back to D7–D9”,
    - “field names align with CONTRACT-1 schema”,
    - “selection semantics forbid ‘latest’”.
- **Dependencies**:
  - Consumes: `CONTRACT-1:work_lift_vector_block_v1` (vector shape + null semantics)
  - Consumes: `CONTRACT-2:work_lift_model_v1` (config constants + selection semantics)
  - Reference-only input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9)
- **Verification**:
  - Manual calculation check for each golden:
    - compute score/triggers/slices/confidence directly from the config constants.
  - Downstream verification (owned by SEAM-3):
    - run `pm_lift --emit-json` with the golden vector and confirm exact match.
- **Rollout/safety**:
  - Advisory-first: goldens are used for regression checks and examples; enforcement posture remains gated by strict mode downstream.

#### S2.T1 — Publish v1 golden cases (vector + expected outputs)

- **Outcome**: A compact set of golden cases that downstream seams can consume for tests/docs.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/schemas/work_lift_vector.schema.json` (SEAM-1)
    - `docs/project_management/system/schemas/work_lift_model.v1.json` (S1)
  - Outputs (choose one, keep it simple):
    - Option A (single file): `docs/project_management/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md`
    - Option B (machine-readable + doc): `docs/project_management/system/standards/shared/work_lift_model_v1_goldens.json` + a short README section pointing to it
- **Implementation notes**:
  - Include at least one “small vector” golden that exercises:
    - base weights,
    - at least one risk multiplier,
    - at least one split trigger threshold,
    - a confidence degradation path for missing inputs and/or prefix tokens.
  - Specify expected outputs explicitly as numbers/strings/arrays (no prose-only expectations).
  - Keep the golden vector valid per `CONTRACT-1` (including `model_version: 1`).
- **Acceptance criteria**:
  - Golden vectors validate against the Lift Vector schema.
  - Expected outputs are fully specified (no “approximately”).
  - A reviewer can recompute the result with the config constants.
- **Test notes**:
  - Validate golden vector JSON against the Lift Vector schema.
  - (Optional) validate that the golden output fields match the planned `pm_lift --emit-json` output keys (CONTRACT-3 is owned by SEAM-3; don’t invent new keys here).
- **Risk/rollback notes**:
  - If a golden depends on derived inputs not yet finalized (SEAM-4), keep the first golden limited to vector-only scoring to avoid coupling churn.

Checklist:
- Implement:
  - Pick 1–3 goldens (start with 1).
  - Write input vector(s) and expected output(s).
  - Document any assumptions (e.g., how missing inputs affect confidence).
- Validate:
  - Schema-validate each input vector.
  - Manually recompute once from config constants.
- Cleanup:
  - Keep files short and readable; avoid large scenario matrices.

#### S2.T2 — Publish a v1 conformance checklist (immutability + traceability + selection)

- **Outcome**: A mechanical checklist that reviewers can use to prevent unintentional v1 drift.
- **Inputs/outputs**:
  - Inputs:
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9)
    - `work_lift_model.v1.json` (S1)
    - Contract registry in `work_lift_v1_seams/threading.md`
  - Output:
    - A “Conformance” section in the chosen golden doc (S2.T1), or a short standalone:
      - `docs/project_management/system/standards/shared/WORK_LIFT_MODEL_V1.md`
- **Implementation notes**:
  - Checklist items should be copy/paste-verifiable:
    - “No edits to v1 scoring constants; create v2 instead.”
    - “Selection semantics are explicit and forbid ‘latest’.”
    - “All constants trace to D7–D9 (or explicitly justified as a default).”
    - “Config keys reference schema field names; no ad-hoc renames.”
  - Add a short note on how to validate goldens (tool-agnostic).
- **Acceptance criteria**:
  - Reviewers can detect drift without reading SEAM-3 code.
  - Checklist explicitly names the contracts it is guarding (`CONTRACT-1`, `CONTRACT-2`).
- **Test notes**:
  - N/A (documentation-only), but checklist should be verified once against the shipped files.
- **Risk/rollback notes**:
  - None.

Checklist:
- Implement:
  - Add the checklist and validation note.
  - Cross-link the contract IDs and file paths.
- Validate:
  - Confirm checklist matches the actual selection semantics embedded in `work_lift_model.v1.json`.
  - Confirm it does not restate scoring constants (keep those in the JSON contract).
- Cleanup:
  - Keep the checklist short and stable (future versions add new docs rather than rewriting v1 guidance).

