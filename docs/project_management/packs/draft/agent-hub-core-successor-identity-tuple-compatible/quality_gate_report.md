RECOMMENDATION: REVISE

# agent-hub-core-successor-identity-tuple-compatible — planning quality gate

## Status
- Recommendation: `REVISE`
- Reason: the owned task graph and kickoff surfaces validate cleanly, but the required checkpoint-plan validator is blocked by `pre-planning/ci_checkpoint_plan.md`, which still uses the draft machine-readable header and is outside this PWS allowlist.

## Mechanical checks
- `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"` → `0` → `PASS`
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"` → `0` → `PASS`
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"` → `1` → `FAIL`
  - blocker: `pre-planning/ci_checkpoint_plan.md` is missing the required header `## Machine-readable plan (linted)`
- `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" OWNED_PATHS="plan.md tasks.json session_log.md quality_gate_report.md kickoff_prompts slices/AHCSITC0/kickoff_prompts slices/AHCSITC1/kickoff_prompts slices/AHCSITC2/kickoff_prompts slices/AHCSITC3/kickoff_prompts"` → `0` → `PASS`

## Review checklist
- `plan.md` locks the accepted slice order and the `CP1` / `CP2` checkpoint rule.
- `tasks.json` stays schema-v4, automation-enabled, cross-platform, and checkpoint-bound.
- `AHCSITC2` and `AHCSITC3` use the full checkpoint-boundary model with `*-integ-core`, `*-integ-<platform>`, and final `*-integ`.
- Slice-task `ac_ids` match `AHCSITC0` through `AHCSITC3` exactly on `*-code`, `*-test`, and final `*-integ`.
- Every populated `kickoff_prompt` path points at a real file under the allowlisted directories.
- Every kickoff prompt includes the exact sentinel line `Do not edit planning docs inside the worktree.`

## Blocking gap
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/ci_checkpoint_plan.md`
  - required change: replace `## Machine-readable plan (draft; not yet mechanically validated)` with `## Machine-readable plan (linted)`
  - status: allowlist request emitted under `logs/pws/AHCSITC-PWS-tasks_checkpoints/`
