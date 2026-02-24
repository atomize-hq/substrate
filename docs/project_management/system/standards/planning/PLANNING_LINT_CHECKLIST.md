# Planning Lint Checklist (Mechanical Quality Gate)

This checklist is **mechanical** and **non-negotiable**. The quality gate reviewer must run it and record results in:
- `docs/project_management/packs/active/<feature>/quality_gate_report.md` (use `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`)

If any check fails, the Planning Pack is not execution-ready.

## Define scope

Set a feature directory variable:
- `export FEATURE_DIR="docs/project_management/packs/active/world-sync"`

## Run the mechanical lint runner (required)

On Linux/macOS:
```bash
make planning-lint FEATURE_DIR="$FEATURE_DIR"
```

Optional (recommended for strict packs): include Work Lift advisory output in the lint run (still non-blocking by default):
```bash
PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"
```

On Windows:
```powershell
make planning-lint-ps FEATURE_DIR=$env:FEATURE_DIR
```

Optional (recommended for strict packs): include Work Lift advisory output in the lint run (still non-blocking by default):
```powershell
$env:PM_LIFT_ADVISORY="1"; make planning-lint-ps FEATURE_DIR=$env:FEATURE_DIR
```

This runner checks (at minimum):
- Required Planning Pack artifacts exist (`plan.md`, `spec_manifest.md`, `impact_map.md`, `tasks.json`, `session_log.md`, `kickoff_prompts/`, and `smoke/*` when applicable)
  - For cross-platform automation packs (schema v3+ + meta.automation.enabled=true): `ci_checkpoint_plan.md` must exist and pass mechanical validation.
- `spec_manifest.md` exists when an ADR is present or referenced
- All backticked required-doc paths listed in `spec_manifest.md` exist on disk
- `impact_map.md` exists as part of the required-doc list (replaces legacy `integration_map.md`)
- Hard-ban scan (no `TBD/TODO/WIP/TBA`, no “open question”, no “etc.”/“and so on”)
- Ambiguity scan (no `should|could|might|maybe` in behavior/contracts)
- `tasks.json` invariants (`make planning-validate`)
  - If `meta.cross_platform=true` (schema v2+):
    - Schema v2/v3: it must include `X-integ-core`, `X-integ-<platform>`, and `X-integ` tasks per slice (where `<platform>` ranges over CI parity platforms, plus optional `wsl` when `wsl_task_mode="separate"`).
    - Schema v4+: it must include `meta.checkpoint_boundaries`, and only checkpoint-boundary slices may define `*-integ-core` / `*-integ-<platform>` tasks (normal slices use only `X-integ` as the per-slice merge task).
  - If WSL coverage is required, use `meta.wsl_required: true` + `meta.wsl_task_mode: "bundled"|"separate"` (do not include `"wsl"` in `meta.behavior_platforms_required` or `meta.ci_parity_platforms_required`).
  - If `FEATURE_DIR/smoke/` exists, smoke scripts are required only for `meta.behavior_platforms_required` (or inferred legacy behavior platforms); `manual_testing_playbook.md` must reference each required smoke script.
  - If `meta.execution_gates: true`, it must include the execution preflight task/report and per-slice closeout report linkage.
- ADR Executive Summary drift checks for any ADRs found/referenced (`make adr-check`)
- Kickoff prompt sentinel coverage (must contain `Do not edit planning docs inside the worktree.`)
- Manual playbook must reference smoke scripts (when both exist)
- `sequencing.json` includes this feature directory
- `sequencing.json` completed sprint pointers resolve (supports archived Planning Packs under `docs/project_management/_archived/`)

When `PM_LIFT_ADVISORY=1` is set, lint also prints a Work Lift advisory report (see: `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`).

## Debugging a failure (optional)

If the runner fails, use these to isolate the cause:

- Hard bans: `rg -n --hidden --glob '!**/.git/**' '\\b(TBD|TODO|WIP|TBA)\\b|open question|\\betc\\.|and so on' "$FEATURE_DIR"`
- Ambiguity words: `rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' '\\b(should|could|might|maybe)\\b' "$FEATURE_DIR"`
- `tasks.json` invariants: `make planning-validate FEATURE_DIR="$FEATURE_DIR"`
- ADR exec summary drift: `make adr-check ADR=docs/project_management/adrs/queued/ADR-000X-foo.md` (or any supported ADR path)
