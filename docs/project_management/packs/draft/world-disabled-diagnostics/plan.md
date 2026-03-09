# world-disabled-diagnostics — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- Orchestration branch: `feat/world-disabled-diagnostics`
- Canonical pre-planning inputs:
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`

## Goal
- Make `substrate shim doctor` and `substrate health` world-disabled aware:
  - disabled/skipped is explicit and non-error when `world.enabled=false`,
  - enabled-but-broken remains fail-visible when `world.enabled=true`,
  - JSON output is additive-only and machine-detectable (status enums + omission rules),
  - behavior is cross-platform (Linux/macOS/Windows) with bounded CI checkpoints.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Each task (code/test/integ) must be scoped to fit within 40% of a typical 272k token context window (≤ 108,800 tokens).

## Triads (slice order)
- `WDD0`: shared effective-config resolution seam for diagnostics (`effective_world_enabled`).
- `WDD1`: `substrate shim doctor` disabled/skipped statuses (text + JSON) + no-probes boundary.
- `WDD2`: `substrate health` disabled/skipped summary + docs alignment; checkpoint boundary (schema v4 platform-fix model).

## CI checkpoints (cross-platform)
- Checkpoint plan: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`
- Boundary slice (schema v4): `WDD2` (see `tasks.json` `meta.checkpoint_boundaries`)
- Ops task: `CP1-ci-checkpoint` (dispatch compile parity + feature smoke from orchestration checkout)

## Validation artifacts (required by this pack)
- Manual playbook: `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
- Smoke scripts:
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`

## Planning validation (mechanical)
- `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/world-disabled-diagnostics"`
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-disabled-diagnostics"`
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-disabled-diagnostics"`
