# SEAM-3 — `pm_lift` core engine + stable output contract (threaded decomposition)

## Seam Brief (Restated)

- **Seam ID**: SEAM-3
- **Name**: `pm_lift` (core computation + outputs)
- **Goal / user value**: Provide a deterministic, inspectable CLI tool that computes Work Lift from either (a) an embedded Lift Vector block or (b) derived artifacts, and emits stable JSON for downstream tooling.
- **Type**: capability
- **Scope**
  - In:
    - Load and apply `docs/project_management/system/schemas/work_lift_model.v1.json` (CONTRACT-2) for scoring (steady-state: no hard-coded weights/thresholds).
    - Validate embedded lift vectors against `docs/project_management/system/schemas/work_lift_vector.schema.json` (CONTRACT-1), or at minimum validate structure/types with actionable errors.
    - Emit stable JSON via `--emit-json` (CONTRACT-3) suitable for downstream workflow integration and future lint/enforcement tooling.
    - Confidence/triggers rules consistent with the decision log (missing inputs, directory/prefix entries).
  - Out:
    - Rewriting inputs (frontmatter updates) unless explicitly introduced later with idempotent rules.
    - Enforcement gates that break legacy packs.
- **Touch surface**:
  - `docs/project_management/system/scripts/planning/pm_lift.py`
  - Optional: helper modules colocated with `pm_lift.py`
  - Tests: `docs/project_management/system/scripts/planning/tests/`
- **Verification**:
  - Golden tests that run `pm_lift.py` and assert score/triggers/confidence and JSON output shape stability.
  - Negative cases: invalid/missing Lift Vector block, wrong types, schema violations, missing/invalid config.
- **Threading constraints**
  - Upstream blockers:
    - SEAM-1 (`CONTRACT-1:work_lift_vector_block_v1`) for authoritative Lift Vector schema/rubric.
    - SEAM-2 (`CONTRACT-2:work_lift_model_v1`) for config-backed weights/triggers/confidence rules.
  - Downstream blocked seams: SEAM-5
  - Contracts produced (owned):
    - `CONTRACT-3:pm_lift_emit_json_v1` (stable JSON output shape for `pm_lift --emit-json`)
  - Contracts consumed:
    - `CONTRACT-1:work_lift_vector_block_v1` (embedded vector markers + JSON schema)
    - `CONTRACT-2:work_lift_model_v1` (scoring constants + trigger/confidence rules)
    - `CONTRACT-4:impact_map_emit_json_v1` (pack-derived inputs via `validate_impact_map.py --emit-json`)

## Slice index

- `S1` → `slice-1-contract-3-emit-json.md`: publish and lock the `--emit-json` output contract (keys, semantics, exit-code/error expectations) to unblock downstream integration
- `S2` → `slice-2-config-backed-scoring-and-validation.md`: implement config-backed scoring + schema-based validation for embedded vectors and derived-pack inputs (confidence/triggers included)
- `S3` → `slice-3-goldens-and-conformance.md`: add goldens + negative cases + conformance guardrails to prevent drift in both scoring and JSON contract

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `CONTRACT-3:pm_lift_emit_json_v1`
    - Definition: stable JSON keys + semantics for `pm_lift.py ... --emit-json` including `model_version`, `lift_score`, `estimated_slices`, `confidence`, `triggers`, `missing_inputs`, `vector`, `derived` (additive keys only).
    - Produced by: S1 (contract spec + CLI alignment) and protected by S3 (contract conformance tests).
- **Contracts consumed**:
  - `CONTRACT-1:work_lift_vector_block_v1` (SEAM-1)
    - Consumed by: S2 (schema validation and field/type semantics) and S3 (fixtures + negative cases).
  - `CONTRACT-2:work_lift_model_v1` (SEAM-2)
    - Consumed by: S2 (weights/triggers/mapping/confidence sourced from config) and S3 (golden scoring cases).
  - `CONTRACT-4:impact_map_emit_json_v1` (SEAM-4)
    - Consumed by: S2 (derived lift inputs and confidence degradation for prefix entries) and S3 (pack-derived fixture tests).
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-3`: S2/S3 explicitly depend on schema + rubric for authoritative validation and fixtures.
  - `SEAM-2 blocks SEAM-3`: S2 explicitly moves scoring constants/thresholds out of code and into config.
  - `SEAM-3 blocks SEAM-5`: S1 publishes/locks CONTRACT-3 early, so SEAM-5 can integrate without binding to implementation details.
- **Parallelization notes**:
  - What can proceed now:
    - S1 can land without SEAM-1/SEAM-2 being fully complete, as long as it does not hard-code semantics that contradict CONTRACT-1/2 and it keeps validation/scoring conservative (advisory-first).
  - What must wait:
    - S2/S3 MUST NOT claim “steady-state config-backed scoring” until SEAM-2 publishes `work_lift_model.v1.json`, and MUST NOT claim schema-backed validation until SEAM-1 publishes `work_lift_vector.schema.json`.
