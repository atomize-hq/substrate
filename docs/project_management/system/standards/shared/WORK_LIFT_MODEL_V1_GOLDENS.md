# Work Lift Model v1 — Golden Cases + Conformance

This document provides a small set of manually verifiable **golden cases** for `CONTRACT-2:work_lift_model_v1`, plus a mechanical conformance checklist to prevent drift.

---

## References (canonical)

- Lift Vector schema (inputs): `docs/project_management/system/schemas/work_lift_vector.schema.json` (`CONTRACT-1:work_lift_vector_block_v1`)
- Lift model config (constants): `docs/project_management/system/schemas/work_lift_model.v1.json` (`CONTRACT-2:work_lift_model_v1`)
- Canonical semantics: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D7/D8/D9)

---

## Golden cases (v1)

### GOLDEN-1 — ADR candidate (blowup + multipliers + missing-input confidence)

Context:
- `context`: `adr_candidate`
- Rationale: exercises base weights, ≥1 risk multiplier, ≥1 split trigger, and missing-input confidence degradation.

Input vector (must validate against `work_lift_vector.schema.json`):

```json
{
  "model_version": 1,
  "touch": {
    "create_files": 2,
    "edit_files": 3,
    "delete_files": 1,
    "deprecate_files": 0,
    "crates_touched": null,
    "boundary_crossings": 1
  },
  "contract": {
    "cli_flags": 1,
    "config_keys": 1,
    "exit_codes": 0,
    "file_formats": 1,
    "behavior_deltas": 2
  },
  "qa": { "new_test_files": 1, "new_test_cases": 4 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 1, "ci_changes": 1 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": true,
    "migration_or_backfill": true,
    "unknowns_high": 2
  },
  "notes": "Golden-1: blowup + multipliers + missing inputs."
}
```

Expected outputs (pins D7–D9 behavior via `work_lift_model.v1.json` constants):
- `model_version`: `1`
- `lift_score`: `78`
- `estimated_slices`: `7`
- `confidence`: `low`
- `missing_inputs`: `["touch.crates_touched"]`
- `triggers` (order-insensitive set; compare as a set in downstream tests):
  - `missing_inputs:touch.crates_touched`
  - `split_required:behavior_deltas>1`
  - `likely_split:lift_score>24`
  - `split_required:estimated_slices>3`

Notes:
- This golden intentionally does **not** depend on prefix-entry derived inputs (SEAM-4) to avoid coupling churn.

---

## Conformance checklist (v1 immutability + traceability + selection)

1) v1 immutability
- Do not change scoring semantics in `work_lift_model.v1.json`; publish `work_lift_model.v2.json` instead.

2) Traceability
- Every constant in `work_lift_model.v1.json` maps to `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` D7–D9 (or is explicitly documented as a default in `work_lift_model.v1.json.sources/summary`).

3) Field-name alignment
- `weights.touch/*`, `weights.qa/*`, `weights.docs/*`, `weights.ops/*` keys exactly match `work_lift_vector.schema.json` keys.
- `weights.contract` uses `behavior_deltas_blowup` (and excludes `behavior_deltas` weight) per the D7 blowup rule.
- `risk_multipliers` keys exactly match the `work_lift_vector.schema.json` boolean risk keys.

4) Selection semantics forbid “latest”
- Selection is explicit and pinned (as encoded in `work_lift_model.v1.json.selection`); no directory scanning for “latest”.

5) Goldens remain valid
- Golden input vectors validate against `work_lift_vector.schema.json`.
- Golden expected outputs are exact (no “approx”).

---

## How to validate goldens

Tool-agnostic:
- Use any JSON Schema validator to validate each golden input vector against `docs/project_management/system/schemas/work_lift_vector.schema.json`.

Python (jsonschema):

```python
import json
from pathlib import Path

from jsonschema import Draft202012Validator

SCHEMA_PATH = Path("docs/project_management/system/schemas/work_lift_vector.schema.json")
GOLDENS_PATH = Path("docs/project_management/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md")

schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
text = GOLDENS_PATH.read_text(encoding="utf-8")

start = text.index("```json") + len("```json")
end = text.index("```", start)
vector = json.loads(text[start:end].strip())

Draft202012Validator(schema).validate(vector)
print("OK: GOLDEN-1 vector validates")
```

