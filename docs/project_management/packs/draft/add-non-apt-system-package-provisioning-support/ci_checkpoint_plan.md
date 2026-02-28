# add-non-apt-system-package-provisioning-support — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice triads (`*-integ`) yet, so this plan is not mechanically validated yet; the slice list below is derived from the draft slice skeleton in `minimal_spec_draft.md`.

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
      "slices": ["ANS0", "ANS1", "ANS2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-to-end checkpoint after ANS2 where provisioning-time manager-aware provisioning and runtime fail-early/remediation posture compose a coherent operator experience. Total slices (3) is below defaults.min=4, so one checkpoint is acceptable; add an intermediate checkpoint only if planning splits slices or a standalone ANS1 risk seam is identified."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`ANS0..ANS2`) — end-to-end manager-aware provisioning + runtime fail-early + operator-doc seam

- Code-grounded boundary (contract completion seam):
  - By the end of `ANS2`, the feature’s core operator contract should be coherent end-to-end:
    - provisioning-time system packages are explicit (`substrate world enable --provision-deps`), manager-aware (APT vs pacman), and fail deterministically on mismatches/unsupported backends, and
    - runtime `substrate world deps current sync|install` is fail-closed for system-package methods (never executes `apt`/`pacman`) with deterministic remediation including the exact provisioning command string.
- Surfaces stabilized at this checkpoint (from `spec_manifest.md` ownership + `minimal_spec_draft.md` slice skeleton):
  - Operator contract + exit-code/remediation invariants: `contract.md` (to be created by this pack)
  - Cross-cutting invariants (fail-closed, no host OS mutation, in-world probe only): `minimal_spec_draft.md`
  - Slice specs:
    - `slices/ANS0/ANS0-spec.md` (in-world OS/manager probe rules)
    - `slices/ANS1/ANS1-spec.md` (schema extension + pacman provisioning semantics)
    - `slices/ANS2/ANS2-spec.md` (runtime fail-early + operator-doc updates)
  - Cross-platform behavior smoke procedures (to be created by this pack; see `impact_map.md` touch set):
    - `smoke/linux-smoke.sh`
    - `smoke/macos-smoke.sh`
    - `smoke/windows-smoke.ps1`
- Risk reduced by running cross-platform CI here (from `impact_map.md`):
  - Backend capability divergence (Linux host-native vs macOS Lima vs Windows WSL) is a frequent source of “primary dev platform is green but other platforms fail” drift; compile parity + behavior smoke at this seam catches it early.
  - Provisioning is a safety-sensitive mutation surface; cross-platform smoke validates the “no host OS mutation” invariant and confirms manager-aware remediation messaging is not “apt-shaped” on pacman worlds.
  - This feature touches `crates/world-agent/src/service.rs`, CLI routing, and multiple operator-facing docs/scripts; full CI testing provides broader regression signal than compile-only parity.

## Follow-ups

This plan cannot be mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint ops tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`

…complete these wiring steps:

1) Confirm slice ids and ordering
   - Ensure the final slice ids in `tasks.json` match the accepted slice ids (expect `ANS0`, `ANS1`, `ANS2` unless planning splits/merges).
   - Ensure this plan’s JSON `slices` list matches the deterministic slice order from `tasks.json`.

2) Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Set `meta.checkpoint_boundaries = ["ANS2"]` to match the checkpoint group boundary (the last slice in each checkpoint group).

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["ANS2-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/kickoff_prompts/CP1-ci-checkpoint.md`

4) If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.

