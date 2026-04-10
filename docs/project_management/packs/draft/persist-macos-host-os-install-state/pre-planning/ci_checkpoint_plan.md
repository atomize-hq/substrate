# persist-macos-host-os-install-state — CI checkpoint plan

This file defines where later multi-platform verification is expected to happen for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint intent during pre-planning.
- If you discover a mismatch between the plan and reality, update this plan first.
- This document remains advisory until downstream FSE planning or decomposition turns the checkpoint cadence into concrete execution behavior.

## Machine-readable plan

```json
{
  "version": 1,
  "defaults": {
    "min_candidates_per_checkpoint": 2,
    "max_candidates_per_checkpoint": 6
  },
  "platform_scope": {
    "behavior_platforms": ["macos"],
    "ci_parity_platforms": ["linux", "macos", "windows"],
    "wsl_required": false
  },
  "checkpoints": [
    {
      "checkpoint_id": "CP1",
      "candidate_ids": ["PMHOS-S1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "none"
      },
      "rationale": "High-risk exception to the default minimum group size. This checkpoint isolates the shared install_state writer seam so compile parity and targeted shared smoke run before the pack moves into macOS fixture and operator-doc work."
    },
    {
      "checkpoint_id": "CP2",
      "candidate_ids": ["PMHOS-S2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted"
      },
      "rationale": "This checkpoint closes the macOS behavior evidence and doc-alignment seam. It runs after the validation topology lands so the heavier macOS-host and no-change proof gates run once at the final user-visible boundary."
    }
  ]
}
```

## Human-readable rationale

### Platform scope

- Behavior change:
  - macOS hosted install and macOS hosted `--no-world`
- CI parity and no-change confirmation:
  - Linux
  - macOS
  - Windows
- WSL:
  - out of scope in current pack evidence because the touch set does not include WSL-owned implementation or validation surfaces

### CP1

- Why the boundary is code-grounded:
  - `PMHOS-S1` locks the shared writer contract before validation topology and docs move. The impact map ties this seam to `scripts/substrate/install-substrate.sh` and shared `install_state.json` semantics.
- What surfaces are stabilized:
  - additive `install_state.json` write semantics
  - same-directory temp-file and atomic-replace flow
  - preservation of unknown keys and pre-existing Linux metadata
  - warning-only recovery and dry-run no-write posture
- What risk is reduced:
  - compile parity and targeted `tests/installers/install_state_smoke.sh` coverage catch shared writer regressions before the pack absorbs macOS fixture growth and doc reconciliation
- What still needs confirmation later:
  - downstream planning needs to lock the exact CI dispatch cadence and exact targeted smoke subset for this boundary

### CP2

- Why the boundary is code-grounded:
  - `PMHOS-S2` owns the final validation split and operator-doc reconciliation. The impact map ties this seam to `tests/mac/installer_parity_fixture.sh`, `tests/installers/install_state_smoke.sh`, and `docs/INSTALLATION.md`.
- What surfaces are stabilized:
  - hosted macOS install and hosted `--no-world` parity evidence
  - warning-only OS-detail capture degradation evidence
  - Linux and Windows no-change proof lines
  - operator-facing installation guidance
- What risk is reduced:
  - compile parity, macOS fixture smoke, and targeted deeper CI testing run after the user-visible behavior and docs align, which avoids redundant macOS-host validation before the validation topology exists
- What still needs confirmation later:
  - downstream planning needs to lock the exact targeted CI suite and final proof line for Windows no-change validation

## Follow-ups

- Replace draft candidate ids `PMHOS-S1` and `PMHOS-S2` with final downstream identifiers once they exist.
- Confirm the exact platform scope and verification cadence once downstream planning stabilizes the touched surfaces.
- Convert checkpoint intent into concrete execution wiring only in the downstream subsystem that owns execution.
- Lock the final Windows no-change proof line when downstream planning fixes the final smoke-coverage split.
