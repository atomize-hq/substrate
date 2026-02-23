### S3 — Goldens + negative cases + conformance guardrails

- **User/system value**: Prevent silent drift in scoring semantics and JSON output contract by pinning goldens and negative fixtures in fast tests.
- **Scope (in/out)**:
  - In:
    - Add golden fixtures for both intake vectors and derived-pack inputs.
    - Add negative fixtures for validation and error handling.
    - Add conformance guards for:
      - CONTRACT-3 JSON key set + type-shape,
      - determinism (ordering, stable triggers strings),
      - config/schema presence behaviors (advisory-first guidance).
  - Out:
    - End-to-end workflow integration (owned by SEAM-5).
    - Revising schema/config artifacts (owned by SEAM-1/SEAM-2).
- **Acceptance criteria**:
  - A single command (python test invocation for planning scripts) runs:
    - golden cases (assert expected `lift_score`, `estimated_slices`, `confidence`, selected triggers),
    - negative cases (assert non-zero exit + actionable stderr).
  - Tests tolerate additive JSON keys:
    - require known-required CONTRACT-3 keys,
    - allow extra keys in `derived` and future additions.
- **Dependencies**:
  - Consumes:
    - `CONTRACT-1` schema file (SEAM-1) for schema-valid fixtures and schema violation fixtures.
    - `CONTRACT-2` config file (SEAM-2) for stable goldens across scoring changes.
- **Verification**:
  - Run the new tests locally.
  - Spot-check example outputs in the goldens for readability and debuggability.
- **Rollout/safety**:
  - Additive-only guardrails: tests should fail on breaking changes, not on benign additive enhancements.

#### S3.T1 — Add golden fixtures + expected outputs (intake vectors)

- **Outcome**: A small set of schema-valid Lift Vectors with pinned expected outputs to detect scoring drift.
- **Inputs/outputs**:
  - Inputs:
    - CONTRACT-1 schema (SEAM-1)
    - CONTRACT-2 config (SEAM-2)
  - Outputs:
    - One or more fixture markdown files containing Lift Vector blocks.
    - Expected JSON outputs (stored as separate `.json` goldens or as inline expectations in tests).
- **Implementation notes**:
  - Cover at least:
    - a “small/low-lift” case (high confidence, few triggers),
    - a “missing inputs” case (`null` values) that forces confidence downgrade + missing_inputs triggers,
    - a “split likely/required” case that exercises threshold-based triggers.
  - Keep fixtures human-readable and small.
- **Acceptance criteria**:
  - Goldens pass deterministically and fail on scoring or trigger-string drift.
- **Test notes**:
  - Run `pm_lift.py from-intake --emit-json` on each fixture and compare selected fields.
- **Risk/rollback notes**:
  - If scoring must change, update CONTRACT-2 version (v2) instead of mutating v1; then add new goldens.

Checklist:
- Implement:
  - Create fixtures and expected outputs.
  - Add tests that compare expected key fields.
- Validate:
  - Confirm fixtures validate against CONTRACT-1 schema.
- Cleanup:
  - Keep expected outputs tolerant of additive `derived` keys.

#### S3.T2 — Add derived-pack goldens (impact map emit JSON) without relying on full packs

- **Outcome**: Stable tests for `from-impact-map` mode that validate prefix-entry confidence/triggers and derived audit fields.
- **Inputs/outputs**:
  - Inputs:
    - CONTRACT-4 shape (SEAM-4)
    - CONTRACT-2 config (SEAM-2)
  - Outputs:
    - A stubbed validator JSON output fixture (or a test harness that injects it) and expected `pm_lift` results.
- **Implementation notes**:
  - Avoid coupling to filesystem state:
    - prefer stubbing `_run_validate_impact_map_emit_json` or injecting fixture output in tests.
  - Cover both:
    - no prefixes (confidence unaffected),
    - prefixes present (confidence downgrade + trigger).
- **Acceptance criteria**:
  - Prefix-entry behavior is deterministic and pinned.
- **Test notes**:
  - Assert both `confidence` and the presence of a stable trigger string.
- **Risk/rollback notes**:
  - If prefix policy changes, that should be a model config version bump (v2) and new goldens.

Checklist:
- Implement:
  - Add fixtures for derived-pack mode.
  - Add tests that run or simulate the `from-impact-map` path.
- Validate:
  - Confirm stdout JSON-only and stable keys.
- Cleanup:
  - Keep stubs minimal and localized to tests.

#### S3.T3 — Add negative cases (validation + config/schema error messaging)

- **Outcome**: Guardrails for common failure modes with actionable diagnostics and stable exit behavior.
- **Inputs/outputs**:
  - Inputs:
    - Invalid fixture markdowns (missing markers, invalid JSON, wrong types, schema violations)
    - Invalid/missing config or schema (simulated via temporary paths or monkeypatch)
  - Outputs:
    - Tests that assert:
      - non-zero exit code,
      - stderr contains actionable guidance (file path + what to fix),
      - stdout is not partial JSON when `--emit-json` is used.
- **Implementation notes**:
  - Keep assertions resilient:
    - match key substrings rather than full error text, but require “actionable” elements (path, field name).
- **Acceptance criteria**:
  - Tests fail if error paths become silent, ambiguous, or emit partial JSON to stdout.
- **Test notes**:
  - Run tests in a clean environment and confirm behavior is consistent.
- **Risk/rollback notes**:
  - If strictness increases, introduce an explicit strict mode toggle; do not surprise legacy users.

Checklist:
- Implement:
  - Add invalid fixtures.
  - Add tests for each failure class.
- Validate:
  - Confirm exit codes and stderr are stable/actionable.
- Cleanup:
  - Keep fixtures small and names descriptive.

