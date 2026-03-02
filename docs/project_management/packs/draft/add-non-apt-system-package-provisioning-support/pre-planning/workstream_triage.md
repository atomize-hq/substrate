# add-non-apt-system-package-provisioning-support — workstream triage

## Evidence (inputs)

- Step sentinels:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/impact-map/last_message.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/CI-checkpoint/last_message.md`
- Canonical artifacts used:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` (`meta.slice_spec_version=2`)
- Lift runs (logs):
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_pack.txt`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_pack.json`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_intake.txt`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_intake.json`

## Lift signal (sizing / split pressure)

- Strict pack: `tasks.json` has `meta.slice_spec_version=2`, so impact-map-derived lift is meaningful.
- Pack-derived lift (from `impact_map.md`):
  - `lift_score=88`, `estimated_slices=8`, triggers include `split_required:estimated_slices>3`.
- ADR-derived lift (from ADR-0033 intake vector):
  - `lift_score=27`, `estimated_slices=3`, trigger `likely_split:lift_score>24`.
- Implication for planning: expect multiple high-churn seams (schema, provisioning exec, runtime fail-early, operator docs/scripts, cross-platform validation) and plan to split the current 3-slice skeleton.

## Proposed workstreams (parallelizable)

### ANS-PWS-contract — Contract + decision register (operator-facing truth)

- Goal: Pin deterministic operator contract and close underspecified decisions so slices can be planned/tested.
- Owned surfaces:
  - Pack: `contract.md`, `decision_register.md`
  - Contract seams from `spec_manifest.md`: exit codes (`0/2/3/4/5`), `--dry-run`/`--verbose` output invariants, mismatch policy, runtime fail-early scope for `deps current install <ITEM...>`, platform/backends matrix (Linux host-native vs macOS Lima vs Windows WSL).
- Dependencies: must reconcile with Gate G1 (cross-pack contract/schema reconciliation) before finalizing runtime behavior language.
- Proposed slices/triads to create during full planning: DRs (expected DR-0002..DR-0005) + acceptance criteria IDs wired into slice triads in `tasks.json`.

### ANS-PWS-tasks_checkpoints — Pack backbone (plan + tasks + checkpoints wiring)

- Goal: Make the pack mechanically strict-ready (task graph + checkpoints + validation commands).
- Owned surfaces:
  - Pack: `plan.md`, `tasks.json`, `ci_checkpoint_plan.md`
  - Wiring: checkpoint boundaries (`tasks.json meta.checkpoint_boundaries`) + checkpoint ops tasks/kickoffs (per `ci_checkpoint_plan.md` follow-ups).
- Dependencies: depends on slice skeleton update (see “Slice skeleton recommendations”) and ANS-PWS-contract for deterministic validation commands.
- Proposed slices/triads to create during full planning: `ANS*-{code,test,integ-*}` triads for each accepted slice; add `CP*-ci-checkpoint` ops tasks.

### ANS-PWS-os_probe — ANS0: in-world OS/manager probe determinism

- Goal: Deterministic, testable OS-family/manager detection (in-world; not host PATH-based).
- Owned surfaces (from `minimal_spec_draft.md` + `impact_map.md`):
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
  - (possible) `crates/world-agent/src/service.rs` if probe needs world-agent behavior changes
  - Slice spec: `slices/ANS0/ANS0-spec.md`
- Dependencies: ANS-PWS-contract must pin precedence rules (`/etc/os-release` vs manager presence) and ambiguity handling.
- Proposed slices/triads: `ANS0-code`, `ANS0-test` (classification edge cases), `ANS0-integ` (probe result drives manager selection).

### ANS-PWS-schema_inventory — Schema + inventory validation + derivation (candidate split from ANS1)

