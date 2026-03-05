# world-deps-apt-provisioning — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- Orchestration branch: `feat/world-deps-apt-provisioning`
- Canonical pre-planning inputs:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/workstream_triage.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/alignment_report.md`

## Goal
- Add an explicit provisioning-time APT workflow: `substrate world enable --provision-deps [--dry-run] [--verbose]`.
- Enforce runtime invariants for `substrate world deps current sync|install`:
  - no runtime `apt`/`apt-get` or mutating `dpkg`,
  - fail early with deterministic remediation that includes `substrate world enable --provision-deps`.
- Preserve “no host OS mutation” guard rails on Linux host-native.
- Keep behavior cross-platform (Linux/macOS/Windows) with checkpoint-gated CI validation.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Each task (code/test/integ) must fit within 108,800 tokens of context.

## Triads (slice order)
- `WDAP0`: provisioning-time APT surface (`world enable --provision-deps`).
- `WDAP1`: runtime fail-early + remediation for APT-backed items.

Note:
- The checkpoint plan partitions only `WDAP0` and `WDAP1`; `tasks.json` wires only these slices.

## CI checkpoints (cross-platform)
- Checkpoint plan: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
- Boundary slices (schema v4): `WDAP0`, `WDAP1` (see `tasks.json` `meta.checkpoint_boundaries`)
- Ops tasks:
  - `CP1-ci-checkpoint` (validates `WDAP0-integ-core`)
  - `CP2-ci-checkpoint` (validates `WDAP1-integ-core`)

## Validation artifacts (required by this pack)
- Manual playbook: `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
- Smoke scripts:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`

## Planning validation (mechanical)
- `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"`
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"`
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"`
