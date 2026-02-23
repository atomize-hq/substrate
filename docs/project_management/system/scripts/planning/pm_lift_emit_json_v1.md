# CONTRACT-3: `pm_lift --emit-json` output contract (v1)

This document defines the stable machine contract for:

- `docs/project_management/system/scripts/planning/pm_lift.py ... --emit-json`

This is **CONTRACT-3:pm_lift_emit_json_v1**.

Evolution rule (v1):
- **Additive keys only**. Never rename or remove existing keys in v1 output.

---

## Stdout / stderr / exit codes (normative)

When `--emit-json` is provided, `pm_lift.py` MUST:

- Write **JSON only** to stdout on success.
- Write nothing to stdout on any non-zero exit.
- Write all warnings/errors to stderr.
- Exit codes:
  - `0` on success
  - `2` on usage errors (argparse)
  - `1` on runtime/validation errors (missing markers, invalid JSON, invalid config, etc.)

---

## Required keys + types (normative)

On success, stdout MUST be a single JSON object with these required keys:

```json
{
  "model_version": "integer (>=1)",
  "lift_score": "integer (>=0)",
  "estimated_slices": "integer (>=1)",
  "confidence": "\"high\" | \"low\"",
  "triggers": "array<string> (sorted asc, unique)",
  "missing_inputs": "array<string> (sorted asc, unique)",
  "vector": "object (the input Lift Vector; see CONTRACT-1)",
  "derived": "object (debug/audit; additive keys allowed)"
}
```

Notes:
- `triggers` and `missing_inputs` are deterministic for the same input/config.
- Additional top-level keys are allowed in future versions only if they are additive and do not change existing meaning.

---

## Stable semantics (v1, normative)

### Confidence

`confidence` MUST be:

- `low` iff:
  - `missing_inputs` is non-empty, OR
  - the derived-pack input indicates prefix entries (`dir_prefixes` non-empty from CONTRACT-4).
- `high` otherwise.

### Missing inputs

- `missing_inputs` entries are Lift Vector JSON paths, e.g. `touch.crates_touched`.
- For every `missing_inputs` entry `X`, `triggers` MUST contain `missing_inputs:X`.

### Triggers

`triggers` contains only machine tokens (no prose). V1 tokens are:

- `split_required:behavior_deltas>1`
- `likely_split:crates_touched>2`
- `likely_split:touch_files_sum>12`
- `likely_split:contract_surface_sum>4`
- `likely_split:lift_score>24`
- `split_required:estimated_slices>3`
- `touch_set_contains_prefix_entries`
- plus any number of `missing_inputs:<json_path>` entries

### Derived (debug/audit)

`derived` MUST include:

- `base_points` (number)
- `risk_multiplier` (number)

For `from-impact-map` mode only, `derived` MUST include:

- `impact_map_touch_counts` (object)
- `touch_effective_for_scoring` (object)

Additional `derived` keys are allowed and MUST be treated as non-breaking additive evolution.

`derived` MUST NOT include unstable, host-specific paths.

---

## Example output (valid v1)

Example output from `pm_lift.py from-intake --emit-json` (illustrative; values depend on the input):

```json
{
  "confidence": "low",
  "derived": {
    "base_points": 0.0,
    "risk_multiplier": 1.0
  },
  "estimated_slices": 1,
  "lift_score": 0,
  "missing_inputs": [
    "touch.crates_touched"
  ],
  "model_version": 1,
  "triggers": [
    "missing_inputs:touch.crates_touched"
  ],
  "vector": {
    "contract": {
      "behavior_deltas": 1,
      "cli_flags": 0,
      "config_keys": 0,
      "exit_codes": 0,
      "file_formats": 0
    },
    "docs": {
      "new_docs_files": 0
    },
    "model_version": 1,
    "notes": "",
    "ops": {
      "ci_changes": 0,
      "new_smoke_steps": 0
    },
    "qa": {
      "new_test_cases": 0,
      "new_test_files": 0
    },
    "risk": {
      "concurrency_or_ordering": false,
      "cross_platform": false,
      "migration_or_backfill": false,
      "security_sensitive": false,
      "unknowns_high": 0
    },
    "touch": {
      "boundary_crossings": 0,
      "crates_touched": null,
      "create_files": 0,
      "delete_files": 0,
      "deprecate_files": 0,
      "edit_files": 0
    }
  }
}
```

