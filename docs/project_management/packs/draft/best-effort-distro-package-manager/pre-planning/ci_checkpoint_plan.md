# best-effort-distro-package-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice triads (`*-integ`) yet, so this plan is not mechanically validated yet; the slice list below is derived from the canonical slice inventory in `pre-planning/spec_manifest.md` and the draft slice skeleton in `pre-planning/minimal_spec_draft.md`.

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
      "slices": ["BEDPM0", "BEDPM1", "BEDPM2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single end-of-feature checkpoint after BEDPM2. Total slices (3) is below defaults.min=4, so one checkpoint is allowed; CP1 waits until distro detection/reporting, override precedence/failure classes, and hermetic evidence form one coherent hosted-installer contract before cross-platform compile parity and Linux behavior smoke run."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`BEDPM0..BEDPM2`) — hosted-installer contract completion + hermetic evidence seam

- Code-grounded boundary:
  - This feature reaches its first complete contract seam only after `BEDPM2`.
  - `BEDPM0` stabilizes best-effort distro detection, distro-family mapping, the `<unknown>` sentinel, and the stable stderr decision line on `scripts/substrate/install-substrate.sh`.
  - `BEDPM1` stabilizes override precedence, fixed `PATH` fallback order, wrapper exit-code pass-through in `scripts/substrate/install.sh`, and the actionable `2` / `3` / `4` failure classes.
  - `BEDPM2` is the first slice that turns those contract rules into deterministic evidence via `tests/installers/pkg_manager_detection_smoke.sh`, the thin Linux smoke wrapper, and the manual evidence path.
  - A single checkpoint is valid because the accepted draft slice count is `3`, which is below the default minimum of `4` triads per checkpoint.
- Surfaces stabilized at CP1 (from `pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`, and `pre-planning/minimal_spec_draft.md`):
  - Installer contract and decision surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/install.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - Operator-facing docs and env semantics:
    - `docs/INSTALLATION.md`
    - `docs/reference/env/contract.md`
    - `PKG_MANAGER`
    - `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
  - Validation/evidence surfaces:
    - `tests/installers/pkg_manager_detection_smoke.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
    - `slices/BEDPM0/BEDPM0-spec.md`
    - `slices/BEDPM1/BEDPM1-spec.md`
    - `slices/BEDPM2/BEDPM2-spec.md`
- Risk reduced by running cross-platform CI here:
  - Compile parity catches regressions in the shared hosted-installer entrypoints and planning-selected docs/scripts across `linux`, `macos`, and `windows` CI environments even though the behavior delta is Linux-only.
  - Linux behavior smoke validates the full precedence chain, fixed manager vocabulary, wrapper exit-code pass-through, and the selected `tests/installers/pkg_manager_detection_smoke.sh` plus `smoke/linux-smoke.sh` thin-wrapper model named by the canonical impact map.
  - Running the checkpoint only after `BEDPM2` avoids spending multi-OS CI before the hosted-installer contract, repo test path, and operator evidence are coherent.
- Gate choice:
  - `feature_smoke = true` because the canonical impact map explicitly selects `tests/installers/pkg_manager_detection_smoke.sh` as the exact repo test path and selects `smoke/linux-smoke.sh` as the feature-local Linux wrapper.
  - `ci_testing = "quick"` because the canonical impact map keeps implementation changes out of `crates/`, `src/`, `crates/world*`, `crates/shim`, `crates/shell`, and `crates/world-agent`, and scopes the work to installer shell scripts, installer docs, repo tests, and planning artifacts. If full planning expands the touch set into those areas or into workflow/infrastructure changes, update this plan first and then raise the checkpoint gate to `full`.

## Follow-ups

This plan cannot be mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint ops tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`

…complete these wiring steps:

1) Confirm slice ids and ordering
   - Ensure the final slice ids in `tasks.json` match the accepted slice ids (`BEDPM0`, `BEDPM1`, `BEDPM2`) unless full planning explicitly splits or merges slices.
   - Ensure this plan’s JSON `slices` list matches the deterministic slice order from `tasks.json`.

2) Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Set `meta.checkpoint_boundaries = ["BEDPM2"]` to match the checkpoint group boundary (the last slice in each checkpoint group).

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["BEDPM2-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/best-effort-distro-package-manager/kickoff_prompts/CP1-ci-checkpoint.md`

4) Preserve the current platform-scope split unless this plan is updated first
   - Keep `meta.behavior_platforms_required = ["linux"]`.
   - Keep `meta.ci_parity_platforms_required = ["linux", "macos", "windows"]`.
   - If full planning decides to widen or narrow either scope, update this plan before changing `tasks.json`.

5) Reconcile upstream planning drift during full planning
   - `pre-planning/spec_manifest.md` still says `pre-planning/ci_checkpoint_plan.md` is not part of the required-doc set and still describes a Linux-only task model with `meta.cross_platform = false`.
   - Full planning must either align those statements to this schema-v4 cross-platform baseline or update this plan and the pack metadata together.

6) If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.
