# persist-macos-host-os-install-state — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms for promotion of the staged candidates:
  - Behavior smoke platforms: `macos`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4 cross-platform automation packs, update `tasks.json` `meta.checkpoint_boundaries` to list the last slice in each checkpoint group once slice tasks exist.
- Pre-planning note: canonical `tasks.json` does not define slice triads (`*-integ`) or checkpoint tasks yet, so this first pass uses the canonical slice inventory from `pre-planning/spec_manifest.md` and the draft slice skeleton from `pre-planning/minimal_spec_draft.md`.

## Machine-readable plan (linted)

Pre-planning note:
- Mechanical validation waits for full-planning task wiring because canonical `tasks.json` has no slice triads or checkpoint tasks yet.

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
      "slices": ["PMHOIS0", "PMHOIS1", "PMHOIS2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single end-of-feature checkpoint after PMHOIS2. Total draft slices is 3, which is below defaults.min=4. CP1 waits until the contract seam, installer writer seam, validation seam, and operator-doc seam form one coherent hosted macOS persistence proof point."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`PMHOIS0..PMHOIS2`) — hosted macOS persistence proof seam

- Code-grounded boundary:
  - `PMHOIS0` freezes the hosted macOS contract backbone, schema invariants, compatibility rules, and decision seams.
  - `PMHOIS1` freezes the writer path in `scripts/substrate/install-substrate.sh`, including same-directory temp-file replace ordering, warning-only degradation, and the `--dry-run` no-write branch.
  - `PMHOIS2` freezes the automated validation and operator-doc seam by aligning `tests/mac/installer_parity_fixture.sh`, `tests/installers/install_state_smoke.sh`, and `docs/INSTALLATION.md` to the same contract.
  - A single checkpoint is valid because the draft slice count is `3`, which is below the default minimum of `4`.
- Surfaces stabilized at CP1:
  - `scripts/substrate/install-substrate.sh`
  - `tests/mac/installer_parity_fixture.sh`
  - `tests/installers/install_state_smoke.sh`
  - `docs/INSTALLATION.md`
  - full-planning contract surfaces named by `pre-planning/spec_manifest.md`:
    - `contract.md`
    - `install-state-schema-spec.md`
    - `filesystem-semantics-spec.md`
    - `platform-parity-spec.md`
    - `compatibility-spec.md`
    - `decision_register.md`
    - `slices/PMHOIS0/PMHOIS0-spec.md`
    - `slices/PMHOIS1/PMHOIS1-spec.md`
    - `slices/PMHOIS2/PMHOIS2-spec.md`
- Risk reduced by running cross-platform CI here:
  - compile parity catches shared shell-script, test-harness, and documentation regressions across `linux`, `macos`, and `windows`
  - feature smoke validates the only new behavior platform in this pack: `macos`
  - macOS smoke evidence covers hosted install success, hosted `--no-world`, and hosted `--dry-run` no-write semantics named in `pre-planning/impact_map.md`
  - the checkpoint catches drift between implementation, automated validation, and operator guidance before triad wiring begins
- Gate choice:
  - `feature_smoke = true` because the user-visible behavior delta is hosted macOS persistence and the platform standard ties smoke ownership to behavior platforms
  - `ci_testing = "quick"` because the canonical touch set stays out of `src/`, `crates/`, `crates/world*`, `crates/shim/`, `crates/shell/`, and `crates/world-agent/`

## Tasks.json wiring basis

This pre-planning pass uses the schema-v4 boundary-only checkpoint model.

Current staged baseline:
- `meta.behavior_platforms_required = ["macos"]`
- `meta.ci_parity_platforms_required = ["linux", "macos", "windows"]`

Full-planning wiring target:
- `meta.checkpoint_boundaries = ["PMHOIS2"]`
- `CP1-ci-checkpoint` depends on `PMHOIS2-integ-core`
- `PMHOIS2-integ-macos`, `PMHOIS2-integ-linux`, and `PMHOIS2-integ-windows` depend on `PMHOIS2-integ-core` and `CP1-ci-checkpoint`
- `PMHOIS2-integ` depends on `PMHOIS2-integ-core` plus the platform-fix tasks present in the accepted triad graph
- non-boundary slices stay on normal `code`, `test`, and final `integ` tasks only

## Follow-ups

- Add slice triad tasks for `PMHOIS0`, `PMHOIS1`, and `PMHOIS2`.
- Add the ops task `CP1-ci-checkpoint` with dependencies tied to the ending slice integration task.
- Set `tasks.json` `meta.checkpoint_boundaries = ["PMHOIS2"]`.
- Replace the draft slice list in this plan if full planning accepts different slice ids.
- Keep the accepted slice order and the JSON `checkpoints[].slices` list in sync.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-macos-host-os-install-state"` after slice tasks and checkpoint wiring exist.
