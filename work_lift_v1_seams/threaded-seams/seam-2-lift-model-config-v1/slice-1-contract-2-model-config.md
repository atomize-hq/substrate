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
  - Bootstrap: tools MUST fall back to baked-in defaults only when the config file is missing (documented behavior); strict mode requires config presence (gated downstream).

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

##### CONTRACT-2: `work_lift_model.v1.json` concrete shape (v1, normative)

For v1, `docs/project_management/system/schemas/work_lift_model.v1.json` MUST be a single JSON object with this shape:

```json
{
  "model_version": 1,
  "summary": "Short human-readable summary",
  "sources": {
    "decision_log": "WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md",
    "scoring": "D7",
    "split_triggers": "D8",
    "lift_to_slices": "D9"
  },
  "selection": {
    "supported_model_versions": [1],
    "default_model_version": 1,
    "vector_model_version_policy": "require_equal",
    "unknown_version_behavior": "error"
  },
  "weights": {
    "touch": {
      "create_files": 3,
      "edit_files": 2,
      "delete_files": 1,
      "deprecate_files": 1,
      "crates_touched": 4,
      "boundary_crossings": 3
    },
    "contract": {
      "cli_flags": 3,
      "config_keys": 3,
      "exit_codes": 4,
      "file_formats": 5,
      "behavior_deltas_blowup": 10
    },
    "qa": {
      "new_test_files": 2,
      "new_test_cases": 1
    },
    "docs": {
      "new_docs_files": 2
    },
    "ops": {
      "new_smoke_steps": 3,
      "ci_changes": 3
    }
  },
  "risk_multipliers": {
    "cross_platform": 1.15,
    "security_sensitive": 1.2,
    "concurrency_or_ordering": 1.15,
    "migration_or_backfill": 1.25
  },
  "unknowns_add": {
    "unknowns_high_multiplier": 2
  },
  "rounding": {
    "score": "ceil"
  },
  "estimated_slices": {
    "divisor": 12,
    "min": 1,
    "rounding": "ceil"
  },
  "confidence": {
    "enum": ["high", "low"],
    "rules": [
      { "when": "missing_inputs_nonempty", "confidence": "low" },
      { "when": "touch_set_contains_prefix_entries", "confidence": "low" },
      { "when": "otherwise", "confidence": "high" }
    ]
  },
  "split_triggers": {
    "adr_candidate": [
      { "id": "split_required:behavior_deltas>1", "when": "contract.behavior_deltas > 1" },
      { "id": "likely_split:crates_touched>2", "when": "touch.crates_touched > 2" },
      { "id": "likely_split:touch_files_sum>12", "when": "touch.create_files + touch.edit_files + touch.delete_files > 12" },
      { "id": "likely_split:contract_surface_sum>4", "when": "contract.cli_flags + contract.config_keys + contract.exit_codes + contract.file_formats > 4" },
      { "id": "likely_split:lift_score>24", "when": "lift_score > 24" },
      { "id": "split_required:estimated_slices>3", "when": "estimated_slices > 3" }
    ],
    "workstream": [
      { "id": "likely_split:lift_score>60", "when": "lift_score > 60" }
    ]
  },
  "prefix_expansion": {
    "enabled_by_default": true,
    "expand_discount": 0.2,
    "expand_cap": 10
  }
}
```

Interpretation requirements:

- All numeric Lift Vector inputs that are missing or `null` MUST be treated as `0` for scoring and MUST generate missing-input outputs per CONTRACT-1/CONTRACT-3.
- `behavior_deltas_blowup` MUST apply as: `+ behavior_deltas_blowup * max(0, contract.behavior_deltas - 1)` (D7).
- `vector_model_version_policy = require_equal` means:
  - if the input vector includes `model_version`, it MUST equal `selection.default_model_version` for v1 (i.e., `1`);
  - otherwise the tool resolves the model version to `1`.
- `unknown_version_behavior = error` means: if a caller selects a model version other than `1` in v1, the tool MUST exit non-zero with actionable stderr (CONTRACT-3 exit code taxonomy).

Prefix expansion math (normative, v1):

- For a single prefix entry, compute `expanded_files` as the number of repo files returned by `git ls-files <prefix>`.
- The effective contribution of that prefix to lift scoring is:
  - `min(expanded_files, expand_cap) * expand_discount`
- **Acceptance criteria**:
  - JSON structure is readable and grouped by concept.
  - No field requires code to interpret precedence ambiguously (ordering or precedence is explicit in JSON).
  - Field names align with `CONTRACT-1` schema keys (no ad-hoc renames).
- **Test notes**:
  - JSON parses cleanly; schema validation for the config itself can be added later, but v1 MUST remain simple enough to review manually.
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
    - (Optional supporting doc in S2) describing how tools select versions.
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
