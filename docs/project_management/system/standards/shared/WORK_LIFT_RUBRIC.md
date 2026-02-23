# Work Lift Rubric — Lift Vector v1

This standard defines how to write a valid **Lift Vector v1** block in intake/ADR/planning markdown.

It is the human companion to the machine schema at `docs/project_management/system/schemas/work_lift_vector.schema.json` and exists to prevent “invented precision” while keeping lift inputs deterministic to parse.

---

## References (canonical)

- Decision log (canonical field list + example): `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D3/D6)
- JSON Schema (machine validation): `docs/project_management/system/schemas/work_lift_vector.schema.json` (`CONTRACT-1:work_lift_vector_block_v1`)
- Scoring/model config (reference only; do not embed weights here): `docs/project_management/system/schemas/work_lift_model.v1.json` (`CONTRACT-2:work_lift_model_v1`, published by SEAM-2)

---

## Non-negotiable invariants (v1)

- A Lift Vector block MUST be wrapped by these exact markers:
  - `<!-- PM_LIFT_VECTOR:BEGIN -->`
  - `<!-- PM_LIFT_VECTOR:END -->`
- Inside the markers there MUST be exactly one fenced JSON block:
  - fence type MUST be `json` (i.e., starts with ```json)
  - content MUST be valid JSON (no comments)
- The JSON object MUST validate against `docs/project_management/system/schemas/work_lift_vector.schema.json`.
- Unknown keys are forbidden (schema uses `additionalProperties: false`). Adding a new field requires publishing a schema update.
- Numeric fields MAY be omitted or set to `null` to mean “unknown at discovery time”. Authors MUST NOT invent precision.
- Tooling semantics (see `CONTRACT-3:pm_lift_emit_json_v1`): tools treat missing/`null` numeric values as `0` for scoring, degrade confidence, and emit `missing_inputs:<json_path>` triggers + `missing_inputs` entries.

---

## How to add a Lift Vector block (copy/paste)

This minimal skeleton is valid and is appropriate when many counts are unknown at discovery time:

