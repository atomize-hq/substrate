# dev-install-world-agent-staging — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
- Orchestration branch: `feat/dev-install-world-agent-staging`
- Canonical planning inputs:
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/workstream_triage.md`

## Goal
- Make the Linux “dev install with `--no-world`, enable later” workflow execution-ready with one shared path rule, one deterministic missing-artifact failure, and one checkpoint that validates the full feature.

## Guardrails
- Specs are the single source of truth.
- Planning-pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Accepted slice order remains `DIWAS0`, then `DIWAS1`.
- `DIWAS1` is the only checkpoint-boundary slice.
- The pack adds no new protocol, telemetry, policy, or config-format surface.
- Each task fits inside the triad context budget limit.

## Triads

### DIWAS0
- Objective:
  - add the standard version-dir missing-artifact preflight to `substrate world enable`
- Tasks:
  - `DIWAS0-code`
  - `DIWAS0-test`
  - `DIWAS0-integ`
- Validation evidence:
  - `cargo test -p shell --test world_enable -- --nocapture`
  - Cases 3 and 4 in `manual_testing_playbook.md`

### DIWAS1
- Objective:
  - stage `world-agent` during Linux `dev-install-substrate.sh --no-world` and validate the selected-profile refresh rule
- Tasks:
  - `DIWAS1-code`
  - `DIWAS1-test`
  - `DIWAS1-integ-core`
  - `CP1-ci-checkpoint`
  - `DIWAS1-integ-linux`
  - `DIWAS1-integ-macos`
  - `DIWAS1-integ-windows`
  - `DIWAS1-integ`
- Validation evidence:
  - `bash docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh`
  - `bash tests/installers/install_smoke.sh`
  - Cases 1, 2, and 5 in `manual_testing_playbook.md`

## Checkpoint cadence
- `CP1-ci-checkpoint` validates the contiguous slice group `DIWAS0`, `DIWAS1`.
- The boundary sits at `DIWAS1` because the total accepted slice count is two.
- `FZ-feature-cleanup` runs after `DIWAS1-integ` and `CP1-ci-checkpoint` complete.

## Validation commands
- Planning validators:
  - `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/dev-install-world-agent-staging"`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/dev-install-world-agent-staging"`
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/dev-install-world-agent-staging"`
  - `make planning-lint FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging"`
- Work-lift evidence:
  - `make pm-lift-pack PACK="docs/project_management/packs/draft/dev-install-world-agent-staging"`
  - `make pm-lift-pack PACK="docs/project_management/packs/draft/dev-install-world-agent-staging" EMIT_JSON=1`
  - `PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging"`
- Targeted execution validation:
  - `cargo test -p shell --test world_enable -- --nocapture`
  - `bash tests/installers/install_smoke.sh`
  - `bash docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh`

## Cross-pack constraints
- `ADR-0035` remains aligned with this pack’s contract and slice order.
- `scripts/substrate/install-substrate.sh` keeps production bundle semantics outside the accepted staged path rule.
- macOS and Windows remain parity-only surfaces for this feature.
