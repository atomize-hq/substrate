# world-disabled-diagnostics — planning quality gate (draft)

## Status
- Recommendation: FLAG FOR HUMAN REVIEW
- Reason: This pack is still in `draft/` and requires a full planning quality gate review before execution triads begin.

## Mechanical checks (must be green)

```bash
FEATURE_DIR="docs/project_management/packs/draft/world-disabled-diagnostics"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null

python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "$FEATURE_DIR"
```

## Review checklist (minimum)
- Slice specs are coherent and testable (`slices/WDD0`..`slices/WDD2`).
- Contract surfaces are deterministic (`contract.md`, `decision_register.md`, `world-disabled-diagnostics-json-schema-spec.md`).
- Cross-platform validation artifacts are runnable and aligned (`manual_testing_playbook.md`, `smoke/*`).
- `tasks.json` triad wiring matches the checkpoint plan and is automation-ready.
