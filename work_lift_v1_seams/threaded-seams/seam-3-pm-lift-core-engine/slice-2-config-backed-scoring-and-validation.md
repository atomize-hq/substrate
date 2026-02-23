### S2 — Config-backed scoring + schema validation (embedded vectors + derived packs)

- **User/system value**: Make `pm_lift` deterministic and inspectable with scoring driven by CONTRACT-2 (auditable config) and validation driven by CONTRACT-1 (authoritative vector schema), while supporting derived-pack inputs (CONTRACT-4).
- **Scope (in/out)**:
  - In:
    - Load and apply `work_lift_model.v1.json` (CONTRACT-2) so weights/triggers/mapping/confidence are config-backed in steady state.
    - Validate embedded Lift Vectors against `work_lift_vector.schema.json` (CONTRACT-1) when present; otherwise fail with actionable guidance or perform conservative structural validation.
    - Ensure missing inputs (`null`) degrade confidence and emit `missing_inputs:*` triggers without crashing.
    - Ensure pack-derived inputs (impact map emit JSON; CONTRACT-4) propagate “prefix entry” signals into confidence/triggers consistently.
  - Out:
    - Changing the model semantics in code (model semantics MUST live in CONTRACT-2 data).
    - Adding enforcement gates; this remains advisory-first.
- **Acceptance criteria**:
  - When `docs/project_management/system/schemas/work_lift_model.v1.json` is present, `pm_lift` uses it (and does not rely on baked-in constants for v1).
  - Intake/ADR mode:
    - detects missing markers, invalid JSON, wrong types with actionable errors,
    - validates against schema (when present) and points to the schema path on failure.
  - Derived-pack mode:
    - consumes `validate_impact_map.py --emit-json` output (CONTRACT-4) robustly,
    - downgrades confidence and/or emits triggers when prefix entries are present (per model config rules).
  - `derived` includes debuggable intermediate computations:
    - base points by category,
    - applied multipliers,
    - mapping from lift score → estimated_slices,
    - confidence degradation reasons (machine-friendly).
- **Dependencies**:
  - Consumes:
    - `CONTRACT-1:work_lift_vector_block_v1` (SEAM-1)
    - `CONTRACT-2:work_lift_model_v1` (SEAM-2)
    - `CONTRACT-4:impact_map_emit_json_v1` (SEAM-4)
  - Must not proceed to “steady-state” acceptance without SEAM-1/SEAM-2 artifacts being present.
- **Verification**:
  - Run `pm_lift.py` with:
    - a valid embedded vector fixture (schema-valid) and assert stable JSON + expected triggers/confidence,
    - an invalid vector fixture and assert error class + actionable guidance,
    - a derived-pack fixture (mocked validator output) and assert prefix-driven confidence/triggers.
- **Rollout/safety**:
  - Advisory vs strict is explicit and deterministic:
    - `pm_lift.py` is an advisory tool; it MUST compute outputs whenever it can, and it MUST represent unknowns via `missing_inputs` + `confidence=low` (CONTRACT-3).
    - The strict posture (non-advisory failures) is implemented by a wrapper script (SEAM-5 S3) that consumes `pm_lift.py --emit-json`.
  - Missing-artifact behavior (v1):
    - If `docs/project_management/system/schemas/work_lift_model.v1.json` is missing, `pm_lift.py` MUST use the baked-in D7 constants and still emit CONTRACT-3 output.
    - If `docs/project_management/system/schemas/work_lift_vector.schema.json` is missing, `pm_lift.py` MUST still parse and type-check the embedded vector (markers + JSON object + section type checks); schema-backed validation is skipped.
    - In strict wrapper mode, missing required artifacts MUST be treated as an error with non-zero exit and actionable stderr (exact exit codes/messages are defined in SEAM-5 S3).

#### S2.T1 — Implement model config application (CONTRACT-2) end-to-end

