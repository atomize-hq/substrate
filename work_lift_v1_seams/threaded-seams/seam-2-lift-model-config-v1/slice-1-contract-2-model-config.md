### S1 — CONTRACT-2 model config published (weights/triggers/versioning)

- **User/system value**: Establish a single auditable, version-pinned source of truth for scoring weights, triggers, and confidence degradation so tuning does not require code edits.
- **Scope (in/out)**:
  - In:
    - Publish `docs/project_management/system/schemas/work_lift_model.v1.json` as the authoritative v1 model config.
    - Encode the canonical v1 rules from `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9) using only tables/constants (no “mini language”).
    - Define explicit version-selection semantics (no dynamic “latest” resolution).
  - Out:
    - Implementing scoring logic in `pm_lift.py` (owned by SEAM-3).
    - Editing `WORK_LIFT_RUBRIC.md` to embed examples (owned by SEAM-1) — this seam provides goldens separately in S2.
- **Acceptance criteria**:
  - `docs/project_management/system/schemas/work_lift_model.v1.json` exists.
  - Config is human-auditable:
    - keys are named after domain concepts (weights/multipliers/triggers/confidence),
    - values are plain numbers/booleans/strings/arrays/objects (no expressions),
    - rule precedence is explicit (ordered lists or named phases).
  - Config encodes the canonical v1 semantics from D7–D9 (traceable via an explicit mapping section or references).
  - Versioning/selection semantics are explicit:
    - tools select `v1` only via an explicit version pin (default pin is allowed, but must be documented),
    - no “scan directory for latest” behavior is implied.
  - Once published, changes to v1 are treated as exceptional; normal edits introduce `work_lift_model.v2.json` instead.
- **Dependencies**:
  - Consumes: `CONTRACT-1:work_lift_vector_block_v1` (field names + `null` semantics for unknown inputs)
  - Reference-only input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9)
- **Verification**:
  - Manual audit pass:
    - Each config constant maps to a D7–D9 rule (or is explicitly marked as a new default with justification).
    - Config can be read top-to-bottom without needing to infer hidden logic.
  - Downstream verification (owned by SEAM-3): `pm_lift --emit-json` produces expected outputs when driven by the config.
- **Rollout/safety**:
  - Bootstrap: tools may fall back to baked-in defaults only when the config file is missing (documented behavior); strict mode requires config presence (gated downstream).

#### S1.T1 — Define the v1 config structure and invariants

- **Outcome**: A stable JSON structure for the model config that stays auditable and extensible (via new versioned files, not mutations).
- **Inputs/outputs**:
  - Inputs:
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9)
    - `docs/project_management/system/schemas/work_lift_vector.schema.json` (from SEAM-1)
  - Output:
    - `docs/project_management/system/schemas/work_lift_model.v1.json` (structure + metadata stubs)
- **Implementation notes**:
  - Include explicit metadata:
    - `model_version: 1` (integer),
    - a short `summary` string for human readers,
    - a `selection` section that documents selection rules (see S1.T3),
    - a `sources` section that references D7–D9 (ids/anchors as strings).
  - Keep v1 rules to tables/constants only:
    - weights: numeric constants,
    - multipliers: numeric constants keyed by enumerations,
    - triggers: explicit threshold tables,
    - mapping constants: numeric constants,
    - confidence degradation: explicit penalties keyed by “missing input”/“prefix token present” signals.
  - Avoid polymorphic rule graphs or embedded expression strings (de-risk “config becomes a programming language”).
- **Acceptance criteria**:
  - JSON structure is readable and grouped by concept.
  - No field requires code to interpret precedence ambiguously (ordering or precedence is explicit in JSON).
  - Field names align with `CONTRACT-1` schema keys (no ad-hoc renames).
- **Test notes**:
  - JSON parses cleanly; schema validation for the config itself (if desired) can be added later, but v1 should remain simple enough to review manually.
- **Risk/rollback notes**:
  - If config structure starts to grow unreviewable, that is a signal to cut scope (move derived logic into SEAM-3 with stable debug outputs) rather than add more config machinery.

Checklist:
- Implement:
  - Draft the top-level structure and metadata.
  - Add placeholder sections for weights/multipliers/triggers/mapping/confidence.
  - Add a `sources` mapping to D7–D9 (strings; no brittle hard links required).
- Validate:
  - Cross-check names against `work_lift_vector.schema.json`.
  - Confirm rule precedence is explicit.
- Cleanup:
  - Ensure the file path matches the contract definition exactly.

#### S1.T2 — Encode canonical D7–D9 v1 constants (weights, multipliers, thresholds, mapping, confidence)

- **Outcome**: `work_lift_model.v1.json` contains the canonical v1 constants needed to compute score/triggers/slices/confidence deterministically.
- **Inputs/outputs**:
  - Inputs:
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7–D9)
    - `CONTRACT-1:work_lift_vector_block_v1` (field names/types; `null` semantics)
  - Output:
    - Fully-populated `docs/project_management/system/schemas/work_lift_model.v1.json`
- **Implementation notes**:
  - Ensure every constant is traceable to D7–D9:
    - either via the `sources` map or via adjacent “why” fields inside JSON.
  - Confidence rules must explicitly cover:
    - missing inputs (`null` fields; absent optional fields if allowed),
    - presence of directory/prefix tokens (as signaled by derived inputs; see SEAM-4 contract).
  - Keep “derived rules” out of config:
    - if a rule requires computation beyond table lookup (e.g., complex prefix expansion), treat it as code in SEAM-3/SEAM-4 with stable debug outputs.
- **Acceptance criteria**:
  - All required sections are populated: weights/multipliers/triggers/mapping/confidence.
  - No “magic” behavior is implied; everything needed for interpretation is present and explicit.
- **Test notes**:
  - Validate against at least one golden case spec (produced in S2) by manual calculation.
- **Risk/rollback notes**:
  - If D7–D9 rules are ambiguous, stop and record the ambiguity as a follow-up decision (do not silently pick a value).

Checklist:
- Implement:
  - Fill in all v1 constant tables per D7–D9.
  - Add traceability metadata for each major section.
- Validate:
  - Manual audit: “config value ↔ D7–D9 reference” for every constant.
  - Confirm confidence degradation rules cover both missing inputs and prefix tokens.
- Cleanup:
  - Ensure JSON remains readable (stable ordering; no excessive nesting).

#### S1.T3 — Define explicit version-selection semantics (no dynamic “latest”)

- **Outcome**: Documented and config-encoded rules for selecting which model config version applies.
- **Inputs/outputs**:
  - Inputs:
    - Contract registry (`CONTRACT-2:work_lift_model_v1`) in `work_lift_v1_seams/threading.md`
  - Outputs:
    - A `selection` section in `work_lift_model.v1.json` that states the intended selection semantics.
    - (Optional supporting doc in S2) describing how tools should select versions.
- **Implementation notes**:
  - Required semantics:
    - Version is always explicit (either via tool default pin to `1`, or via an explicit CLI flag / input field); never “latest file in directory”.
    - Future `v2` publishes as a new file (`work_lift_model.v2.json`) with explicit selection rules updated in tooling/docs.
    - `v1` is immutable: edits to scoring behavior must introduce a new versioned file.
- **Acceptance criteria**:
  - A reader can answer: “How does the tool decide v1 vs v2?” without guessing.
  - Semantics forbid implicit “latest” behavior.
- **Test notes**:
  - N/A (semantic contract), but must be referenced by downstream tool implementation (SEAM-3).
- **Risk/rollback notes**:
  - If selection semantics are underspecified, downstream implementations will diverge; keep this crisp even if tooling defaults evolve later.

Checklist:
- Implement:
  - Write the selection rules in JSON in plain language.
  - Ensure immutability guidance is explicit.
- Validate:
  - Confirm alignment with the contract registry and seam brief.
- Cleanup:
  - Keep language tool-agnostic (don’t hardcode CLI flag names that may change).

