# best-effort-distro-package-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/implemented/best-effort-distro-package-manager/`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Full-planning wiring note: `tasks.json` now uses the schema v4 boundary-only task model for the accepted slice order `BEDPM0` → `BEDPM1` → `BEDPM2` → `BEDPM3`. `BEDPM3` is the single checkpoint-boundary slice and `tasks.json` mirrors that with `meta.checkpoint_boundaries = ["BEDPM3"]`.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["BEDPM0", "BEDPM1", "BEDPM2", "BEDPM3"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single end-of-feature checkpoint after BEDPM3. Total slices (4) meets defaults.min=4, so one checkpoint is allowed; CP1 waits until distro detection/reporting, override precedence/failure classes, wrapper/env-doc propagation, and hermetic evidence form one coherent hosted-installer contract before cross-platform compile parity and Linux behavior smoke run."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`BEDPM0..BEDPM3`) — hosted-installer contract completion + hermetic evidence seam

- Code-grounded boundary:
  - This feature reaches its first complete contract seam only after `BEDPM3`.
  - `BEDPM0` stabilizes best-effort distro detection, distro-family mapping, the `<unknown>` sentinel, and the stable stderr decision line on `scripts/substrate/install-substrate.sh`.
  - `BEDPM1` stabilizes override precedence, fixed `PATH` fallback order, and the actionable `2` / `3` / `4` failure classes.
  - `BEDPM2` stabilizes wrapper exit-code pass-through in `scripts/substrate/install.sh` and operator/env-doc propagation across `docs/INSTALLATION.md` and `docs/reference/env/contract.md`.
  - `BEDPM3` turns those contract rules into deterministic evidence via `tests/installers/pkg_manager_detection_smoke.sh`, the thin Linux smoke wrapper, and the manual evidence path.
  - A single checkpoint is valid because the accepted draft slice count is `4`, which meets the default minimum of `4` triads per checkpoint.
- Surfaces stabilized at CP1 (from `pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`, and `pre-planning/minimal_spec_draft.md`):
  - Installer contract and decision surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/install.sh`
    - `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md`
  - Operator-facing docs and env semantics:
    - `docs/INSTALLATION.md`
    - `docs/reference/env/contract.md`
    - `PKG_MANAGER`
    - `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
  - Validation/evidence surfaces:
    - `tests/installers/pkg_manager_detection_smoke.sh`
    - `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh`
    - `docs/project_management/packs/implemented/best-effort-distro-package-manager/manual_testing_playbook.md`
    - `slices/BEDPM0/BEDPM0-spec.md`
    - `slices/BEDPM1/BEDPM1-spec.md`
    - `slices/BEDPM2/BEDPM2-spec.md`
    - `slices/BEDPM3/BEDPM3-spec.md`
- Risk reduced by running cross-platform CI here:
  - Compile parity catches regressions in the shared hosted-installer entrypoints and planning-selected docs/scripts across `linux`, `macos`, and `windows` CI environments even though the behavior delta is Linux-only.
  - Linux behavior smoke validates the full precedence chain, fixed manager vocabulary, wrapper exit-code pass-through, and the selected `tests/installers/pkg_manager_detection_smoke.sh` plus `smoke/linux-smoke.sh` thin-wrapper model named by the canonical impact map.
  - Running the checkpoint only after `BEDPM3` avoids spending multi-OS CI before the hosted-installer contract, wrapper/doc propagation, repo test path, and operator evidence are coherent.
- Gate choice:
  - `feature_smoke = true` because the canonical impact map explicitly selects `tests/installers/pkg_manager_detection_smoke.sh` as the exact repo test path and selects `smoke/linux-smoke.sh` as the feature-local Linux wrapper.
  - `ci_testing = "quick"` because the canonical impact map keeps implementation changes out of `crates/`, `src/`, `crates/world*`, `crates/shim`, `crates/shell`, and `crates/world-service`, and scopes the work to installer shell scripts, installer docs, repo tests, and planning artifacts. If full planning expands the touch set into those areas or into workflow/infrastructure changes, update this plan first and then raise the checkpoint gate to `full`.

## Wiring Status

- `tasks.json` now defines:
  - normal-slice triads for `BEDPM0`, `BEDPM1`, and `BEDPM2`
  - the boundary slice `BEDPM3` as `BEDPM3-code`, `BEDPM3-test`, `BEDPM3-integ-core`, `BEDPM3-integ-linux`, `BEDPM3-integ-macos`, `BEDPM3-integ-windows`, and `BEDPM3-integ`
  - the checkpoint ops task `CP1-ci-checkpoint`
  - the feature cleanup ops task `FZ-feature-cleanup`
- Platform scope remains:
  - behavior smoke: `linux`
  - CI parity: `linux`, `macos`, `windows`