````md
<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {},
  "contract": {},
  "qa": {},
  "docs": {},
  "ops": {},
  "risk": {},
  "notes": ""
}
```
<!-- PM_LIFT_VECTOR:END -->
````

Notes:
- Add known fields as you learn them; leaving numeric fields omitted or setting them to `null` is allowed.
- Use `0` only when you are confident the true value is zero.

---

## Null / unknown guidance (discovery-time)

Use these conventions consistently:

- Use `0` when you believe the count is deterministically **zero**.
- Use `null` or omit the field when the count is **unknown** at discovery time.
  - `null` and omission are equivalent for downstream tooling.
- `risk.*` booleans are booleans only (no `null`). If omitted, producer tooling treats them as `false`, but authors SHOULD set them explicitly to `true` when the risk clearly applies.

Directory/prefix policy (for planning-time derivations from `impact_map.md`, per D6):
- If a touch set entry is a directory/prefix token (implies multiple files but doesn’t specify count), count it as **1** toward the corresponding `touch.*_files` count; tooling may degrade confidence.

---

## Field-by-field rubric (v1)

All JSON paths below are relative to the Lift Vector root object.

### Root

- `model_version` (`integer`, optional; if present MUST be `1`)
  - Meaning: Lift Vector schema/model version.
  - Guidance: omit unless a tool/script writes it; if present, keep it as `1`.
- `notes` (`string`)
  - Meaning: freeform context; can be empty.

### `touch` (counts of file/module touch surface)

- `touch.create_files` (`integer|null`)
  - Meaning: count of created files (or touch-set “Create” entries).
  - Counting guidance: count each explicit path entry as 1; directory/prefix entries count as 1.
  - Unknown guidance: omit or `null` when unknown.
- `touch.edit_files` (`integer|null`)
  - Meaning: count of edited files (or “Edit” entries).
  - Counting guidance: same as `touch.create_files`.
  - Unknown guidance: omit or `null` when unknown.
- `touch.delete_files` (`integer|null`)
  - Meaning: count of deleted files (or “Delete” entries).
  - Counting guidance: same as `touch.create_files`.
  - Unknown guidance: omit or `null` when unknown.
- `touch.deprecate_files` (`integer|null`)
  - Meaning: count of deprecated files (or “Deprecate” entries).
  - Counting guidance: same as `touch.create_files`.
  - Unknown guidance: omit or `null` when unknown.
- `touch.crates_touched` (`integer|null`)
  - Meaning: rough count of crates/major modules touched.
  - Counting guidance: count distinct crates/modules expected to change materially.
  - Unknown guidance: omit or `null` when unclear.
- `touch.boundary_crossings` (`integer|null`)
  - Meaning: number of subsystems affected (requires a defined taxonomy).
  - Counting guidance: count distinct subsystem boundaries crossed; do not guess if taxonomy is undefined.
  - Unknown guidance: omit or `null` when the taxonomy/impact is unclear.

### `contract` (user-facing contract surface deltas)

- `contract.cli_flags` (`integer|null`)
  - Meaning: new/changed commands or flags.
  - Counting guidance: count distinct public flags/commands changed (not internal CLI plumbing).
  - Unknown guidance: omit or `null` when unknown.
- `contract.config_keys` (`integer|null`)
  - Meaning: new/changed config keys.
  - Counting guidance: count distinct keys or config surface changes.
  - Unknown guidance: omit or `null` when unknown.
- `contract.exit_codes` (`integer|null`)
  - Meaning: new/changed exit codes.
  - Counting guidance: count distinct exit code meanings added/changed.
  - Unknown guidance: omit or `null` when unknown.
- `contract.file_formats` (`integer|null`)
  - Meaning: new/changed on-disk schemas or file formats.
  - Counting guidance: count distinct serialized formats/schemas impacted (not in-memory structs).
  - Unknown guidance: omit or `null` when unknown.
- `contract.behavior_deltas` (`integer|null`, minimum 1 when present)
  - Meaning: intentionally counts meaningful behavior changes.
  - Counting guidance: typically `1` per ADR candidate; `>1` is an intentional “blow up” signal.
  - Unknown guidance: omit or `null` when unknown, but prefer setting `1` if you know there is at least one behavior delta.

### `qa` (test delta)

- `qa.new_test_files` (`integer|null`)
  - Meaning: number of new test files expected.
  - Counting guidance: count distinct test files added.
  - Unknown guidance: omit or `null` when unknown.
- `qa.new_test_cases` (`integer|null`)
  - Meaning: number of new test cases/assertions expected.
  - Counting guidance: count distinct cases/assertion groups; do not overfit granularity at discovery time.
  - Unknown guidance: omit or `null` when unknown.

### `docs` (documentation delta)

- `docs.new_docs_files` (`integer|null`)
  - Meaning: number of new documentation files expected.
  - Counting guidance: count distinct new docs files (not edits).
  - Unknown guidance: omit or `null` when unknown.

### `ops` (operational/CI delta)

- `ops.new_smoke_steps` (`integer|null`)
  - Meaning: number of new smoke test steps expected.
  - Counting guidance: count distinct steps that would appear in a smoke procedure/script.
  - Unknown guidance: omit or `null` when unknown.
- `ops.ci_changes` (`integer|null`)
  - Meaning: number of CI workflow changes expected.
  - Counting guidance: count distinct CI workflow/config updates.
  - Unknown guidance: omit or `null` when unknown.

### `risk` (risk booleans + unknowns)

- `risk.cross_platform` (`boolean`)
  - Meaning: work requires behavior/support across multiple platforms.
  - Guidance: set `true` when Linux/macOS/Windows parity matters.
- `risk.security_sensitive` (`boolean`)
  - Meaning: touches security-sensitive behavior, secrets, policy, or exploit surface.
  - Guidance: set `true` for anything that can leak or weaken protections.
- `risk.concurrency_or_ordering` (`boolean`)
  - Meaning: introduces or changes concurrency, ordering, or race-prone behavior.
  - Guidance: set `true` when correctness depends on timing/order.
- `risk.migration_or_backfill` (`boolean`)
  - Meaning: requires a migration or backfill.
  - Guidance: set `true` when existing state/data/config must be transformed.
- `risk.unknowns_high` (`integer|null`)
  - Meaning: count of blocking/high unknowns.
  - Counting guidance: count distinct unknowns that could change the plan materially.
  - Unknown guidance: omit or `null` when you can’t estimate even the count.

---

## Example (valid Lift Vector v1 block)

This example is copied from the decision log (D3) and MUST validate against the schema.

````md
<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 0,
    "edit_files": 0,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": null,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": ""
}
```
<!-- PM_LIFT_VECTOR:END -->
````

