# SEAM-2 — Lift model config v1 (weights/triggers/versioning) (threaded decomposition)

## Seam Brief (Restated)

- **Seam ID**: SEAM-2
- **Name**: Lift model config v1
- **Goal / value**: Make Lift Score computation tunable and inspectable via a single versioned config file, so scoring changes don’t require code edits and can be reviewed like policy.
- **Type**: integration
- **Scope**
  - In:
    - Create `docs/project_management/system/schemas/work_lift_model.v1.json` defining:
      - weights for base points,
      - risk multipliers,
      - slice mapping constants,
      - split trigger thresholds,
      - confidence rules related to missing inputs and prefix entries.
    - Explicit version-selection semantics (how tools choose v1 vs future v2).
  - Out:
    - Multiple competing model files or dynamic “latest” selection without explicit version pinning.
- **Touch surface**:
  - New: `docs/project_management/system/schemas/work_lift_model.v1.json`
  - Reference-only input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9 canonical v1 rules)
- **Verification**:
  - Unit-style validation by running `pm_lift --emit-json` with known vectors and confirming score/triggers match expected values.
  - A “golden” example (vector + expected outputs) that can be used in docs/tests to detect drift.
- **Threading constraints**
  - Upstream blockers:
    - SEAM-1 (`CONTRACT-1:work_lift_vector_block_v1`) insofar as model config rule keys must reference schema-defined Lift Vector fields consistently.
  - Downstream blocked seams: SEAM-3, SEAM-5
  - Contracts produced (owned):
    - `CONTRACT-2:work_lift_model_v1` (versioned model config JSON + explicit selection semantics)
  - Contracts consumed:
    - `CONTRACT-1:work_lift_vector_block_v1` (field names/types + null semantics for “unknown” inputs)

## Slice index

- `S1` → `slice-1-contract-2-model-config.md`: publish CONTRACT-2 as an auditable, immutable v1 config with explicit version-selection semantics
- `S2` → `slice-2-goldens-and-conformance.md`: add golden cases + conformance guardrails so reviewers and downstream seams can detect drift without reverse-engineering the formula

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `CONTRACT-2:work_lift_model_v1`
    - Definition: `docs/project_management/system/schemas/work_lift_model.v1.json` containing weights/multipliers/triggers/mapping constants/confidence rules, plus explicit selection semantics (no dynamic “latest”).
    - Produced by: S1 (config contract) and S2 (golden cases + audit/conformance material).
- **Contracts consumed**:
  - `CONTRACT-1:work_lift_vector_block_v1`
    - Consumed by: S1/S2 to ensure config keys and golden vectors match schema field names and `null`-allowed semantics.
- **Dependency edges honored**:
  - `SEAM-2 blocks SEAM-3`: S1 must land before `pm_lift` can be config-backed instead of hard-coded.
  - `SEAM-2 feeds SEAM-5`: S1/S2 provide a stable scoring source-of-truth for advisory workflow integration (without requiring code changes for tuning).
- **Parallelization notes**:
  - What can proceed now:
    - All work in this seam can land independently (new files under `docs/project_management/system/schemas/` and optional supporting docs under `docs/project_management/system/standards/shared/`).
  - What must wait:
    - SEAM-3/SEAM-5 MUST NOT rely on the config semantics until S1 lands. S2 goldens are required before any strict-mode enablement work.
