# world-disabled-reason-attribution — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- Orchestration branch: `feat/world-disabled-reason-attribution`
- Canonical pre-planning inputs:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/workstream_triage.md`

## Goal
- replay output and replay telemetry attribute effective world disablement with the same winning-layer semantics as ADR-0037
- replay-local opt-out tokens remain unchanged
- redaction stays strict and cross-platform

## Guardrails
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Each task must fit within 40 percent of a typical 272k token context window.

## Triads (slice order)
- `WDRA0`: shared replay disable-attribution classifier seam.
- `WDRA1`: replay stderr copy and `replay_strategy` telemetry wiring.
- `WDRA2`: regression coverage, docs alignment, and cross-platform validation seam.

## CI checkpoints
- Checkpoint plan: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`
- Boundary slice: `WDRA2`
- Ops task: `CP1-ci-checkpoint`

## Validation artifacts
- Manual playbook: `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`
- Smoke scripts:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`

## Planning validation
- `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/world-disabled-reason-attribution"`
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-disabled-reason-attribution"`
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-disabled-reason-attribution"`
- `make planning-lint FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution"`
