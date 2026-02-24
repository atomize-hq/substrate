### S1 — CONTRACT-1 schema published (Lift Vector block + JSON schema)

- **User/system value**: Establish a stable, parseable Lift Vector v1 contract so scripts/tools can validate and explain lift inputs deterministically.
- **Scope (in/out)**:
  - In:
    - Publish `docs/project_management/system/schemas/work_lift_vector.schema.json` matching the canonical Lift Vector v1 fields.
    - Encode `null`-allowed semantics for discovery-time unknowns without forcing invented precision.
  - Out:
    - Model config, scoring weights, or any `pm_lift.py` implementation details (owned by other seams).
- **Acceptance criteria**:
  - The schema exists at `docs/project_management/system/schemas/work_lift_vector.schema.json`.
  - Schema validates:
    - a fully specified Lift Vector object (all counts present, including ints for `touch.crates_touched` / `touch.boundary_crossings`),
    - a discovery-time object with `touch.crates_touched: null` and/or `touch.boundary_crossings: null`.
  - Schema rejects at least one intentionally-invalid example with a clear type mismatch (e.g., `touch.create_files: true`).
  - Schema supports additive evolution (new optional keys can be added without breaking existing vectors).
- **Dependencies**: none
- **Verification**:
  - Use any JSON Schema validator to validate sample objects (e.g., VS Code JSON schema validation, `python -m jsonschema` if available, or `ajv`).
  - Confirm the D3 example block in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` validates against the schema.
- **Rollout/safety**:
  - Advisory-first: schema can be used for validation when the block is present; presence requirements remain gated by strict mode (handled outside this seam).

#### S1.T1 — Define Lift Vector v1 JSON schema

- **Outcome**: A JSON Schema that matches the canonical Lift Vector v1 shape and semantics.
- **Inputs/outputs**:
  - Input: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D3/D6 canonical shape + examples)
  - Output: `docs/project_management/system/schemas/work_lift_vector.schema.json`
- **Implementation notes**:
  - Required top-level keys (v1): `touch`, `contract`, `qa`, `docs`, `ops`, `risk`, `notes`.
  - `model_version` is optional; if missing, consumers/tools default to `1` (and MUST emit the resolved `model_version` in machine outputs).
  - Use `minimum: 0` for count fields where meaningful.
  - Allow discovery-time unknowns by using `integer | null` for numeric count fields; missing and `null` both mean “unknown”.
  - Tools MUST treat missing/`null` as `0` for scoring while:
    - setting `confidence = low`, and
    - emitting `missing_inputs:<json_path>` triggers (and `missing_inputs` array entries) per CONTRACT-3.
  - Schema policy for unknown keys:
    - For Lift Vector v1, schema validation MUST reject unknown keys (`additionalProperties: false`) at the root and inside each section object.
    - Additive evolution happens by publishing a new schema version that adds new optional keys; old vectors continue to validate because the new keys are optional.

##### Contract-1 field inventory (v1, normative)

The schema MUST define exactly these JSON paths (and no others) for v1:

- Root
  - `model_version` (integer, optional; if present MUST equal `1`)
  - `touch` (object, required; `additionalProperties: false`)
    - `create_files` (integer|null; minimum 0 when integer)
    - `edit_files` (integer|null; minimum 0 when integer)
    - `delete_files` (integer|null; minimum 0 when integer)
    - `deprecate_files` (integer|null; minimum 0 when integer)
    - `crates_touched` (integer|null; minimum 0 when integer)
    - `boundary_crossings` (integer|null; minimum 0 when integer)
  - `contract` (object, required; `additionalProperties: false`)
    - `cli_flags` (integer|null; minimum 0 when integer)
    - `config_keys` (integer|null; minimum 0 when integer)
    - `exit_codes` (integer|null; minimum 0 when integer)
    - `file_formats` (integer|null; minimum 0 when integer)
    - `behavior_deltas` (integer|null; minimum 1 when integer)
  - `qa` (object, required; `additionalProperties: false`)
    - `new_test_files` (integer|null; minimum 0 when integer)
    - `new_test_cases` (integer|null; minimum 0 when integer)
  - `docs` (object, required; `additionalProperties: false`)
    - `new_docs_files` (integer|null; minimum 0 when integer)
  - `ops` (object, required; `additionalProperties: false`)
    - `new_smoke_steps` (integer|null; minimum 0 when integer)
    - `ci_changes` (integer|null; minimum 0 when integer)
  - `risk` (object, required; `additionalProperties: false`)
    - `cross_platform` (boolean; default `false` if omitted by producer tooling)
    - `security_sensitive` (boolean; default `false` if omitted by producer tooling)
    - `concurrency_or_ordering` (boolean; default `false` if omitted by producer tooling)
    - `migration_or_backfill` (boolean; default `false` if omitted by producer tooling)
    - `unknowns_high` (integer|null; minimum 0 when integer)
  - `notes` (string; can be empty)

##### Missing-input trigger naming (v1, normative)

When a numeric value is missing or `null`, the tool MUST:

- append `<json_path>` (e.g., `touch.crates_touched`) to the `missing_inputs` array, and
- append `missing_inputs:<json_path>` (e.g., `missing_inputs:touch.crates_touched`) to the `triggers` array (CONTRACT-3).
- **Acceptance criteria**:
  - Schema property names/types match D6.
  - Null allowances match D6.
  - At least one embedded `examples` entry matches the canonical D3 sample.
- **Test notes**:
  - Validate against the D3 sample and at least one “fully specified” example.
- **Risk/rollback notes**:
  - If schema-backed validation needs gating, consumers MUST gate it behind an explicit strict-mode opt-in and MUST still enforce marker+JSON parsing and section type checks.

Checklist:
- Implement:
  - Draft schema structure (root + nested objects).
  - Add per-field descriptions (brief, not rubric-level prose).
  - Add `examples` covering fully-specified + discovery-time nulls.
- Test:
  - Validate the D3 sample block against the schema.
  - Validate a “fully specified” sample.
  - Validate an invalid-type sample fails.
- Validate:
  - Cross-check every field against D6 (no missing/extra canonical keys).
- Cleanup:
  - Ensure file path matches the approved location exactly.

#### S1.T2 — Define a conformance checklist for schema/rubric drift

- **Outcome**: A short, explicit checklist (in the rubric or a “Conformance” section) that makes drift detectable during future edits.
- **Inputs/outputs**:
  - Input: D6 canonical field list + D3 example block
  - Output: A “Conformance” subsection in `WORK_LIFT_RUBRIC.md` (authored in S2) that enumerates:
    - canonical fields,
    - where `null` is allowed,
    - marker conventions.
- **Implementation notes**:
  - Keep the checklist concise and mechanical (suitable for manual review).
- **Acceptance criteria**:
  - Checklist exists and is copy/paste-verifiable during PR review.
- **Test notes**:
  - N/A (documentation-only), but checklist MUST be verified against the schema once.
- **Risk/rollback notes**:
  - None.

Checklist:
- Implement:
  - Write the checklist structure to be filled in S2.
- Test:
  - Verify checklist matches schema after S2.T2.
- Validate:
  - Confirm checklist doesn’t include scoring weights (owned by CONTRACT-2).
- Cleanup:
  - None.
