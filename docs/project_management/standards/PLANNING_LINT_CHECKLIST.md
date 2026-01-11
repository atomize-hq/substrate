# Planning Lint Checklist (Mechanical Quality Gate)

This checklist is **mechanical** and **non-negotiable**. The quality gate reviewer must run it and record results in:
- `docs/project_management/next/<feature>/quality_gate_report.md` (use `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`)

If any check fails, the Planning Pack is not execution-ready.

## Define scope

Set a feature directory variable:
- `export FEATURE_DIR="docs/project_management/next/world-sync"`

## Run the mechanical lint runner (required)

On Linux/macOS:
```bash
make planning-lint FEATURE_DIR="$FEATURE_DIR"
```

On Windows:
```powershell
make planning-lint-ps FEATURE_DIR=$env:FEATURE_DIR
```

This runner checks (at minimum):
- Required Planning Pack artifacts exist (`plan.md`, `tasks.json`, `session_log.md`, `kickoff_prompts/`, and `smoke/*` when applicable)
- Hard-ban scan (no `TBD/TODO/WIP/TBA`, no “open question”, no “etc.”/“and so on”)
- Ambiguity scan (no `should|could|might|maybe` in behavior/contracts)
- `tasks.json` invariants (`make planning-validate`)
  - If `tasks.json` opts into schema v2 cross-platform parity (`meta.schema_version >= 2` and `meta.ci_parity_platforms_required` or legacy `meta.platforms_required`), it must include the required `X-integ-core`, `X-integ-<platform>`, and `X-integ` tasks per slice (where `<platform>` ranges over CI parity platforms, plus optional `wsl` when `wsl_task_mode="separate"`).
  - If WSL coverage is required, use `meta.wsl_required: true` + `meta.wsl_task_mode: "bundled"|"separate"` (do not include `"wsl"` in `meta.behavior_platforms_required` or `meta.ci_parity_platforms_required`).
  - If `FEATURE_DIR/smoke/` exists, smoke scripts are required only for `meta.behavior_platforms_required` (or inferred legacy behavior platforms); `manual_testing_playbook.md` must reference each required smoke script.
  - If `meta.execution_gates: true`, it must include the execution preflight task/report and per-slice closeout report linkage.
- ADR Executive Summary drift checks for any ADRs found/referenced (`make adr-check`)
- Kickoff prompt sentinel coverage (must contain `Do not edit planning docs inside the worktree.`)
- Manual playbook must reference smoke scripts (when both exist)
- `sequencing.json` includes this feature directory
- `sequencing.json` completed sprint pointers resolve (supports archived Planning Packs under `docs/project_management/_archived/`)

## Debugging a failure (optional)

If the runner fails, use these to isolate the cause:

- Hard bans: `rg -n --hidden --glob '!**/.git/**' '\\b(TBD|TODO|WIP|TBA)\\b|open question|\\betc\\.|and so on' "$FEATURE_DIR"`
- Ambiguity words: `rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' '\\b(should|could|might|maybe)\\b' "$FEATURE_DIR"`
- `tasks.json` invariants: `make planning-validate FEATURE_DIR="$FEATURE_DIR"`
- ADR exec summary drift: `make adr-check ADR=docs/project_management/next/ADR-000X-foo.md`