---

## Conformance checklist (schema ↔ rubric ↔ examples)

Mechanical checks (PR review friendly):

1) Marker lines match exactly: `PM_LIFT_VECTOR:BEGIN` / `PM_LIFT_VECTOR:END`.
2) Example JSON parses as JSON and validates against `docs/project_management/system/schemas/work_lift_vector.schema.json`.
3) Field inventory matches exactly: rubric fields == schema properties (no extras, no missing).
4) Null allowances match:
   - numeric fields are `integer|null` (or omitted),
   - booleans are booleans (non-nullable),
   - `notes` is a string.
5) Strictness acknowledged: unknown keys are invalid unless the schema is updated.

### Canonical field inventory (v1)

| json_path | type | null_allowed |
| --- | --- | --- |
| `model_version` | `integer (const 1, optional)` | no |
| `touch` | `object (required)` | no |
| `touch.create_files` | `integer|null` | yes |
| `touch.edit_files` | `integer|null` | yes |
| `touch.delete_files` | `integer|null` | yes |
| `touch.deprecate_files` | `integer|null` | yes |
| `touch.crates_touched` | `integer|null` | yes |
| `touch.boundary_crossings` | `integer|null` | yes |
| `contract` | `object (required)` | no |
| `contract.cli_flags` | `integer|null` | yes |
| `contract.config_keys` | `integer|null` | yes |
| `contract.exit_codes` | `integer|null` | yes |
| `contract.file_formats` | `integer|null` | yes |
| `contract.behavior_deltas` | `integer|null (minimum 1 when integer)` | yes |
| `qa` | `object (required)` | no |
| `qa.new_test_files` | `integer|null` | yes |
| `qa.new_test_cases` | `integer|null` | yes |
| `docs` | `object (required)` | no |
| `docs.new_docs_files` | `integer|null` | yes |
| `ops` | `object (required)` | no |
| `ops.new_smoke_steps` | `integer|null` | yes |
| `ops.ci_changes` | `integer|null` | yes |
| `risk` | `object (required)` | no |
| `risk.cross_platform` | `boolean` | no |
| `risk.security_sensitive` | `boolean` | no |
| `risk.concurrency_or_ordering` | `boolean` | no |
| `risk.migration_or_backfill` | `boolean` | no |
| `risk.unknowns_high` | `integer|null` | yes |
| `notes` | `string (required)` | no |

### How to validate the example

VS Code:
- Use `docs/project_management/system/schemas/work_lift_vector.schema.json` as the validation schema for the example JSON.

Python (jsonschema):

```python
import json
from pathlib import Path

from jsonschema import Draft202012Validator

SCHEMA_PATH = Path("docs/project_management/system/schemas/work_lift_vector.schema.json")
RUBRIC_PATH = Path("docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md")

BEGIN_LINE = "<!-- PM_LIFT_VECTOR:BEGIN -->"
END_LINE = "<!-- PM_LIFT_VECTOR:END -->"

schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
lines = RUBRIC_PATH.read_text(encoding="utf-8").splitlines()

marker_blocks = []
inside = False
current = []
for line in lines:
    if line.strip() == BEGIN_LINE:
        inside = True
        current = []
        continue
    if line.strip() == END_LINE and inside:
        marker_blocks.append(current)
        inside = False
        continue
    if inside:
        current.append(line)

if inside:
    raise SystemExit("Unclosed PM_LIFT_VECTOR block (missing END marker)")

if not marker_blocks:
    raise SystemExit("No PM_LIFT_VECTOR blocks found in WORK_LIFT_RUBRIC.md")

def _extract_json(block_lines: list[str]) -> dict:
    try:
        start = next(i for i, l in enumerate(block_lines) if l.strip() == "```json")
    except StopIteration:
        raise SystemExit("PM_LIFT_VECTOR block missing ```json fence")
    try:
        end = next(i for i in range(start + 1, len(block_lines)) if block_lines[i].strip() == "```")
    except StopIteration:
        raise SystemExit("PM_LIFT_VECTOR block missing closing ``` fence")
    return json.loads("\n".join(block_lines[start + 1 : end]).strip())

vectors = [_extract_json(b) for b in marker_blocks]
vector = next((v for v in vectors if v.get("model_version") == 1), vectors[0])
Draft202012Validator(schema).validate(vector)
print("OK: Lift Vector example validates")
```