- Goal: Add/validate `install.method=pacman` + `install.pacman` shape and make requirement derivation deterministic.
- Owned surfaces:
  - `crates/shell/src/builtins/world_deps/inventory.rs`
  - `crates/shell/src/builtins/world_deps/errors.rs`
  - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - Test surface: `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
  - Cross-pack schema authority: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- Dependencies: Gate G1 (cross-pack contract/schema reconciliation) and ANS-PWS-contract (mismatch policy + runtime/provisioning invariants).
- Proposed slices/triads: `ANS1-code` (schema/validation/derivation), `ANS1-test`, `ANS1-integ` (enabled-set → derived requirement sets).

### ANS-PWS-provisioning_wiring — Provisioning-time execution wiring (candidate split from ANS1)

- Goal: Implement/validate `world enable --provision-deps` provisioning behavior for pacman worlds (including `--dry-run`/`--verbose`) without weakening hardened runtime rules.
- Owned surfaces (from `impact_map.md`):
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
  - `scripts/substrate/world-enable.sh`
  - `crates/world-agent/src/service.rs` (guard rails / request profile, if required)
  - Test surface: `crates/shell/tests/world_enable.rs`
- Dependencies: ANS-PWS-os_probe (probe semantics), ANS-PWS-schema_inventory (schema + derivation), ANS-PWS-contract (exit codes + output invariants).
- Proposed slices/triads: `ANS2-code`, `ANS2-test`, `ANS2-integ` (unsupported backend behavior + deterministic remediation).

### ANS-PWS-runtime_fail_early — Runtime fail-early + health guidance (candidate split from ANS2)

- Goal: Ensure runtime `world deps current sync|install` never invokes OS package managers and emits deterministic remediation.
- Owned surfaces (from `impact_map.md`):
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - `crates/shell/src/builtins/world_deps/errors.rs`
  - `crates/shell/src/builtins/health.rs`
  - Test surface: `crates/shell/tests/world_deps_apt_install_wdp5.rs` (repurpose to assert “no runtime apt”)
- Dependencies: ANS-PWS-contract must pin fail-early scope (enabled set vs explicit args) and remediation invariants.
- Proposed slices/triads: `ANS3-code`, `ANS3-test`, `ANS3-integ` (mixed enabled sets; ensure no dead-end workflows).

### ANS-PWS-docs_validation — Operator docs + scripts + smoke/manual validation (candidate split from ANS2)

- Goal: Align operator docs/scripts with the new provisioning-time workflow and cross-platform behavior, without “apt-shaped” guidance on pacman worlds.
- Owned surfaces (from `impact_map.md`):
  - Docs: `docs/reference/world/deps/README.md`, `docs/internals/world/deps.md`, `docs/WORLD.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/COMMANDS.md`, `docs/cross-platform/wsl_world_troubleshooting.md`
  - Scripts: `scripts/substrate/install-substrate.sh`
  - Validation artifacts: `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`
- Dependencies: ANS-PWS-contract wording and ANS-PWS-provisioning_wiring/ANS-PWS-runtime_fail_early behavior must be stable enough to document.
- Proposed slices/triads: `ANS4-docs`, `ANS5-ops` (or the repo’s standard task types) + per-platform smoke triads if required by automation.

## Sequencing + gates (hard ordering)

- Gate G0 — Resolve ADR feature-dir drift
  - ADR-0033 references `.../world-deps-pacman-provisioning/`, but this pack is `.../add-non-apt-system-package-provisioning-support/` (see `spec_manifest.md` follow-up #1). Full planning must converge on exactly one canonical directory and update links to avoid dual-authority specs.
- Gate G1 — Cross-pack contract/schema reconciliation must start early
  - `world-deps-packages-bundles-contract/contract.md` is in the impact-map touch set and is already flagged as contradictory with ADR-0033 (see `spec_manifest.md` follow-up #9). Resolve authority/wording before locking runtime behavior tests.
- Gate G2 — Decision register minimum set before task wiring
  - OS detection precedence, pacman invocation + idempotency, mismatch policy, runtime fail-early scope, provisioning isolation/guard-rails decision, and `--verbose` guarantees must be pinned before generating strict `tasks.json` triads/checkpoints.
- Gate G3 — If slice skeleton changes, update CI plan and checkpoint wiring first
  - `ci_checkpoint_plan.md` currently has a single `CP1` for `ANS0..ANS2`; if planning accepts the split recommendations below, update `ci_checkpoint_plan.md` + `tasks.json meta.checkpoint_boundaries` and add intermediate checkpoints only where risk seams justify the additional CI cost.

## Risks + unknowns (must resolve during full planning)

- Deterministic OS-family detection rules (`/etc/os-release` + manager presence) are underspecified (spec-manifest follow-up #2).
- Pacman invocation contract (flags, db refresh posture, idempotency, failure→exit mapping) is underspecified (#3).
- Mixed-method enabled sets (`apt` + `pacman`) mismatch policy must be singular and deterministic (#4).
- Runtime fail-early scope for `deps current install <ITEM...>` must be pinned (#5).
- Provisioning isolation/guard-rails approach (world-agent request profile vs internal guard rails) must be selected and reflected in docs/tests (#6).
- `--verbose` minimum guarantees (streams/content) must be defined (#7).
- Operator-doc update targets must be enumerated by exact path/headings and must link to `contract.md` rather than restating it (#8).

## Slice skeleton recommendations (required)

Starting point: `minimal_spec_draft.md` draft skeleton is `ANS0/ANS1/ANS2`.

Based on pack-derived lift (`estimated_slices=8`) and the authored Touch Set breadth:

- `SPLIT` `ANS1` → (`ANS1` + `ANS2`)
  - `ANS1`: schema + inventory validation + requirement derivation (`install.method=pacman`, `install.pacman`)
  - `ANS2`: provisioning-time execution wiring (`world enable --provision-deps`, pacman invocation, world-agent guard rails if needed)
- `RENAME` current `ANS2` → `ANS3` (runtime fail-early core)
- `ADD` `ANS4` (operator docs + scripts sweep; remediation messaging alignment)
- `ADD` `ANS5` (manual testing playbook + smoke scripts + cross-platform validation scaffolding)

Follow-up: if world-agent guard-rails require meaningful protocol/profile work, consider an additional `ADD` slice dedicated to world-agent changes to avoid churn across shell work.

## Follow-ups

- Re-run `make pm-lift-pack ...` after any significant Touch Set edits to confirm lift reflects the accepted scope.
- Once slice ids are finalized, wire `tasks.json` triads + `meta.checkpoint_boundaries` and make `ci_checkpoint_plan.md` mechanically valid per its follow-ups.
