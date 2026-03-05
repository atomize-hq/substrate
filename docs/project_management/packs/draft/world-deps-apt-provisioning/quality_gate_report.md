# world-deps-apt-provisioning — planning quality gate (draft)

## Status
- Recommendation: FLAG FOR HUMAN REVIEW
- Reason: This pack is in `draft/` and requires a full planning quality gate review before execution triads begin.
- Blocker: `validate_slice_specs.py` fails because `slices/WDAP1/WDAP1-spec.md` has 10 top-level AC bullets; v2 requires 1..8.

## Mechanical checks (must be green)

```bash
FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null

python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "$FEATURE_DIR"
```

## Review checklist (minimum)
- Slice specs are coherent and testable (`slices/WDAP0`, `slices/WDAP1`).
- Contract surfaces are deterministic (`contract.md`, `decision_register.md`).
- Cross-platform validation artifacts are runnable and aligned (`manual_testing_playbook.md`, `smoke/*`).
- `tasks.json` triad wiring matches the checkpoint plan and is automation-ready.
