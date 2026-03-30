# dev-install-world-agent-staging — planning quality gate (draft)

## Status
- Recommendation: FLAG FOR HUMAN REVIEW
- Reason: This pack is in `draft/` and should be reviewed for execution readiness before running the first execution triads.

## Mechanical checks (must be green)

```bash
FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null

python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "$FEATURE_DIR"
```

## Review checklist (minimum)
- Contract is deterministic and matches the current code seams (`contract.md`, `decision_register.md`).
- Slice specs are singular-delta and testable (`slices/DIWAS0`, `slices/DIWAS1`).
- Validation artifacts are runnable and aligned (`manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `platform-parity-spec.md`).
- `tasks.json` wiring matches the checkpoint plan and is automation-ready.

