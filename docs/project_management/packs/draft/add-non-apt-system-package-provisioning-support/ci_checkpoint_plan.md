# add-non-apt-system-package-provisioning-support — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- Slice specs: see `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).

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
      "slices": ["C0", "C1", "C2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Pre-planning first pass: this pack currently implies three slices (C0 probe, C1 pacman provisioning, C2 runtime fail-early + validation + docs). Total slices < defaults.min, so a single end-to-end checkpoint is acceptable. Replace placeholder slice ids with real slice ids once tasks.json triads exist."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`C0..C2`) — end-to-end stabilization

Code-grounded boundary:
- Contract-completion seam: by the end of `C2`, the provisioning-time probe (`C0`), the provisioning execution contract (`C1`), and the runtime fail-early + operator doc updates + required validation assertions (`C2`) should compose a coherent operator workflow for `substrate world enable --provision-deps` and the affected runtime `world deps` surfaces.

Surfaces stabilized at this checkpoint (from `spec_manifest.md`):
- Operator contract + remediation/exit-code invariants: `contract.md`
- Provisioning protocol + guard rails: `world-deps-pacman-provisioning-protocol-spec.md`
- Cross-platform behavior smoke procedures:
  - `smoke/linux-smoke.sh` (Linux host-native unsupported behavior)
  - `smoke/macos-smoke.sh` (macOS Lima backend)
  - `smoke/windows-smoke.ps1` (Windows WSL backend)
- Slice behaviors:
  - `C0` probe algorithm and derived manager enum
  - `C1` pacman requirement derivation + command contract + deterministic failure mapping
  - `C2` runtime fail-early scope + required test/docs update targets

Risk reduced by running cross-platform CI here (from `impact_map.md`):
- Platform-specific routing and backend capability differences (Linux host-native vs macOS Lima vs Windows WSL) can cause compile and behavior drift even when the primary dev platform is green.
- Provisioning is a high-risk surface (intentional opt-in “mutation” inside the world) and must preserve the “no host OS mutation” posture; smoke coverage across the behavior platforms is the quickest regression signal for support/unsupported cases.

## Follow-ups

This plan cannot be mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`

…complete these wiring steps:

1) Replace placeholder slice ids
   - Replace `C0/C1/C2` with stable, feature-derived slice ids per `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`.
   - Update this plan’s JSON `slices` list to match the real slice ids in deterministic order.

2) Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Set `meta.checkpoint_boundaries = ["<LAST_SLICE_ID>"]` to match the checkpoint boundary (the last slice in each checkpoint group).

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["<LAST_SLICE_ID>-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/kickoff_prompts/CP1-ci-checkpoint.md`
   - If additional checkpoints are added, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.

