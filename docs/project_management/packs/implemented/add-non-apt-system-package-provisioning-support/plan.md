# add-non-apt-system-package-provisioning-support — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- Orchestration branch: `feat/add-non-apt-system-package-provisioning-support`
- Canonical planning inputs:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/workstream_triage.md`

## Goal
- Make manager-aware system-package provisioning execution-ready without downgrading cross-platform validation:
  - detect the world package manager in-world and fail closed on unsupported or contradictory states
  - add `install.method=pacman` plus deterministic inventory rendering and validation
  - route `substrate world enable --provision-deps` through one exact pacman provisioning path
  - keep runtime `substrate world deps current sync|install` read-only for system-package managers with deterministic remediation
  - lock Linux, macOS, and Windows validation evidence plus reconciliation targets for shared contract docs

## Guardrails
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- This remains a schema-v4 automation pack with `cross_platform=true`.
- Do not collapse the accepted slice order `NASP0` → `NASP1` → `NASP2` → `NASP3` → `NASP4`.
- Each task must fit within the triad context budget limit (<= 108,800 tokens).

## Triads
- `NASP0`: world-manager probe and provisioning support gate.
- `NASP1`: pacman schema extension and inventory views.
- `NASP2`: provisioning routing and pacman command execution.
  - Checkpoint boundary with `NASP2-integ-core`, `NASP2-integ-linux`, `NASP2-integ-macos`, `NASP2-integ-windows`, `NASP2-integ`, and `CP1-ci-checkpoint`.
- `NASP3`: runtime fail-early and manager-aware remediation.
- `NASP4`: validation evidence and contract reconciliation.
  - Checkpoint boundary with `NASP4-integ-core`, `NASP4-integ-linux`, `NASP4-integ-macos`, `NASP4-integ-windows`, `NASP4-integ`, and `CP2-ci-checkpoint`.

## Checkpoint cadence
- `CP1-ci-checkpoint` validates the contiguous slice group `NASP0`, `NASP1`, `NASP2`.
- `CP2-ci-checkpoint` validates the contiguous slice group `NASP3`, `NASP4`.
- `NASP3-code` and `NASP3-test` must not start until `CP1-ci-checkpoint` completes.
- `FZ-feature-cleanup` runs only after `NASP4-integ` and `CP2-ci-checkpoint` complete.

## Validation commands
- Planning validators:
  - `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
- Targeted Rust validation for execution work:
  - `cargo test -p shell --test world_enable -- --nocapture`
  - `cargo test -p shell --test world_deps_inventory_validation_wdp0 -- --nocapture`
  - `cargo test -p shell --test world_deps_inventory_views -- --nocapture`
  - `cargo test -p shell --test world_deps_current_dry_run_wdp3 -- --nocapture`
  - `cargo test -p shell --test world_deps_apt_install_wdp5 -- --nocapture`
- Smoke validation:
  - `bash docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - `bash docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - `pwsh -File docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- Manual validation:
  - execute the four cases in `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`

## Cross-pack constraints
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` must defer to this pack for the shared manager-aware `--provision-deps` contract once reconciliation lands.
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` remains authoritative for unchanged inventory merge and enabled-set semantics.
- Do not invent a new protocol, env var, or telemetry surface while implementing this pack.
