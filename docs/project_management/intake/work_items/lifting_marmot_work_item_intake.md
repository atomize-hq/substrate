---
codename: lifting_marmot
created: "2026-02-26T00:37:09Z"
status: ready_for_lockdown
depends_on: []
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `lifting_marmot`
- Created: 2026-02-26T00:37:09Z
- Status: ready_for_lockdown

## 2. Optional workstream link

- Workstream: (none yet)

## 3. Title (imperative)

Bring all ADR intake forms up to the current intake template (add Work Lift summary + computed outputs).

## 4. Why not ADR

- This is documentation/system hygiene work with no A/B architecture or contract decision and no product/runtime behavior delta.

## 5. Task definition (bounded)

- For every file under `docs/project_management/intake/adrs/`:
  - Add section 14 “Lift Summary” per `docs/project_management/system/prompts/discovery/brainstorm_to_adr.md`.
  - Add a valid `PM_LIFT_VECTOR` block (v1) per `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`.
  - Run `make pm-lift-intake FILE=<adr_intake>` and paste the computed outputs into the intake under Lift Summary.
  - Minimal formatting fixes to align section numbering with the current ADR intake standard (no content/meaning rewrites).

## 6. Done means (<= 8 outcomes)

- All intake files in `docs/project_management/intake/adrs/` contain exactly one valid `PM_LIFT_VECTOR` block.
- Each ADR intake includes a Lift Summary section with the latest `pm-lift-intake` computed outputs embedded.
- `make pm-lift-intake FILE=docs/project_management/intake/adrs/<file>` succeeds for every ADR intake file.

## 7. Likely touch paths

- `docs/project_management/intake/adrs/*.md`
- `docs/project_management/intake/work_items/lifting_marmot_work_item_intake.md`

## 8. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 9. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": 1,
    "edit_files": 9,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 0
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": null
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": "Estimate: one new WI intake file + edits to nine existing ADR intake files."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 23
Estimated slices: 2
Confidence: low
Triggers:
- missing_inputs:contract.behavior_deltas
Missing inputs:
- contract.behavior_deltas
```

## 10. Open questions

- None.
