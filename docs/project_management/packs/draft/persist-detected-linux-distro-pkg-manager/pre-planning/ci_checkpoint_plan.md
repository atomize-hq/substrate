# persist-detected-linux-distro-pkg-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- Slice specs: see `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slices exist in `tasks.json`).

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
      "slices": ["PDL0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single-slice feature per minimal_spec_draft: CP1 validates the PDL0 core integration branch at the contract-completion seam where install_state.json persistence + smoke assertions become end-to-end coherent. Feature smoke is required for the Linux-only behavior delta; compile parity + quick CI testing reduce cross-platform drift risk."
    }
  ]
}
```

## Human-readable rationale (required)

CP1 is the only checkpoint for this feature because the pre-planning slice skeleton defines one slice (`PDL0`).

Boundary rationale (code-grounded):
- Contract completion seam: by the end of `PDL0`, the installer persists the new `host_state.platform.*` metadata into `$SUBSTRATE_HOME/install_state.json` (`schema_version=1`) and the installer smoke assertions cover the file-exists + new-key requirements.
- Surfaces stabilized at this checkpoint (from `spec_manifest.md`):
  - Operator contract and platform scope (Linux-only behavior delta): `contract.md`
  - Schema contract for additive keys under `schema_version=1`: `install-state-schema-spec.md`
  - Decision resolution for “must exist” vs “best-effort persistence” tension: `decision_register.md`
  - Slice acceptance and smoke assertions: `slices/PDL0/PDL0-spec.md`
- Risk reduced by running cross-platform CI here (from `impact_map.md`):
  - Installer script changes + file-write semantics are high-impact; running CI parity at the checkpoint reduces the chance of landing a change that breaks compilation on macOS/Windows even when the Linux behavior delta is the main intent.
  - Running behavior smoke at the checkpoint ensures the Linux installer persistence contract is validated before proceeding past the seam.

## Follow-ups

This plan cannot be mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`

…complete these wiring steps:

1) Confirm slice ids
   - Ensure the slice ids in `tasks.json` match the accepted slice ids (expect `PDL0`).
   - Ensure this plan’s JSON `slices` list matches the deterministic slice order from `tasks.json`.

2) Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Set `meta.checkpoint_boundaries = ["PDL0"]` to match the checkpoint boundary (the last slice in each checkpoint group).

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["PDL0-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/kickoff_prompts/CP1-ci-checkpoint.md`

4) If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.

