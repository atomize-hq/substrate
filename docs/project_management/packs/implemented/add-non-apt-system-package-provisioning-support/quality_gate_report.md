# add-non-apt-system-package-provisioning-support — planning quality gate (draft)

## Status
- Recommendation: FLAG FOR HUMAN REVIEW
- Reason: The planning pack is now automation-ready, but it remains under `draft/` and still requires a formal planning quality-gate review before execution triads begin.

## Mechanical checks

```bash
FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null

python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "$FEATURE_DIR"
python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "$FEATURE_DIR"
```

## Review checklist
- `tasks.json` matches the accepted slice order and uses schema-v4 checkpoint-boundary wiring for `NASP2` and `NASP4`.
- Every task referenced by `tasks.json` has a kickoff prompt, and every kickoff prompt includes the exact sentinel line `Do not edit planning docs inside the worktree.`
- `plan.md`, `pre-planning/ci_checkpoint_plan.md`, and `tasks.json` agree on:
  - orchestration branch
  - slice order
  - checkpoint boundaries
  - Linux, macOS, and Windows validation scope
- `session_log.md` records the planning wiring pass and validator outcomes.

## Current evidence
- `validate_tasks_json.py` passed on 2026-03-08.
- `validate_slice_specs.py` passed on 2026-03-08.
- `validate_ci_checkpoint_plan.py` passed on 2026-03-08.
- `planning-micro-lint` passed on the owned paths on 2026-03-08.