- **Outcome**: Scoring and trigger/confidence rules are driven by `work_lift_model.v1.json` rather than hard-coded constants.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/schemas/work_lift_model.v1.json` (from SEAM-2)
    - CONTRACT-2 selection semantics (explicit; no “latest” scanning)
  - Outputs:
    - Updated `pm_lift.py` scoring pipeline that:
      - selects the pinned model version deterministically,
      - applies weights/multipliers/thresholds/mapping/confidence rules from config,
      - includes config-driven intermediate values in `derived`.
- **Implementation notes**:
  - Keep selection semantics aligned with SEAM-2:
    - default pin to v1 if absent, but never “pick latest file”.
  - Prefer a two-phase compute:
    - phase A: normalize inputs (including `null` handling),
    - phase B: apply config tables to compute base points, multipliers, triggers, estimated_slices, confidence.
  - Ensure config parsing failures are actionable (path + reason).
- **Acceptance criteria**:
  - With config present, altering a weight in the config changes results without code edits.
  - `derived` includes enough to audit config application (no hidden math).
- **Test notes**:
  - Golden cases in S3 MUST pin expected results for a known config + vector set.
- **Risk/rollback notes**:
  - If config shape is still evolving, keep a temporary backward-compat parser layer, but do not fork semantics.

Checklist:
- Implement:
  - Wire scoring constants to config lookups.
  - Encode confidence degradation and triggers per config.
- Test:
  - Run at least one fixture with config present and observe changed outputs when a constant changes.
- Validate:
  - Confirm selection semantics match SEAM-2 (no implicit “latest”).
- Cleanup:
  - Remove or clearly fence baked-in defaults as fallback-only.

#### S2.T2 — Add schema-backed validation for embedded Lift Vector blocks (CONTRACT-1)

- **Outcome**: Intake/ADR lift vectors are validated against the authoritative schema with actionable errors, and “wrong type / missing required field” issues are caught early.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/schemas/work_lift_vector.schema.json` (from SEAM-1)
    - Intake/ADR markdown containing Lift Vector block markers
  - Outputs:
    - `pm_lift.py` validates embedded vector JSON against schema (when present), or performs conservative structural validation with clear instructions when schema is missing.
- **Implementation notes**:
  - If adding a dependency like `jsonschema` is undesirable, implement minimal validation for required fields/types aligned with CONTRACT-1 (but do not diverge from schema semantics).
  - Error messages MUST point to:
    - missing markers / missing fenced block,
    - JSON parse error location,
    - schema violation paths (json pointer-like paths).
- **Acceptance criteria**:
  - Invalid vectors fail fast with actionable errors.
  - `null` numeric inputs remain allowed and do not crash; they degrade confidence + emit missing_inputs triggers (per seam brief).
- **Test notes**:
  - S3 negative fixtures MUST cover: wrong types, missing required keys, invalid JSON, missing markers.
- **Risk/rollback notes**:
  - Avoid making validation stricter than CONTRACT-1 without a strict mode toggle (advisory-first).

Checklist:
- Implement:
  - Load schema by path; validate embedded vector JSON.
  - Improve error messaging paths.
- Test:
  - Run against one valid and one invalid fixture.
- Validate:
  - Confirm `null` is accepted where schema allows it.
- Cleanup:
  - Keep schema path resolution deterministic (repo-root relative).

#### S2.T3 — Make derived-pack input handling (CONTRACT-4) explicit and confidence-safe

- **Outcome**: `from-impact-map` mode consumes `validate_impact_map.py --emit-json` robustly and emits consistent confidence/triggers when prefix entries exist.
- **Inputs/outputs**:
  - Inputs:
    - `validate_impact_map.py --emit-json` output (CONTRACT-4)
    - Model config (CONTRACT-2) for any prefix-entry confidence/triggers policy
  - Outputs:
    - `pm_lift.py`:
      - maps derived signals into the Lift Vector inputs consistently,
      - records prefix-entry signal(s) in `derived`,
      - downgrades confidence / emits triggers per config.
- **Implementation notes**:
  - Keep “lift-only” prefix expansion policy explicit:
    - raw counts for vector compatibility,
    - effective counts for scoring (if model config requires it),
    - surface both in `derived`.
  - Ensure validator warnings stay on stderr and stdout remains JSON-only in `--emit-json` mode.
- **Acceptance criteria**:
  - When prefix entries exist, confidence is downgraded deterministically and a machine-friendly trigger is emitted.
  - `derived` contains enough details to explain how prefixes affected scoring.
- **Test notes**:
  - Prefer fixtures that stub validator output rather than requiring a full Planning Pack on disk.
- **Risk/rollback notes**:
  - Do not mutate/“rewrite” pack artifacts; compute-only.

Checklist:
- Implement:
  - Map CONTRACT-4 fields into derived + confidence/triggers.
- Test:
  - Fixture with and without prefix entries.
- Validate:
  - Confirm stdout remains JSON-only for `--emit-json`.
- Cleanup:
  - Keep derived fields stable and additive.
